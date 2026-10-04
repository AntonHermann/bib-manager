# Project instructions

Superpowers process skills are active in this project. Use the personal
installation; do not vendor its skills or bootstrap into this repository.

## Project commands

Run commands from the repository root with Rust 1.95 or newer.

- Install dependencies: `./.agents/prepare`
- Unit tests (no external PDF fixtures): `cargo test --workspace --lib --bins`
- Lint/typecheck: `cargo clippy --workspace --all-targets -- -D warnings`

Extraction integration tests additionally require the ignored PDF fixtures
listed in `.agents/linked`. Delta links them from the primary checkout on
this machine when creating new checkouts; existing checkouts are not updated.
The coordinate comparison also needs `mutool` to exercise both backends.
See `bench/README.md` for corpus setup and optional PDFium requirements.
