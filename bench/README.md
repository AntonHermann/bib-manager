# Extraction benchmark

Measures PDF backends against hand-curated assertions (olmOCR-Bench style): do known sentences
appear contiguously and in the correct order in the normalized text? Are expected special
characters present? How much data garbage is produced?

```sh
cargo run -p extract-bench -- fetch
BIB_PDFIUM_LIB_DIR=$PWD/bench/cache/pdfium/lib \
  cargo run --release -p extract-bench --features pdfium -- run --out bench/results/<date>.md
```

`fetch` must run before `cargo test --workspace`: `bib-extract`'s integration tests need the
corpus files in `bench/cache/` (see `crates/bib-extract/tests/common/mod.rs`).

`libpdfium` is not in the repo and must be downloaded once before a run with `--features pdfium`
(path as above in `BIB_PDFIUM_LIB_DIR`):

```sh
mkdir -p bench/cache/pdfium && curl -sSfL https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-linux-x64.tgz | tar xz -C bench/cache/pdfium
```

`releases/latest` points to whichever version is newest; which build was actually loaded
is recorded in every report under "Backends" (the `libpdfium` version number).

## Curation

Per document:

- **Three sentences:** (a) the first sentence of the abstract, (b) a sentence from the right
  column or the second half of page 2, (c) a sentence from the last paragraph before the
  references.
- **Rules for sentences:** at least 8 words, no formulas, no citation markers, no footnote
  markers, ends with a period. Copied verbatim from the LaTeX source, not from a PDF
  extraction (otherwise the benchmark would measure the tool used to curate it). No sentence
  that wraps at a real hyphen in the PDF (`fine-⏎tuned`): normalization joins `-` at line
  breaks, so the sentence would be unfindable for every backend.
- **One order pair:** `before = [["<start of b>", "<start of c>"]]`, each a unique
  snippet of at least 5 words.
- **Characters:** Greek letters rendered in body text (`\epsilon`, `\varepsilon` → `ε`;
  `\delta` → `δ`).
- **Verification against the source:** every sentence must appear in the LaTeX source:
  `tr -s '[:space:]' ' ' < <file>.tex | grep -F -c '<sentence>'` ≥ 1.
- **Verification in the PDF:** search for every sentence in the PDF viewer and find it
  visibly (macros can alter the rendered text).
- Documents without a LaTeX source (Dwork 2006): sentences taken only from the PDF viewer,
  checked visually against the page.

If after the first run **not a single** backend finds a sentence, the assertion is probably
wrong: check and correct it in the PDF before results are scored.

## Decision rule (step 0a)

R(b) = sentences found / all sentences across the corpus, O(b) = correct order pairs / all pairs.
`mutool` is reference only (AGPL, not selectable as default).

1. **`pdf_oxide` remains the default** if R(pdf_oxide) ≥ max(R(pdfium), R(mutool)) − 0.05
   **and** O(pdf_oxide) ≥ max(O(pdfium), O(mutool)) − 0.05
   **and** for every document where pdf_oxide is missing characters or has control
   characters/`(cid:`/U+FFFD, `pdf-extract` provides all expected characters (the cascade holds).
2. **Otherwise pdfium**, if it satisfies condition 1 with pdfium in place of pdf_oxide.
3. **Otherwise stop** and discuss the results with the user.
