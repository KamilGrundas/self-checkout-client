# Client repository instructions

Rust/Iced source is under `src/`, translations under `assets/lang`, and
assets under `assets/ui`. Read applicable parent instructions before editing;
this repository remains independent of host paths, remote SSH aliases, domains,
container runtimes, and provider brands.

Preserve existing changes and work on `main`. Do not create task branches or
pull requests in the standard workflow. Commits, pushes, device synchronization,
or deployment require separate direct approval. Do not commit local environment
files, device IDs, credentials, captures, or build output.

Run `cargo fmt --check`, `cargo check`, `cargo test`, and configured Clippy
checks. Device camera, touch, display, and certificate-trust validation is
separate observed work. Configure browser-accessible API URLs through local
device configuration; do not use internal Compose service names or weaken TLS.
