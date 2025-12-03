# Copilot Instructions

## Rust style

- Use `mod.rs` for imports and exports only. Give structs anbd functions their own files.
- Include or update unit tests when behavior changes; run `cargo test` before submitting.
- Prefer descriptive error types (use `thiserror` or `anyhow` only where consistent with the repo).
