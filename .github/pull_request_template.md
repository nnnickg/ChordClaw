## Scope

What changed:

## Checks

- [ ] `cargo fmt --all --check`
- [ ] `cargo test --workspace --locked`
- [ ] `cargo test -p chordclaw-wasm --no-default-features --features identify --locked`
- [ ] `cargo test -p chordclaw-wasm --all-features --locked`
- [ ] `cargo clippy --workspace --all-targets --locked -- -D warnings`
- [ ] No generated build output, local caches, credentials, or personal config files are included
- [ ] Security-sensitive changes are called out explicitly
