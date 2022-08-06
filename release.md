# Release process

- Update `CHANGELOG` with date and version
- Update `Cargo.toml` with version
- Update `README.md` and other images to point to github raw content at commit SHA1 of current HEAD
- Update other documentation links to point to the new Bevy release (if any) on `docs.rs`
- `cargo fmt --all`
- `cargo test --no-default-features`
- `cargo test --all-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo doc --no-deps`
- `cargo +nightly build --all-features` (for `docs.rs`)
