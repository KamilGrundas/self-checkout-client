# Self-Checkout Client

Rust desktop client for the self-checkout flow, built with `iced`.

It is designed to work with the backend and infrastructure from the same project family:
- Backend: `https://github.com/KamilGrundas/self-checkout-backend`
- Infrastructure: `https://github.com/KamilGrundas/self-checkout-infra`

## What It Does

- connects to the backend health check and checkout session API,
- restores an unfinished session for the same device,
- fetches and displays products,
- syncs the cart with the backend,
- closes the checkout session after payment,
- blocks local cart changes when backend synchronization fails.

## Requirements

- Rust stable
- running backend API
- configured checkout counter in the backend

## Configuration

The client reads configuration from `.env` or environment variables:

```env
DEFAULT_LANG=pl
API_BASE_URL=http://127.0.0.1:8000
CHECKOUT_COUNTER_ID=put-counter-id-here
CHECKOUT_COUNTER_PASSWORD=put-counter-password-here
CLIENT_ID_STORAGE_PATH=.self-checkout-client-id
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
