use crate::camera::CameraOption;
use crate::checkout::CheckoutSession;
use crate::message::Message;
use futures_util::{SinkExt, StreamExt};
use iced::Subscription;
use iced::futures::Stream;
use iced::futures::channel::mpsc::Sender as IcedSender;
use iced::stream;
use serde::Deserialize;
use std::hash::{Hash, Hasher};
use std::pin::Pin;
use std::sync::OnceLock;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use tokio::time::{Instant, sleep, timeout};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::http::HeaderValue;
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;

fn tokio_runtime() -> &'static Runtime {
    static RT: OnceLock<Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .thread_name("self-checkout-ws")
            .build()
            .expect("failed to build tokio runtime for websocket")
    })
}

const HEARTBEAT_TIMEOUT_SECS: u64 = 12;
const RECONNECT_BASE_DELAY_SECS: u64 = 1;
const RECONNECT_MAX_DELAY_SECS: u64 = 10;

#[derive(Debug, Clone)]
pub struct WsConfig {
    pub api_base_url: String,
    pub api_key: String,
    pub available_cameras: Vec<CameraOption>,
}

impl Hash for WsConfig {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.api_base_url.hash(state);
        self.api_key.hash(state);
        self.available_cameras.hash(state);
    }
}

#[derive(Debug, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum WsEvent {
    Connected,
    Disconnected,
    SessionUpdated {
        session: CheckoutSession,
        admin_takeover: bool,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
enum ServerMessage {
    SessionState {
        session: CheckoutSession,
        #[serde(default)]
        admin_takeover: bool,
    },
    Ping,
    Error {},
}

pub fn subscription(config: WsConfig) -> Subscription<Message> {
    Subscription::run_with(config, build_stream)
}

fn build_stream(config: &WsConfig) -> Pin<Box<dyn Stream<Item = Message> + Send>> {
    let config = config.clone();
    Box::pin(stream::channel(
        32,
        move |mut output: IcedSender<Message>| async move {
            let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

            tokio_runtime().spawn(async move {
                run_connection_loop(config, tx).await;
            });

            while let Some(msg) = rx.recv().await {
                if output.send(msg).await.is_err() {
                    break;
                }
            }
        },
    ))
}

async fn run_connection_loop(config: WsConfig, tx: mpsc::UnboundedSender<Message>) {
    let request = match ws_request(&config) {
        Ok(request) => request,
        Err(error) => {
            eprintln!("[ws] invalid configuration: {error}");
            let _ = tx.send(Message::WsEvent(WsEvent::Disconnected));
            return;
        }
    };
    eprintln!("[ws] connecting to checkout session websocket");
    let mut delay = RECONNECT_BASE_DELAY_SECS;

    loop {
        match tokio_tungstenite::connect_async(request.clone()).await {
            Ok((stream, _response)) => {
                eprintln!("[ws] connected");
                delay = RECONNECT_BASE_DELAY_SECS;
                if tx.send(Message::WsEvent(WsEvent::Connected)).is_err() {
                    return;
                }
                let (available_cameras, camera_discovery_succeeded) =
                    match tokio::task::spawn_blocking(crate::camera::list_cameras).await {
                        Ok(Ok(cameras)) => (cameras, true),
                        Ok(Err(error)) => {
                            eprintln!(
                                "[ws] camera discovery failed: {error}; preserving {} previously discovered cameras",
                                config.available_cameras.len()
                            );
                            (config.available_cameras.clone(), false)
                        }
                        Err(error) => {
                            eprintln!(
                                "[ws] camera discovery task failed: {error}; preserving {} previously discovered cameras",
                                config.available_cameras.len()
                            );
                            (config.available_cameras.clone(), false)
                        }
                    };
                eprintln!(
                    "[ws] reporting {} available cameras",
                    available_cameras.len()
                );
                pump(stream, &tx, &available_cameras, camera_discovery_succeeded).await;
                eprintln!("[ws] pump exited, reconnecting in {delay}s");
                if tx.send(Message::WsEvent(WsEvent::Disconnected)).is_err() {
                    return;
                }
            }
            Err(err) => {
                eprintln!("[ws] connect failed: {err}, retrying in {delay}s");
                if tx.send(Message::WsEvent(WsEvent::Disconnected)).is_err() {
                    return;
                }
            }
        }

        sleep(Duration::from_secs(delay)).await;
        delay = (delay * 2).min(RECONNECT_MAX_DELAY_SECS);
    }
}

async fn pump<S>(
    stream: S,
    output: &mpsc::UnboundedSender<Message>,
    available_cameras: &[CameraOption],
    camera_discovery_succeeded: bool,
) where
    S: futures_util::Stream<Item = Result<WsMessage, tokio_tungstenite::tungstenite::Error>>
        + futures_util::Sink<WsMessage, Error = tokio_tungstenite::tungstenite::Error>
        + Unpin,
{
    let (mut sink, mut source) = stream.split();
    let mut last_seen = Instant::now();
    let camera_report = serde_json::json!({
        "type": "available_cameras",
        "available_cameras": available_cameras,
        "camera_discovery_succeeded": camera_discovery_succeeded,
    });
    if sink
        .send(WsMessage::Text(camera_report.to_string().into()))
        .await
        .is_err()
    {
        return;
    }

    loop {
        let recv = timeout(Duration::from_secs(2), source.next()).await;
        match recv {
            Ok(Some(Ok(WsMessage::Text(text)))) => {
                last_seen = Instant::now();
                match serde_json::from_str::<ServerMessage>(&text) {
                    Err(err) => {
                        eprintln!("[ws] failed to parse server message: {err}; raw={text}");
                    }
                    Ok(msg) => match msg {
                        ServerMessage::SessionState {
                            session,
                            admin_takeover,
                        } => {
                            eprintln!(
                                "[ws] session_state: items={}, admin_takeover={admin_takeover}",
                                session.cart.len()
                            );
                            if output
                                .send(Message::WsEvent(WsEvent::SessionUpdated {
                                    session,
                                    admin_takeover,
                                }))
                                .is_err()
                            {
                                return;
                            }
                        }
                        ServerMessage::Ping => {
                            let _ = sink.send(WsMessage::Text("pong".into())).await;
                        }
                        ServerMessage::Error {} => {}
                    },
                }
            }
            Ok(Some(Ok(WsMessage::Ping(payload)))) => {
                last_seen = Instant::now();
                let _ = sink.send(WsMessage::Pong(payload)).await;
            }
            Ok(Some(Ok(WsMessage::Pong(_)))) => {
                last_seen = Instant::now();
            }
            Ok(Some(Ok(WsMessage::Close(_)))) => break,
            Ok(Some(Ok(_))) => {
                last_seen = Instant::now();
            }
            Ok(Some(Err(_))) => break,
            Ok(None) => break,
            Err(_) => {
                if last_seen.elapsed() > Duration::from_secs(HEARTBEAT_TIMEOUT_SECS) {
                    let _ = sink.send(WsMessage::Close(None)).await;
                    break;
                }
            }
        }
    }
}

fn ws_request(
    config: &WsConfig,
) -> Result<tokio_tungstenite::tungstenite::http::Request<()>, String> {
    let base = config.api_base_url.trim_end_matches('/');
    let scheme_swapped = if let Some(rest) = base.strip_prefix("https://") {
        format!("wss://{rest}")
    } else if let Some(rest) = base.strip_prefix("http://") {
        format!("ws://{rest}")
    } else {
        base.to_string()
    };
    let api_v1 = if scheme_swapped.ends_with("/api/v1") {
        scheme_swapped
    } else {
        format!("{scheme_swapped}/api/v1")
    };
    let mut request = format!("{api_v1}/ws/checkout-session")
        .into_client_request()
        .map_err(|error| format!("invalid websocket URL: {error}"))?;
    let api_key = HeaderValue::try_from(config.api_key.as_str())
        .map_err(|error| format!("invalid checkout API key: {error}"))?;
    request.headers_mut().insert("X-API-Key", api_key);
    Ok(request)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn websocket_request_keeps_handshake_headers_and_adds_counter_key() {
        let request = ws_request(&WsConfig {
            api_base_url: "https://api.example.test".to_string(),
            api_key: "sck_test".to_string(),
            available_cameras: vec![],
        })
        .expect("request should be valid");

        assert_eq!(
            request.uri(),
            "wss://api.example.test/api/v1/ws/checkout-session"
        );
        assert!(request.headers().contains_key("Sec-WebSocket-Key"));
        assert_eq!(request.headers().get("X-API-Key").unwrap(), "sck_test");
    }
}
