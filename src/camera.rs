use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct CapturedFrame {
    pub bytes: Vec<u8>,
    pub content_type: &'static str,
    pub file_name: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraOption {
    pub index: u32,
    pub device_id: String,
    pub label: String,
}

impl fmt::Display for CameraOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.label)
    }
}

pub struct CameraWorker {
    latest_frame: Arc<Mutex<Option<CapturedFrame>>>,
    last_error: Arc<Mutex<Option<String>>>,
    frame_revision: Arc<AtomicU64>,
    stop_flag: Arc<AtomicBool>,
    join_handle: Option<JoinHandle<()>>,
}

/// Cloneable handle to a camera worker — can be sent to async tasks.
#[derive(Clone)]
pub struct SharedCameraHandle {
    latest_frame: Arc<Mutex<Option<CapturedFrame>>>,
    last_error: Arc<Mutex<Option<String>>>,
    frame_revision: Arc<AtomicU64>,
}

impl SharedCameraHandle {
    pub fn latest_frame(&self) -> Result<CapturedFrame, String> {
        if let Some(error) = self
            .last_error
            .lock()
            .map_err(|_| "Failed to read camera error state".to_string())?
            .clone()
        {
            return Err(error);
        }

        self.latest_frame
            .lock()
            .map_err(|_| "Failed to read camera frame buffer".to_string())?
            .clone()
            .ok_or_else(|| "No camera frame available yet".to_string())
    }

    /// Waits for several new frames so the captured image reflects the current
    /// scene rather than what the camera was seeing before the call.
    pub fn capture_fresh(&self) -> Result<CapturedFrame, String> {
        let start_revision = self.frame_revision.load(Ordering::Relaxed);
        let min_revision = start_revision.saturating_add(3);
        let earliest_capture = Instant::now() + Duration::from_millis(600);
        let deadline = Instant::now() + Duration::from_secs(5);

        while Instant::now() < deadline {
            if let Some(error) = self
                .last_error
                .lock()
                .map_err(|_| "Failed to read camera error state".to_string())?
                .clone()
            {
                return Err(error);
            }

            let current_revision = self.frame_revision.load(Ordering::Relaxed);
            if Instant::now() >= earliest_capture && current_revision >= min_revision {
                return self.latest_frame();
            }

            thread::sleep(Duration::from_millis(30));
        }

        self.latest_frame()
    }
}

impl CameraWorker {
    pub fn start_nonblocking(camera: CameraOption) -> Self {
        let latest_frame = Arc::new(Mutex::new(None));
        let last_error = Arc::new(Mutex::new(None));
        let frame_revision = Arc::new(AtomicU64::new(0));
        let stop_flag = Arc::new(AtomicBool::new(false));

        let thread_latest_frame = Arc::clone(&latest_frame);
        let thread_last_error = Arc::clone(&last_error);
        let thread_frame_revision = Arc::clone(&frame_revision);
        let thread_stop_flag = Arc::clone(&stop_flag);
        let join_handle = thread::spawn(move || {
            camera_worker_loop(
                camera,
                thread_latest_frame,
                thread_last_error,
                thread_frame_revision,
                thread_stop_flag,
            )
        });

        Self {
            latest_frame,
            last_error,
            frame_revision,
            stop_flag,
            join_handle: Some(join_handle),
        }
    }

    pub fn latest_frame(&self) -> Result<CapturedFrame, String> {
        if let Some(error) = self
            .last_error
            .lock()
            .map_err(|_| "Failed to read camera error state".to_string())?
            .clone()
        {
            return Err(error);
        }

        self.latest_frame
            .lock()
            .map_err(|_| "Failed to read camera frame buffer".to_string())?
            .clone()
            .ok_or_else(|| "No camera frame is available yet".to_string())
    }

    pub fn shared_handle(&self) -> SharedCameraHandle {
        SharedCameraHandle {
            latest_frame: Arc::clone(&self.latest_frame),
            last_error: Arc::clone(&self.last_error),
            frame_revision: Arc::clone(&self.frame_revision),
        }
    }

    pub fn capture_now(&self) -> Result<CapturedFrame, String> {
        let start_revision = self.frame_revision.load(Ordering::Relaxed);
        let min_revision = start_revision.saturating_add(1);
        let earliest_capture = Instant::now() + Duration::from_millis(250);
        let deadline = Instant::now() + Duration::from_secs(3);

        while Instant::now() < deadline {
            if let Some(error) = self
                .last_error
                .lock()
                .map_err(|_| "Failed to read camera error state".to_string())?
                .clone()
            {
                return Err(error);
            }

            let current_revision = self.frame_revision.load(Ordering::Relaxed);
            if Instant::now() >= earliest_capture && current_revision >= min_revision {
                return self.latest_frame();
            }

            thread::sleep(Duration::from_millis(30));
        }

        self.latest_frame()
    }
}

impl Drop for CameraWorker {
    fn drop(&mut self) {
        self.stop_flag.store(true, Ordering::Relaxed);
        if let Some(join_handle) = self.join_handle.take() {
            let _ = join_handle.join();
        }
    }
}

fn camera_worker_loop(
    camera: CameraOption,
    latest_frame: Arc<Mutex<Option<CapturedFrame>>>,
    last_error: Arc<Mutex<Option<String>>>,
    frame_revision: Arc<AtomicU64>,
    stop_flag: Arc<AtomicBool>,
) {
    if let Err(error) = camera_stream_loop(camera, latest_frame, frame_revision, stop_flag) {
        if let Ok(mut error_guard) = last_error.lock() {
            *error_guard = Some(error);
        }
    }
}

#[cfg(target_os = "macos")]
pub fn list_cameras() -> Result<Vec<CameraOption>, String> {
    use std::process::Command;

    let output = Command::new("ffmpeg")
        .args(["-f", "avfoundation", "-list_devices", "true", "-i", ""])
        .output()
        .map_err(|error| format!("Failed to run ffmpeg for camera discovery: {error}"))?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut cameras = Vec::new();

    for line in stderr.lines() {
        let Some(marker_start) = line.find("] [") else {
            continue;
        };

        let suffix = &line[marker_start + 3..];
        let Some(index_end) = suffix.find(']') else {
            continue;
        };

        let index_str = &suffix[..index_end];
        let Ok(index) = index_str.parse::<u32>() else {
            continue;
        };

        let label = suffix[index_end + 1..].trim();
        if label.is_empty() {
            continue;
        }

        cameras.push(CameraOption {
            index,
            device_id: index.to_string(),
            label: label.to_string(),
        });
    }

    if cameras.is_empty() {
        return Err(
            "No cameras found. Check macOS camera permissions for Terminal and verify ffmpeg can see your devices."
                .to_string(),
        );
    }

    Ok(cameras)
}

#[cfg(target_os = "macos")]
fn looks_like_jpeg(bytes: &[u8]) -> bool {
    bytes.len() > 4
        && bytes[0] == 0xFF
        && bytes[1] == 0xD8
        && bytes[bytes.len() - 2] == 0xFF
        && bytes[bytes.len() - 1] == 0xD9
}

#[cfg(target_os = "macos")]
fn camera_stream_loop(
    camera: CameraOption,
    latest_frame: Arc<Mutex<Option<CapturedFrame>>>,
    frame_revision: Arc<AtomicU64>,
    stop_flag: Arc<AtomicBool>,
) -> Result<(), String> {
    use std::fs;
    use std::process::{Command, Stdio};
    use std::time::SystemTime;

    let mut frame_path = std::env::temp_dir();
    frame_path.push(format!("self-checkout-camera-{}.jpg", camera.index));

    let input = format!("{}:none", camera.index);
    let mut child = Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "avfoundation",
            "-framerate",
            "15",
            "-i",
            &input,
            "-vf",
            "fps=5",
            "-q:v",
            "4",
            "-update",
            "1",
            "-y",
        ])
        .arg(frame_path.as_os_str())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| {
            format!(
                "Failed to start camera stream for {}: {error}",
                camera.label
            )
        })?;

    let mut last_modified: Option<SystemTime> = None;

    while !stop_flag.load(Ordering::Relaxed) {
        if let Ok(metadata) = fs::metadata(&frame_path) {
            let modified = metadata.modified().ok();
            if modified.is_some() && modified != last_modified {
                let frame_bytes = fs::read(&frame_path).map_err(|error| {
                    format!("Failed to read camera frame from {}: {error}", camera.label)
                })?;

                if looks_like_jpeg(&frame_bytes) {
                    if let Ok(mut frame_guard) = latest_frame.lock() {
                        *frame_guard = Some(CapturedFrame {
                            bytes: frame_bytes,
                            content_type: "image/jpeg",
                            file_name: "snapshot.jpg",
                        });
                    }
                    frame_revision.fetch_add(1, Ordering::Relaxed);
                    last_modified = modified;
                }
            }
        }

        if let Some(status) = child.try_wait().map_err(|error| {
            format!(
                "Failed to monitor camera stream for {}: {error}",
                camera.label
            )
        })? {
            let _ = fs::remove_file(&frame_path);
            return Err(format!(
                "Camera stream for {} stopped unexpectedly with status {}",
                camera.label, status
            ));
        }

        thread::sleep(Duration::from_millis(80));
    }

    let _ = child.kill();
    let _ = child.wait();
    let _ = fs::remove_file(&frame_path);
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn list_cameras() -> Result<Vec<CameraOption>, String> {
    use nokhwa::{native_api_backend, nokhwa_check, query};

    if !nokhwa_check() {
        return Err("Camera permission is not granted".to_string());
    }

    let backend = native_api_backend()
        .ok_or_else(|| "No native camera backend is available on this platform".to_string())?;
    let devices = query(backend).map_err(|error| format!("Failed to query cameras: {error}"))?;

    Ok(devices
        .into_iter()
        .enumerate()
        .map(|(index, info)| CameraOption {
            index: info.index().as_index().unwrap_or(index as u32),
            device_id: info.misc(),
            label: format!("{} ({})", info.human_name(), index + 1),
        })
        .collect())
}

#[cfg(not(target_os = "macos"))]
fn camera_stream_loop(
    camera: CameraOption,
    latest_frame: Arc<Mutex<Option<CapturedFrame>>>,
    frame_revision: Arc<AtomicU64>,
    stop_flag: Arc<AtomicBool>,
) -> Result<(), String> {
    use image::{DynamicImage, ImageFormat};
    use nokhwa::pixel_format::RgbFormat;
    use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
    use nokhwa::{Camera, nokhwa_check};
    use std::io::Cursor;

    if !nokhwa_check() {
        return Err("Camera permission is not granted".to_string());
    }

    let requested =
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
    let camera_index = if camera.device_id.is_empty() {
        CameraIndex::Index(camera.index)
    } else {
        CameraIndex::String(camera.device_id.clone())
    };

    let mut device = Camera::new(camera_index, requested)
        .map_err(|error| format!("Failed to open camera {}: {error}", camera.label))?;

    device
        .open_stream()
        .map_err(|error| format!("Failed to start camera stream: {error}"))?;

    while !stop_flag.load(Ordering::Relaxed) {
        let frame = device
            .frame()
            .map_err(|error| format!("Failed to capture camera frame: {error}"))?;

        let image = frame
            .decode_image::<RgbFormat>()
            .map_err(|error| format!("Failed to decode camera frame: {error}"))?;

        let mut png_bytes = Cursor::new(Vec::new());
        DynamicImage::ImageRgb8(image)
            .write_to(&mut png_bytes, ImageFormat::Png)
            .map_err(|error| format!("Failed to encode camera frame: {error}"))?;

        if let Ok(mut frame_guard) = latest_frame.lock() {
            *frame_guard = Some(CapturedFrame {
                bytes: png_bytes.into_inner(),
                content_type: "image/png",
                file_name: "snapshot.png",
            });
        }
        frame_revision.fetch_add(1, Ordering::Relaxed);
    }

    Ok(())
}
