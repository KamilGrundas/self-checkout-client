use crate::camera::CameraOption;
use crate::product::{Category, Product};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartItem {
    pub product_id: String,
    pub name: String,
    pub unit: String,
    pub price: f64,
    pub quantity: f64,
    pub quantity_label: String,
    pub line_total: f64,
    pub image_url: Option<String>,
}

impl CartItem {
    pub fn from_product(product: &Product, quantity: f64) -> Self {
        let quantity_label = if product.unit == "kg" {
            format!("{quantity:.2} kg")
        } else {
            format!("{} szt", quantity as u32)
        };

        Self {
            product_id: product.id.clone(),
            name: product.name.clone(),
            unit: product.unit.clone(),
            price: product.price,
            quantity,
            quantity_label,
            line_total: quantity * product.price,
            image_url: product.image_url.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct CounterSettings {
    #[serde(default = "default_ml_mode")]
    pub ml_mode: String,
    #[serde(default)]
    pub shelf_camera_device_id: Option<String>,
    #[serde(default)]
    pub scale_camera_device_id: Option<String>,
    #[serde(default = "default_language")]
    pub language: String,
}

fn default_ml_mode() -> String {
    "off".to_string()
}

fn default_language() -> String {
    "pl".to_string()
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct CheckoutSession {
    pub id: String,
    pub closed: bool,
    pub payment_status: String,
    pub cart: Vec<CartItem>,
    #[serde(default)]
    pub counter_settings: CounterSettings,
}

#[derive(Debug, Clone)]
pub struct ConnectionData {
    pub products: Vec<Product>,
    pub categories: Vec<Category>,
    pub checkout_session: CheckoutSession,
    pub cameras: Vec<CameraOption>,
    pub camera_error: String,
}

#[derive(Debug, Serialize)]
pub struct ConnectPayload {
    pub available_cameras: Vec<CameraOption>,
    pub camera_discovery_succeeded: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connect_payload_serializes_available_cameras() {
        let payload = ConnectPayload {
            available_cameras: vec![CameraOption {
                index: 2,
                device_id: "camera-device".to_string(),
                label: "Camera label".to_string(),
            }],
            camera_discovery_succeeded: true,
        };

        let json = serde_json::to_value(payload).expect("payload should serialize");
        assert_eq!(json["available_cameras"][0]["device_id"], "camera-device");
        assert_eq!(json["available_cameras"][0]["label"], "Camera label");
        assert_eq!(json["available_cameras"][0]["index"], 2);
        assert_eq!(json["camera_discovery_succeeded"], true);
    }
}

#[derive(Debug, Serialize)]
pub struct SyncCartPayload {
    pub cart: Vec<CartItem>,
}

#[derive(Debug, Serialize)]
pub struct PaymentPayload {}
