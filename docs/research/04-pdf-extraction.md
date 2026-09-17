# PDF text extraction: measurements and decision

Measured on 2026-09-14 on the development laptop (see `09`). The test programs live
in [`probes/`](probes/).

## Starting problem

In the EHR seminar, full texts were produced with `pdftotext -nopgbrk`. Documented pitfalls:

- **Tables** fall apart into columns of numbers without row alignment without `-layout` (led to a wrong number in the notes).
- **ε disappears** in several DP papers ("-differential privacy").
- **Math-italic characters:** one book sets `𝜖` (U+1D716) instead of `ε`; `grep 'ε'` finds almost nothing.

## Test 1: three extractors on two seminar PDFs (local, unpublished)

Counted: occurrences of `ε ϵ 𝜖 𝜀` in the full text.

| PDF | pdftotext | pdfplumber | PyMuPDF |
|---|---|---|---|
| Dwork 2006, *Differential Privacy* | 0 (`\x0f`) | 0 (`(cid:15)`) | **28** |
| Dankar 2013, *Practicing Differential Privacy in Health Care* | 0 | 0 | 0 |

Fonts:
- Dwork 2006: LaTeX Computer Modern as **Type1** (`CMMI9`, `CMSY10`, `CMR10`, …), the math font without a `ToUnicode` table.
- Dankar 2013: Palatino, `SymbolMT`, `MT-Extra`, Wingdings (Word/MathType style). All three fail here; only OCR/a vision model or viewing the PDF spot directly helps.

Conclusion: **PaperMage (which uses pdfplumber) would have inherited the ε problem.**

## Test 2: five extractors on two public PDFs

- `dwork2006.pdf`: Dwork 2006, [Microsoft Research copy](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/dwork.pdf), 6 pages, PDF 1.4, Type1 fonts.
- `abadi2016.pdf`: Abadi et al. 2016, *Deep Learning with Differential Privacy*, [arXiv:1607.00133](https://arxiv.org/pdf/1607.00133), two-column, modern pdfTeX. **Note:** according to `file`, the download had only 2 pages, possibly not the full paper. Re-check for the benchmark.

### ε preservation

| Library | Language | License | Dwork 2006 | Abadi 2016 | Output at the ε spot (Dwork) |
|---|---|---|---|---|---|
| pdftotext (Poppler) | C++ | GPL | 0 | – | `K gives \x0f-differential privacy` |
| pdfplumber | Python | MIT | 0 | – | `(cid:15)-differential privacy` |
| PyMuPDF | Python/C | AGPL-3.0 | 28 | – | – |
| **mupdf-rs** 0.8 | Rust/C | AGPL-3.0 | **28** | 92 | `K gives ϵ-differential privacy` |
| **pdfium-render** 0.9.4 (+ libpdfium 155.0.8057) | Rust/C++ | MIT OR Apache-2.0, pdfium BSD-3 | **0** | 92 | `K gives \u{f}-differential privacy` |
| pypdfium2 (pdfium 153.0.7999) | Python | Apache/BSD | 0 | – | `\x0f-differential privacy` |
| **pdf-extract** 0.12.0 | Rust | MIT | **28** | 92 | ε present, phrase not found by search due to line break |
| **pdf_oxide** 0.3.78 | Rust | MIT OR Apache-2.0 | **0** | 92 | `K gives -differential privacy` (character missing entirely) |
| mutool 1.25.1 (`draw -F stext.json`) | CLI | AGPL-3.0 | 28 | – | – |

Observation: all ε occurrences detected in Dwork 2006 are `ϵ` (U+03F5, "lunate epsilon"), not `ε` (U+03B5). Normalization must treat both as equal. NFKC does **not** do this automatically for U+03F5 → U+03B5; a custom equivalence table is needed (also for `𝜖` U+1D716, `𝜀` U+1D700, which NFKC maps to `ϵ`/`ε`).

### Why the ε is missing in some cases

- The Type1 math font has no Unicode mapping. Character code 0x0F corresponds to the glyph name `epsilon1` in the **embedded font encoding table**.
- **pdf-extract** reads this table with `type1_encoding_parser` from `FontFile` and maps glyph names to Unicode via the Adobe Glyph List (`glyphnames.rs`) (`src/lib.rs` starting around line ~381).
- **pdf_oxide** has the same building blocks (`src/fonts/type1_encoding.rs`, `glyph_name_to_unicode` in `src/fonts/font_dict.rs:5377`, even `builtin_encoding_looks_like_cipher` for a plausibility check), but doesn't apply them here, or discards the character. **Hypothesis:** a small, manageable gap, suitable for an upstream contribution. Not investigated further.
- **pdfium** doesn't use the embedded encoding in this case.

### Text quality beyond ε

Normalization before comparison: NFKC, strip line-end hyphens, collapse whitespace, lowercase.

**Word count:**

| | pdftotext | mupdf-rs | pdf-extract | pdfium | pdf_oxide |
|---|---|---|---|---|---|
| Dwork 2006 | 6121 | 5904 | 6052 | 6043 | 6156 |
| Abadi 2016 | 11946 | 11321 | 11647 | 11642 | 11781 |

**Agreement:** 200 random 8-word windows from library X (row), found as a contiguous sequence in library Y (column).

Dwork 2006:

| from \ in | pdftotext | mupdf | pdf-extract | pdfium | pdf_oxide |
|---|---|---|---|---|---|
| pdftotext | – | 74% | 78% | 80% | 70% |
| mupdf | 81% | – | 88% | 79% | 64% |
| pdf-extract | 84% | 86% | – | 77% | 70% |
| pdfium | 86% | 82% | 84% | – | 69% |
| pdf_oxide | 76% | 66% | 72% | 69% | – |

Abadi 2016:

| from \ in | pdftotext | mupdf | pdf-extract | pdfium | pdf_oxide |
|---|---|---|---|---|---|
| pdftotext | – | 78% | 82% | 66% | 76% |
| mupdf | 84% | – | 89% | 76% | 77% |
| pdf-extract | 82% | 83% | – | 74% | 74% |
| pdfium | 70% | 73% | 76% | – | 64% |
| pdf_oxide | 72% | 69% | 72% | 60% | – |

- `pdf_oxide` deviates most strongly from the others. **Without a reference text, it can't be decided who is right about reading order and word breaks.**
- A known sentence from the abstract of Abadi 2016 wasn't found exactly by **any** extractor (probably a line break/hyphenation or an incomplete download) → the benchmark needs a more robust method.
- Ligatures: `pdf-extract` outputs 96 ligature characters (`ﬁ ﬂ ﬀ ﬃ ﬄ`), mupdf-rs 0. NFKC resolves them.
- Wording comparison via `difflib` (Dwork 2006): pdf-extract vs mupdf 0.949; pdftotext vs mupdf 0.932.

### Position data

| Library | What you get |
|---|---|
| mupdf-rs / mutool | blocks → lines → characters with quad, origin, font, size, color, flags |
| pdfium-render | characters with box, text ranges, font |
| pdf_oxide | `TextChar` with `bbox`, font name, size, weight, italic, monospace, color, MCID (tagged PDF), origin, rotation, advance, ascent/descent, matrix; also `extract_spans`, profiles such as `ExtractionProfile.academic()` |
| pdf-extract | only raw events via the `OutputDev` trait (`output_character(trm, width, spacing, font_size, char)`, `begin_word`, `end_line`); words, lines, reading order must be built yourself |

Further properties:
- `pdf-extract` contains `expect` calls in font parsing (e.g. `.expect("encoding")`) → possible panics on broken PDFs.
- `mupdf-rs`'s default features pull in `font-kit` → requires `fontconfig` headers. With `default-features = false, features = ["base14-fonts"]` it builds without them; release build ~42s on 16 threads.
- `pdfium-render` needs a bundled `libpdfium.so` (e.g. from [bblanchon/pdfium-binaries](https://github.com/bblanchon/pdfium-binaries), MIT build scripts, pdfium itself BSD-3).

## Licenses (short version from the discussion)

| Type | Examples | Obligation on distribution |
|---|---|---|
| permissive | MIT, BSD, Apache-2.0 | license notice; Apache adds a patent clause |
| weak copyleft | LGPL, MPL-2.0 | disclose changes to the library itself |
| strong copyleft | GPL | the whole distributed program must be GPL, source must be provided |
| network copyleft | AGPL-3.0 | like GPL, plus applies when a modified version is offered over a network |

- Obligations arise only on distribution (AGPL: including third-party network access). Private use is free.
- MuPDF is dual-licensed (AGPL or commercial via Artifex).
- A separately launched program (e.g. `mutool` as a subprocess) generally doesn't make the calling program a derivative work.
- Not legal advice.

## Decision (confirmed by the user)

- Project license **MIT OR Apache-2.0**. User's rationale: broad impact on science, including via closed-source software, matters more than copyleft.
- Extraction **behind an interface**; the core only knows characters with Unicode, box, font, page.
- Default backend **`pdf_oxide`** plus **a custom repair step** for characters without a Unicode mapping (Type1 encoding + glyph names, modeled on `pdf-extract`), ideally as an upstream pull request.
- **Optional `mutool`** as an external process, if installed (comparison/fallback).
- **First planning step: a benchmark** with ~10 open-access papers of varying age and layout, reference derived from arXiv LaTeX sources. Metrics: wording matches, ε/special characters, ligatures, reading order. If `pdf_oxide` performs poorly → switch to pdfium + repair step.

## Other extraction tools (not measured)

| Tool | Status 2026-09-14 | Note |
|---|---|---|
| GROBID | active, Apache-2.0, CPU | structure, sentences, citation markers, reference list with coordinates (see `05`) |
| Docling | active, MIT, 66k stars | every element with provenance, page, box; formula enrichment CPU/CUDA only |
| Marker | active, Apache-2.0 (check model weight license separately) | olmOCR-Bench "balanced" 76.0 |
| MinerU | active, own license | strong for Chinese and scientific text |
| olmOCR 2 | active, Apache-2.0 | olmOCR-Bench 82.4; locally NVIDIA ≥ 12 GB only |
| Chandra 2 (Datalab) | – | olmOCR-Bench 85.8 |

Benchmarks: **olmOCR-Bench** (~1,400 documents, >7,000 unit-test-style checks, ODC-BY) and **OmniDocBench** (1,651 pages, 10 document types).

Sources: [olmOCR paper](https://olmocr.allenai.org/papers/olmocr.pdf) · [OmniDocBench](https://github.com/opendatalab/OmniDocBench) · [Formula extraction benchmark](https://arxiv.org/html/2512.09874v1) · [Table extraction benchmark](https://arxiv.org/html/2603.18652v1) · [MinerU vs Docling vs Marker](https://builderai.tools/blog/pdf-parsing-for-rag-mineru-docling-marker-compared) · [Docling Technical Report](https://arxiv.org/pdf/2408.09869) · [DoclingDocument](https://docling-project.github.io/docling/reference/docling_document/) · [mupdf-rs](https://github.com/messense/mupdf-rs) · [pdfium-render](https://github.com/ajrcarey/pdfium-render) · [pdf-extract](https://crates.io/crates/pdf-extract) · [pdf_oxide](https://crates.io/crates/pdf_oxide)

## Robust anchors (W3C Web Annotation)

- **TextQuoteSelector:** exact text + prefix + suffix; used in almost all annotation systems, usually sufficient.
- Combined with **TextPositionSelector** for robustness.
- **Fuzzy anchoring** (Hypothesis): verbatim → deterministic normalization → Levenshtein within ~5% tolerance.
- Especially important for PDFs: unstable extraction order, ligatures, OCR changes, multi-column layouts, different whitespace per viewer.
- Library: `anchor-quote` (Robert Knight), successor to the Hypothesis logic.
- **Design consequence:** anchors are a data-model decision, not something to retrofit. Pure coordinate anchors are lost on re-extraction.

Sources: [Hypothesis: Fuzzy Anchoring](https://web.hypothes.is/blog/fuzzy-anchoring/) · [W3C Web Annotation Data Model](https://www.w3.org/TR/annotation-model/) · [anchor-quote](https://github.com/robertknight/anchor-quote) · [Jon Udell: Notes for an annotation SDK](https://blog.jonudell.net/2021/09/03/notes-for-an-annotation-sdk/) · [Semiont: W3C selectors](https://github.com/The-AI-Alliance/semiont/blob/main/docs/protocol/W3C-SELECTORS.md)
