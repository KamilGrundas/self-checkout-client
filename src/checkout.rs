use crate::product::Product;
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

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct CheckoutSession {
    pub id: String,
    pub counter_id: String,
    pub client_id: String,
    pub closed: bool,
    pub payment_status: String,
    pub cart: Vec<CartItem>,
}

#[derive(Debug, Serialize)]
pub struct ConnectPayload {
    pub counter_id: String,
    pub password: String,
    pub client_id: String,
}

#[derive(Debug, Serialize)]
pub struct SyncCartPayload {
    pub counter_id: String,
    pub password: String,
    pub client_id: String,
    pub cart: Vec<CartItem>,
}
