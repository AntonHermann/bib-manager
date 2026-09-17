# Rust-Bausteine

Abgefragt über GitHub-API und crates.io am 2026-09-14.

## Crates

| Crate | Zweck | Version | Aktualisiert | Downloads | Lizenz |
|---|---|---|---|---|---|
| `typst-syntax` | Typst-Parser | 0.15.1 | 2026-07-17 | 2,8 Mio. | Apache-2.0 |
| `rusqlite` | SQLite | 0.40.2 | 2026-08-08 | 106 Mio. | MIT |
| `tower-lsp-server` | LSP-Framework (Community-Fork von tower-lsp) | 0.23.0 | 2026-09-11 | 2,1 Mio. | Apache-2.0 |
| `lsp-server` | LSP-Framework (rust-analyzer, synchron) | 0.10.0 | 2026-07-16 | 15,7 Mio. | MIT/Apache-2.0 |
| `pdf_oxide` | PDF-Extraktion, reines Rust | 0.3.78 | 2026-09-08 | 0,9 Mio. | MIT OR Apache-2.0 |
| `pdf-extract` | PDF-Text, reines Rust | 0.12.0 | 2026-06-25 | 5,0 Mio. | MIT |
| `lopdf` | PDF-Objektmodell | 0.45.0 | 2026-09-08 | 19,3 Mio. | MIT |
| `hayro` | PDF-Rendering, reines Rust | 0.7.1 | 2026-06-05 | 2,2 Mio. | Apache-2.0 OR MIT |
| `pdfium-render` | Bindung an pdfium | 0.9.4 | 2026-09-06 | 2,3 Mio. | MIT OR Apache-2.0 |
| `mupdf` (mupdf-rs) | Bindung an MuPDF | 0.8.0 | 2026-06-22 | 1,6 Mio. | **AGPL-3.0** |

## Repos

| Repo | Letzter Push | Sterne | Lizenz |
|---|---|---|---|
| `typst/typst` | 2026-09-14 | 56.012 | Apache-2.0 |
| `Myriad-Dreamin/tinymist` | 2026-09-14 | 3.532 | Apache-2.0 |
| `tower-lsp-community/tower-lsp-server` | 2026-09-11 | 222 | Apache-2.0 |
| `openlawlibrary/pygls` (Python-LSP, zum Vergleich) | 2026-09-14 | 808 | Apache-2.0 |
| `asg017/sqlite-vec` (Vektorsuche in SQLite) | 2026-05-18 | 8.107 | Apache-2.0 |
| `anush008/fastembed-rs` (lokale Embeddings) | 2026-09-12 | 1.009 | Apache-2.0 |
| `messense/mupdf-rs` | 2026-09-14 | 206 | AGPL-3.0 |
| `ajrcarey/pdfium-render` | 2026-08-16 | 705 | (GitHub: NOASSERTION; crates.io: MIT OR Apache-2.0) |
| `uben0/tree-sitter-typst` | 2025-04-02 | 193 | MIT |
| `zotero/zotero` | 2026-09-14 | 15.272 | AGPL-3.0 (GitHub: NOASSERTION) |

## Vorhandene Toolchains (gemessen)

- `cargo`/`rustc` 1.95.0, `python3` 3.13.7, `uv` 0.9.6, `node` v24.14.1
- `clang` + `libclang-18`, `pkg-config`, `make`
- `fontconfig`-Entwicklungsdateien **fehlen** (relevant für mupdf-rs mit Standard-Features)
- `mutool` 1.25.1 installiert (ohne ICC-Support)
- `pdftotext` installiert

## Einschätzung Sprachwahl (Ergebnis)

- Kein Baustein der ersten Version braucht Python: Zotero (HTTP), Typst-Parser, LSP, SQLite, PDF-Extraktion sind alle in Rust verfügbar.
- Python wird erst für spätere ML-Teile interessant (und auch dort gibt es `fastembed-rs`); dann als optionaler Nebenprozess.
- Entscheidung: **Rust**, ein Programm `bib` (siehe `12`).
