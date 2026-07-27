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
The client also keeps its keyboard-triggered settings overlay (`o` on the
Welcome screen); changes made there update the same backend counter settings.

**ML_off** — no camera or ML service required.
The operator places a product on the scale, selects it from the list manually, and removes it. Basic checkout flow.

**ML_label** — requires at least one camera and a running ML service.
The operator places a product on the scale; the scale camera takes a snapshot. After the operator moves the product to the drop zone, the shelf camera takes a snapshot of the drop zone. The captured images are uploaded to the ML service for later labeling and training.

**ML_on** — requires at least one camera and a running ML service.
The scale camera identifies the product and suggests it at the top of the list. After dropping the product, the shelf camera verifies that the items in the drop zone match the cart.

`ML_on` and `ML_label` are disabled in settings until at least one camera is selected.

## What It Does

- connects to the backend health check and checkout session API,
- restores an unfinished session for the same device,
- fetches and displays products,
- syncs the cart with the backend,
- closes the checkout session after payment,
- blocks local cart changes when backend synchronization fails,
- discovers cameras on connection and reports the inventory to the backend and
  admin over HTTP/WebSocket,
- applies the settings snapshot returned for the current checkout session,
- provides a keyboard-triggered settings overlay (press `o` on the Welcome screen) for mode and camera configuration,
- shows live camera preview in the settings overlay,
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
API_BASE_URL=http://127.0.0.1:8000
CHECKOUT_COUNTER_ID=put-counter-id-here
CHECKOUT_COUNTER_PASSWORD=put-counter-password-here
CLIENT_ID_STORAGE_PATH=.self-checkout-client-id
ML_API_BASE_URL=http://127.0.0.1:8001
FFMPEG_PATH=/opt/homebrew/bin/ffmpeg
```

`FFMPEG_PATH` is optional on macOS. The client first checks this override, then
the process `PATH`, `/opt/homebrew/bin/ffmpeg`, and
`/usr/local/bin/ffmpeg`. This keeps camera discovery working when `cargo run`
is launched from an IDE or another environment with a reduced `PATH`.

`APP_ENV` controls the window mode:
- `dev` (default) -> windowed mode
- `prod` or `production` -> fullscreen mode

## Run

```bash
cargo run
```

## Check Build

```bash
cargo check
```

## Runtime Notes

- The client retries backend connection 3 times with a 3-second delay.
- If the backend is still unavailable, it shows a manual reconnect action.
- If the backend fails during an active session, the client shows a reconnect overlay and reloads session state from the backend before allowing further changes.
- After the backend connection succeeds, the client goes directly to the
  Welcome screen and applies the current session's backend settings snapshot.
- Press `o` on the Welcome screen to open the settings overlay where you can select the mode (`ML_off`, `ML_on`, `ML_label`), choose shelf/scale cameras with live preview, and clear camera selection.
- Settings changed through the client are written to the backend. Admin changes
  are applied when the current session closes and the client reconnects to a
  new session. If a selected camera is unavailable, the session falls back to
  `ML_off`.
- Camera workers start without blocking the UI. The settings overlay shows a live camera preview at ~5 fps with resized thumbnails for smooth rendering.
- A failed camera discovery report never clears the backend's last successful
  camera inventory.
- `ML_off` runs the checkout flow without camera or ML integration.
- In `ML_label`, the scale camera uploads scale snapshots; the shelf camera uploads shelf snapshots.
- In `ML_label`, `Tap to start` uploads the first baseline image of the empty drop zone.
- In `ML_label`, each next shelf snapshot is uploaded only after the placement modal is shown and the operator clicks `Ready`.
- In `ML_on`, the scale camera identifies the product and suggests it at the top of the list.
- The client stores a local device identifier in `.self-checkout-client-id` so the backend can restore an unfinished session.
- Successful payment is confirmed through the backend before the local cart is
  cleared. The backend closes the old session and disconnects its client
  WebSocket so the next session receives the latest counter settings.
