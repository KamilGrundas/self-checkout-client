# Client repository instructions

Rust 2024/Iced kiosk application. Source code is under `src/`, translations under `assets/lang`, UI assets under `assets/ui`, and dependency resolution is recorded in `Cargo.lock`.

Read `../AGENTS.md` first. Inspect Git with `git -C self-checkout-client`; never edit directly on `main` or `master`, combine repositories in one commit, or commit `.env`, `.self-checkout-client-id`, `target`, credentials, camera captures, or machine-specific IDE state. Keep Polish and English translation keys aligned.

Checks are `cargo fmt --check`, `cargo check`, `cargo test`, and, when configured for the target, `cargo clippy --all-targets --all-features -- -D warnings`. Hardware camera and kiosk-display behavior requires an explicit device test and must not be inferred from compilation alone.

Run validation on remote dev via `../ops/dev-sync.sh --repo client --dry-run` and `../ops/dev-test.sh --repo client`. Keep commits focused and imperative; coordinate API contract changes with backend and ML PRs.

The optional, replaceable target computer is `ssh dev-client`. Local development remains
supported and device unavailability is a skipped optional step, not task
failure. After relevant client changes, run
`../ops/dev-client-check.sh --optional`; when reachable, inspect first, dry-run
and apply `../ops/dev-client-sync.sh`, then use
`../ops/dev-client-deploy.sh` and `../ops/dev-client-status.sh`. These scripts
synchronize only `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `src/`, and
`assets/`; they preserve `.env`, `.self-checkout-client-id`,
`.self-checkout-settings.json`, `target/`, and device startup configuration.
Uncommitted sources are allowed for device testing, but synchronization grants
no Git, release, or production authority.

Build natively with the target's cached, inspected toolchain and reuse its
actual startup mechanism. Verify the new process is the rebuilt binary and
check that both APIs resolve to services on `dev`. Read the device-owned target
profile first and refresh it only after the machine or its environment changes.
Do not assume filesystem paths, packages, service management, or graphical
runtime details. Do not claim visual, camera, touch, or scale behavior was
validated unless it was observed on the physical target.

The canonical development endpoints are `https://dev.api.teik.pl` and
`https://dev.ml.teik.pl`. Native HTTPS and WSS use the operating-system trust
store, so verify that the target trusts the Caddy development CA; do not weaken
TLS verification, add an application-private CA fallback, or replace these
URLs with raw IP addresses and published Compose ports.

Treat target authentication as part of deployment, not as manual setup. A
reachable target must have a dedicated checkout-counter record in the backend
on `dev`, named `dev-client` unless its non-secret target profile records
another explicit name. Verify the credentials with the checkout-session
connect endpoint. If the record is missing or stale, create or rotate it on
`dev` with `../ops/dev-client-authorize.sh`, which atomically updates only the
protected device-owned runtime configuration; never record the ID or password
in source, documentation, output, or the cached target profile.

The base branch is `main` as recorded in `../repos.yaml`. Create short-lived branches from a freshly fetched `origin/main`, and never implement directly on `main` or `master`. Use Conventional Commits with scopes such as `client`, `ui`, `checkout`, `camera`, `settings`, or `websocket`.

Definition of Done: rustfmt, `cargo check`, and `cargo test` pass locally and/or
on remote dev as supported; Clippy is run when the target supports it; the
integrated healthcheck passes; Polish and English translations remain aligned;
reachable `dev-client` deployments are rebuilt, restarted, hash-verified, and
authenticated against the backend on `dev`; the normal affected services on
`dev` and the target client work together after every relevant change;
unreachable device work is explicitly skipped; hardware-dependent behavior is
marked as tested or not tested; and API/configuration impact and rollback are
documented.
