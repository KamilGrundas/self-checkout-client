# Client repository instructions

Rust 2024/Iced kiosk application. Source code is under `src/`, translations under `assets/lang`, UI assets under `assets/ui`, and dependency resolution is recorded in `Cargo.lock`.

Read `../AGENTS.md` first. Inspect Git with `git -C self-checkout-client`; never edit directly on `main` or `master`, combine repositories in one commit, or commit `.env`, `.self-checkout-client-id`, `target`, credentials, camera captures, or machine-specific IDE state. Keep Polish and English translation keys aligned.

Checks are `cargo fmt --check`, `cargo check`, `cargo test`, and, when configured for the target, `cargo clippy --all-targets --all-features -- -D warnings`. Hardware camera and kiosk-display behavior requires an explicit device test and must not be inferred from compilation alone.

Run validation on remote dev via `../ops/dev-sync.sh --repo client --dry-run` and `../ops/dev-test.sh --repo client`. Keep commits focused and imperative; coordinate API contract changes with backend and ML PRs.

The base branch is `main` as recorded in `../repos.yaml`. Create short-lived branches from a freshly fetched `origin/main`, and never implement directly on `main` or `master`. Use Conventional Commits with scopes such as `client`, `ui`, `checkout`, `camera`, `settings`, or `websocket`.

Definition of Done: rustfmt, `cargo check`, and `cargo test` pass on remote dev; Clippy is run when the target supports it; the integrated healthcheck passes; Polish and English translations remain aligned; hardware-dependent behavior is explicitly marked as tested or not tested; and API/configuration impact and rollback are documented.
