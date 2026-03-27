use crate::checkout::CheckoutSession;
use crate::product::{Product, ProductImage};

#[derive(Debug, Clone)]
pub enum Message {
    StartPressed,
    RetryConnectionPressed,
    ProductSelected(String),
    CategorySelected(String),
    QuantityChanged(String),
    KeypadPressed(char),
    KeypadClear,
    KeypadBackspace,
    ConfirmAddToCart,
    CancelAddToCart,
    ProductImageLoaded {
        product_id: String,
        result: Result<ProductImage, String>,
    },
    WeightMeasured(f64),
    ConnectionFinished(
        Result<(Vec<Product>, Vec<crate::product::Category>, CheckoutSession), String>,
    ),
    RecoveryFinished(
        Result<(Vec<Product>, Vec<crate::product::Category>, CheckoutSession), String>,
    ),
    CartSynced(Result<CheckoutSession, String>),
    PayPressed,
    PaymentFinished(Result<(Vec<Product>, Vec<crate::product::Category>, CheckoutSession), String>),
}
