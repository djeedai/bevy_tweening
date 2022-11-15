# Release process

- Update `CHANGELOG` with date and version
- Update `Cargo.toml` with version
- Update `README.md` and other images to point to github raw content at commit SHA1 of current HEAD
- Update other documentation links to point to the new Bevy release (if any) on `docs.rs`
- `cargo fmt --all`
- `cargo test --no-default-features`
- `cargo test --no-default-features --features="bevy_ui"`
- `cargo test --no-default-features --features="bevy_sprite"`
- `cargo test --no-default-features --features="bevy_text"`
- `cargo test --no-default-features --features="bevy_asset"`
- `cargo test --all-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo +nightly build --all-features` (for `docs.rs`)
- `cargo +nightly doc --all-features --no-deps`
