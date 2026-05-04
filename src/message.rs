use crate::camera::{CameraOption, CapturedFrame};
use crate::checkout::CheckoutSession;
use crate::product::{Product, ProductImage};
use crate::ws::WsEvent;

/// Decoded camera preview frames ready for display.
#[derive(Debug, Clone)]
pub struct PreviewFrames {
    pub shelf: Option<Result<iced::widget::image::Handle, String>>,
    pub scale: Option<Result<iced::widget::image::Handle, String>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    StartPressed,
    ToggleSettings,
    SettingsModeSelected(crate::MlMode),
    CameraPreviewTick(PreviewFrames),
    RetryConnectionPressed,
    HelpPressed,
    LanguagePressed,
    CameraSelected(CameraOption),
    ClearShelfCamera,
    ScaleCameraSelected(CameraOption),
    ClearScaleCamera,
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
    ConnectionFinished(
        Result<(Vec<Product>, Vec<crate::product::Category>, CheckoutSession), String>,
    ),
    RecoveryFinished(
        Result<(Vec<Product>, Vec<crate::product::Category>, CheckoutSession), String>,
    ),
    CartSynced(Result<CheckoutSession, String>),
    PayPressed,
    PaymentMethodSelected,
    PaymentMethodBackPressed,
    PaymentTerminalDecision(bool),
    PaymentSuccessTimeout,
    SearchProductPressed,
    ProductPageChanged(usize),
    ScaleCameraFrameReady(Result<CapturedFrame, String>),
    ClassifyFinished(Result<Vec<(String, f64)>, String>),
    SettingsPushed(Result<(), String>),
    WsEvent(WsEvent),
}
