use iced::widget::image;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    #[serde(deserialize_with = "deserialize_price")]
    pub price: f64,
    pub unit: String,
    pub image_url: Option<String>,
}

pub type ProductImage = image::Handle;

#[derive(Debug, Deserialize)]
pub struct ProductsResponse {
    pub data: Vec<Product>,
}

fn deserialize_price<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum PriceValue {
        Number(f64),
        String(String),
    }

    match PriceValue::deserialize(deserializer)? {
        PriceValue::Number(value) => Ok(value),
        PriceValue::String(value) => value.parse().map_err(serde::de::Error::custom),
    }
}
