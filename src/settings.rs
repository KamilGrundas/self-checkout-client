use crate::MlMode;
use crate::camera::CameraOption;
use crate::checkout::{CounterSettings, CounterSettingsUpdatePayload};

pub fn ml_mode_from_str(value: &str) -> MlMode {
    match value {
        "on" => MlMode::On,
        "label" => MlMode::Label,
        _ => MlMode::Off,
    }
}

pub fn ml_mode_to_str(mode: MlMode) -> &'static str {
    match mode {
        MlMode::Off => "off",
        MlMode::On => "on",
        MlMode::Label => "label",
    }
}

pub fn find_camera<'a>(
    device_id: Option<&str>,
    cameras: &'a [CameraOption],
) -> Option<&'a CameraOption> {
    let id = device_id?;
    cameras.iter().find(|c| c.device_id == id)
}

pub fn build_settings_payload(
    counter_id: String,
    password: String,
    settings: &CounterSettings,
) -> CounterSettingsUpdatePayload {
    CounterSettingsUpdatePayload {
        counter_id,
        password,
        ml_mode: settings.ml_mode.clone(),
        shelf_camera_device_id: settings.shelf_camera_device_id.clone(),
        scale_camera_device_id: settings.scale_camera_device_id.clone(),
        language: settings.language.clone(),
    }
}

pub fn settings_from_state(
    ml_mode: MlMode,
    shelf_camera: Option<&CameraOption>,
    scale_camera: Option<&CameraOption>,
    language: &str,
) -> CounterSettings {
    CounterSettings {
        ml_mode: ml_mode_to_str(ml_mode).to_string(),
        shelf_camera_device_id: shelf_camera.map(|c| c.device_id.clone()),
        scale_camera_device_id: scale_camera.map(|c| c.device_id.clone()),
        language: language.to_string(),
    }
}
