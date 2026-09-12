# Self-Checkout Client

Rust desktop client for the self-checkout flow, built with `iced`.

It is designed to work with the backend and infrastructure from the same project family:
- Backend: `https://github.com/KamilGrundas/self-checkout-backend`
- Infrastructure: `https://github.com/KamilGrundas/self-checkout-infra`
- ML service: `https://github.com/KamilGrundas/self-checkout-ml`

## Checkout Modes

The mode, language, shelf camera, and scale camera are stored with the checkout
counter in the backend. The admin selects cameras from the inventory reported
by the connected client. A new checkout session snapshots those settings, so
an admin edit made during an active session takes effect from the next session.
The admin configures those settings. The client only reports its detected
camera inventory and applies the settings snapshot returned with a session.

**ML_off** — no camera or ML service required.
The operator places a product on the scale, selects it from the list manually, and removes it. Basic checkout flow.

**ML_label** — requires at least one camera and a running ML service.
The operator places a product on the scale; the scale camera takes a snapshot. After the operator moves the product to the drop zone, the shelf camera takes a snapshot of the drop zone. The captured images are uploaded to the ML service for later labeling and training.

**ML_on** — requires at least one camera and a running ML service.
The scale camera identifies the product and suggests it at the top of the list. After dropping the product, the shelf camera verifies that the items in the drop zone match the cart.

`ML_on` and `ML_label` are disabled in settings until at least one camera is selected.

## What It Does

- connects to the backend health check and checkout session API,
- restores the unfinished session for the Counter bound to its API key,
- fetches and displays products,
- syncs the cart with the backend,
- closes the checkout session after payment,
- blocks local cart changes when backend synchronization fails,
- discovers cameras on connection and reports the inventory to the backend and
  admin over HTTP/WebSocket,
- applies the settings snapshot returned for the current checkout session,
- uploads shelf snapshots (drop zone camera) to `POST /api/v1/checkout-sessions/{id}/shelf-snapshots`,
- uploads scale snapshots (scale camera) to `POST /api/v1/checkout-sessions/{id}/scale-snapshots`.

## Requirements

- Rust stable
- running backend API
- optional running ML API for `ML_label`
- configured checkout counter in the backend
- on macOS, FFmpeg with AVFoundation camera access

## Configuration

The client reads configuration from `.env` or environment variables:

```env
DEFAULT_LANG=pl
APP_ENV=dev
API_BASE_URL=http://localhost:8000
CHECKOUT_API_KEY=put-counter-api-key-here
ML_API_BASE_URL=http://localhost:8001
HIDE_CURSOR=false
```

HTTPS and WSS connections use the operating system's trusted root
certificates. The selected local environment owns the corresponding trust
configuration; do not weaken TLS verification in the client.

The checkout-session connection is derived from `API_BASE_URL` as
`/api/v1/ws/checkout-session` and authenticates with `X-API-Key`; it does not
put credentials in the URL. When `API_BASE_URL` is behind Nginx Proxy Manager,
enable **Websockets Support** on that API Proxy Host. Without it, HTTP API calls
may succeed but the client receives `404 Not Found` when opening the session
WebSocket.

`FFMPEG_PATH` is optional on macOS. The client first checks this override, then
the process `PATH`, `/opt/homebrew/bin/ffmpeg`, and
`/usr/local/bin/ffmpeg`. This keeps camera discovery working when `cargo run`
is launched from an IDE or another environment with a reduced `PATH`.

`APP_ENV` controls the window mode:
- `dev` (default) -> windowed mode
- `prod` or `production` -> fullscreen mode

`HIDE_CURSOR` explicitly controls pointer visibility. When it is omitted, the
cursor is hidden in `prod`/`production` and visible in local development.

## Run

```bash
cargo run
```

## Check Build

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo check
cargo test
```

## Optional target-computer validation

The client can run on a separately managed optional device. Access is optional;
local implementation and Cargo checks continue when the device is unavailable.

Device synchronization and deployment are local operator procedures, not
workspace scripts. They must preserve device-owned configuration and never
grant commit, push, release, or production authority.

Device `.env`, build outputs and startup configuration are owned by the local
operator. Target API endpoints must lead exclusively to the selected local
environment. The only kiosk credential is the Counter-bound
`CHECKOUT_API_KEY`; it has no locally persisted identity or settings.

The target profile and hardware-validation checklist are local operator
procedures. Remote process checks do not replace direct confirmation of
display, touch, camera, and scale behavior.

## Runtime Notes

- The client retries backend connection 3 times with a 3-second delay.
- If the backend is still unavailable, it shows a manual reconnect action.
- If the backend fails during an active session, the client shows a reconnect overlay and reloads session state from the backend before allowing further changes.
- After the backend connection succeeds, the client goes directly to the
  Welcome screen and applies the current session's backend settings snapshot.
- An administrator selects the mode (`ML_off`, `ML_on`, `ML_label`) and shelf/
  scale cameras. Changes apply when the current session closes and the client
  reconnects to a new session. If a selected camera is unavailable, the session
  falls back to `ML_off`.
- Camera workers start without blocking the UI once the session settings select
  a camera.
- A failed camera discovery report never clears the backend's last successful
  camera inventory.
- `ML_off` runs the checkout flow without camera or ML integration.
- In `ML_label`, the scale camera uploads scale snapshots; the shelf camera uploads shelf snapshots.
- In `ML_label`, `Tap to start` uploads the first baseline image of the empty drop zone.
- In `ML_label`, each next shelf snapshot is uploaded only after the placement modal is shown and the operator clicks `Ready`.
- In `ML_on`, the scale camera identifies the product and suggests it at the top of the list.
- `CHECKOUT_API_KEY` identifies one checkout counter. The backend restores its
  one unfinished session directly from that key; no local device-ID file is used.
- Successful payment is confirmed through the backend before the local cart is
  cleared. The backend closes the old session and disconnects its client
  WebSocket so the next session receives the latest counter settings.
