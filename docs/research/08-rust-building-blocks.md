# Rust building blocks

Queried via the GitHub API and crates.io on 2026-09-14.

## Crates

| Crate | Purpose | Version | Updated | Downloads | License |
|---|---|---|---|---|---|
| `typst-syntax` | Typst parser | 0.15.1 | 2026-07-17 | 2.8M | Apache-2.0 |
| `rusqlite` | SQLite | 0.40.2 | 2026-08-08 | 106M | MIT |
| `tower-lsp-server` | LSP framework (community fork of tower-lsp) | 0.23.0 | 2026-09-11 | 2.1M | Apache-2.0 |
| `lsp-server` | LSP framework (rust-analyzer, synchronous) | 0.10.0 | 2026-07-16 | 15.7M | MIT/Apache-2.0 |
| `pdf_oxide` | PDF extraction, pure Rust | 0.3.78 | 2026-09-08 | 0.9M | MIT OR Apache-2.0 |
| `pdf-extract` | PDF text, pure Rust | 0.12.0 | 2026-06-25 | 5.0M | MIT |
| `lopdf` | PDF object model | 0.45.0 | 2026-09-08 | 19.3M | MIT |
| `hayro` | PDF rendering, pure Rust | 0.7.1 | 2026-06-05 | 2.2M | Apache-2.0 OR MIT |
| `pdfium-render` | binding to pdfium | 0.9.4 | 2026-09-06 | 2.3M | MIT OR Apache-2.0 |
| `mupdf` (mupdf-rs) | binding to MuPDF | 0.8.0 | 2026-06-22 | 1.6M | **AGPL-3.0** |

## Repos

| Repo | Last push | Stars | License |
|---|---|---|---|
| `typst/typst` | 2026-09-14 | 56,012 | Apache-2.0 |
| `Myriad-Dreamin/tinymist` | 2026-09-14 | 3,532 | Apache-2.0 |
| `tower-lsp-community/tower-lsp-server` | 2026-09-11 | 222 | Apache-2.0 |
| `openlawlibrary/pygls` (Python LSP, for comparison) | 2026-09-14 | 808 | Apache-2.0 |
| `asg017/sqlite-vec` (vector search in SQLite) | 2026-05-18 | 8,107 | Apache-2.0 |
| `anush008/fastembed-rs` (local embeddings) | 2026-09-12 | 1,009 | Apache-2.0 |
| `messense/mupdf-rs` | 2026-09-14 | 206 | AGPL-3.0 |
| `ajrcarey/pdfium-render` | 2026-08-16 | 705 | (GitHub: NOASSERTION; crates.io: MIT OR Apache-2.0) |
| `uben0/tree-sitter-typst` | 2025-04-02 | 193 | MIT |
| `zotero/zotero` | 2026-09-14 | 15,272 | AGPL-3.0 (GitHub: NOASSERTION) |

## Existing toolchains (measured)

- `cargo`/`rustc` 1.95.0, `python3` 3.13.7, `uv` 0.9.6, `node` v24.14.1
- `clang` + `libclang-18`, `pkg-config`, `make`
- `fontconfig` development files are **missing** (relevant for mupdf-rs with default features)
- `mutool` 1.25.1 installed (without ICC support)
- `pdftotext` installed

## Language choice assessment (result)

- No building block for the first version needs Python: Zotero (HTTP), the Typst parser, LSP, SQLite, PDF extraction are all available in Rust.
- Python only becomes interesting later for ML parts (and even there, `fastembed-rs` exists); then as an optional side process.
- Decision: **Rust**, a single program `bib` (see `12`).
