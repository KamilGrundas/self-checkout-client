use crate::camera::CapturedFrame;
use crate::checkout::{CheckoutSession, ConnectionData};
use crate::product::ProductImage;
use crate::ws::WsEvent;

#[derive(Debug, Clone)]
pub enum Message {
    StartPressed,
    RetryConnectionPressed,
    HelpPressed,
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
    ShelfSnapshotUploaded {
        session_id: String,
        capture_index: usize,
        result: Result<(), String>,
    },
    ScaleSnapshotUploaded {
        session_id: String,
        capture_index: usize,
        result: Result<(), String>,
    },
    ShelfPlacementReady,
    ShelfPlacementConfirmed,
    WeightMeasured(f64),
    ConnectionFinished(Result<ConnectionData, String>),
    RecoveryFinished(Result<ConnectionData, String>),
    CartSynced(Result<CheckoutSession, String>),
    PayPressed,
    PaymentMethodSelected,
    PaymentMethodBackPressed,
    PaymentTerminalDecision(bool),
    PaymentCompleted(Result<CheckoutSession, String>),
    PaymentSuccessTimeout,
    SearchProductPressed,
    ProductPageChanged(usize),
    ScaleCameraFrameReady(Result<CapturedFrame, String>),
    ClassifyFinished(Result<Vec<(String, f64)>, String>),
    WsEvent(WsEvent),
}
