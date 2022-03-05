# Release process

- Update `CHANGELOG` with date and version
- Update `Cargo.toml` with version
- Update `README.md` and other images to point to github raw content at commit SHA1 of current HEAD
- `cargo fmt --all`
- `cargo build`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test`
- `cargo docs --no-deps`
- `cargo +nightly build` (for `docs.rs`)
