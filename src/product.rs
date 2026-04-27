use iced::widget::image;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    #[serde(deserialize_with = "deserialize_price")]
    pub price: f64,
    pub unit: String,
    #[allow(dead_code)]
    pub category_name: String,
    pub category_key: String,
    pub image_url: Option<String>,
    pub thumbnail_url: Option<String>,
}

pub type ProductImage = image::Handle;

#[derive(Debug, Clone, Deserialize)]
pub struct Category {
    #[allow(dead_code)]
    pub id: String,
    pub name: String,
    pub key: String,
}

#[derive(Debug, Deserialize)]
pub struct ProductsResponse {
    pub data: Vec<Product>,
}

#[derive(Debug, Deserialize)]
pub struct CategoriesResponse {
    pub data: Vec<Category>,
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
