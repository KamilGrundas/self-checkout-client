# Self-Checkout Client

Rust desktop client for the self-checkout flow, built with `iced`.

It is designed to work with the backend and infrastructure from the same project family:
- Backend: `https://github.com/KamilGrundas/self-checkout-backend`
- Infrastructure: `https://github.com/KamilGrundas/self-checkout-infra`
- ML service: `https://github.com/KamilGrundas/self-checkout-ml`

## Checkout Modes

The operator selects a mode at startup.

**ML_off** — no camera or ML service required.
The operator places a product on the scale, selects it from the list manually, and removes it. Basic checkout flow.

**ML_label** — requires two cameras and a running ML service.
The operator places a product on the scale; the scale camera takes a snapshot. After the operator moves the product to the drop zone, the shelf camera takes a snapshot of the drop zone. The captured images are uploaded to the ML service for later labeling and training.

**ML_on** — requires two cameras and a running ML service (currently visible in the UI but disabled).
The scale camera identifies the product and suggests it at the top of the list. After dropping the product, the shelf camera verifies that the items in the drop zone match the cart.

## What It Does

- connects to the backend health check and checkout session API,
- restores an unfinished session for the same device,
- fetches and displays products,
- syncs the cart with the backend,
- closes the checkout session after payment,
- blocks local cart changes when backend synchronization fails,
- supports startup mode selection for `ML_off` and `ML_label`,
- uploads shelf snapshots (drop zone camera) to `POST /api/v1/checkout-sessions/{id}/shelf-snapshots`,
- uploads scale snapshots (scale camera) to `POST /api/v1/checkout-sessions/{id}/scale-snapshots`.

## Requirements

- Rust stable
- running backend API
- optional running ML API for `ML_label`
- configured checkout counter in the backend

## Configuration

The client reads configuration from `.env` or environment variables:

```env
DEFAULT_LANG=pl
API_BASE_URL=http://127.0.0.1:8000
CHECKOUT_COUNTER_ID=put-counter-id-here
CHECKOUT_COUNTER_PASSWORD=put-counter-password-here
CLIENT_ID_STORAGE_PATH=.self-checkout-client-id
ML_API_BASE_URL=http://127.0.0.1:8001
```

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
- After the backend connection succeeds, the operator chooses a startup mode on a dedicated screen.
- `ML_off` runs the checkout flow without camera or ML integration.
- `ML_label` requires selecting a camera before entering the checkout flow.
- In `ML_label`, the scale camera uploads scale snapshots; the shelf camera uploads shelf snapshots.
- In `ML_label`, `Tap to start` uploads the first baseline image of the empty drop zone.
- In `ML_label`, each next shelf snapshot is uploaded only after the placement modal is shown and the operator clicks `Ready`.
- `ML_on` is visible in the UI but currently disabled.
- The client stores a local device identifier in `.self-checkout-client-id` so the backend can restore an unfinished session.
