use crate::MlMode;
use crate::camera::CameraOption;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const SETTINGS_FILE: &str = ".self-checkout-settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedSettings {
    #[serde(default = "default_mode")]
    pub ml_mode: String,
    #[serde(default)]
    pub shelf_camera_device_id: Option<String>,
    #[serde(default)]
    pub scale_camera_device_id: Option<String>,
}

fn default_mode() -> String {
    "off".to_string()
}

impl Default for PersistedSettings {
    fn default() -> Self {
        Self {
            ml_mode: default_mode(),
            shelf_camera_device_id: None,
            scale_camera_device_id: None,
        }
    }
}

impl PersistedSettings {
    pub fn load() -> Self {
        let path = settings_path();
        fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let path = settings_path();
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, json);
        }
    }

    pub fn ml_mode_enum(&self) -> MlMode {
        match self.ml_mode.as_str() {
            "on" => MlMode::On,
            "label" => MlMode::Label,
            _ => MlMode::Off,
        }
    }

    pub fn from_state(
        ml_mode: MlMode,
        shelf_camera: Option<&CameraOption>,
        scale_camera: Option<&CameraOption>,
    ) -> Self {
        Self {
            ml_mode: match ml_mode {
                MlMode::Off => "off",
                MlMode::On => "on",
                MlMode::Label => "label",
            }
            .to_string(),
            shelf_camera_device_id: shelf_camera.map(|c| c.device_id.clone()),
            scale_camera_device_id: scale_camera.map(|c| c.device_id.clone()),
        }
    }

    pub fn find_shelf_camera<'a>(&self, cameras: &'a [CameraOption]) -> Option<&'a CameraOption> {
        self.shelf_camera_device_id
            .as_ref()
            .and_then(|id| cameras.iter().find(|c| &c.device_id == id))
    }

    pub fn find_scale_camera<'a>(&self, cameras: &'a [CameraOption]) -> Option<&'a CameraOption> {
        self.scale_camera_device_id
            .as_ref()
            .and_then(|id| cameras.iter().find(|c| &c.device_id == id))
    }
}

fn settings_path() -> PathBuf {
    PathBuf::from(SETTINGS_FILE)
}
