# Test programs for the PDF measurements

Reconstructed from the runs on 2026-09-14 (see `../04-pdf-extraction.md`). Throwaway code,
for reproduction only. All programs count `ε ϵ 𝜖 𝜀` and write the text to `out/`.

## Setup

```sh
mkdir -p pdfs out
curl -sL -o pdfs/dwork2006.pdf https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/dwork.pdf
curl -sL -o pdfs/abadi2016.pdf https://arxiv.org/pdf/1607.00133
# pdfium only:
mkdir -p pdfium_bin && curl -sL https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-linux-x64.tgz | tar xz -C pdfium_bin
```

## Runs

```sh
export CARGO_TARGET_DIR=$PWD/target
for doc in dwork2006 abadi2016; do
  pdftotext -q -nopgbrk pdfs/$doc.pdf out/${doc}_pdftotext.txt
  cargo run -q --release --manifest-path mupdf_probe/Cargo.toml      --bin mupdf_probe      pdfs/$doc.pdf out/${doc}_mupdf.txt
  cargo run -q --release --manifest-path pdfextract_probe/Cargo.toml --bin pdfextract_probe pdfs/$doc.pdf out/${doc}_pdfextract.txt
  cargo run -q --release --manifest-path pdfium_probe/Cargo.toml     --bin pdfium_probe     pdfs/$doc.pdf out/${doc}_pdfium.txt pdfium_bin/lib/
  cargo run -q --release --manifest-path oxide_probe/Cargo.toml      --bin oxide_probe      pdfs/$doc.pdf out/${doc}_oxide.txt
done
uv run -q --with pymupdf --with pdfplumber python python_extractors.py pdfs/dwork2006.pdf
python3 compare.py out
mutool draw -q -F stext.json -o out/dwork_stext.json pdfs/dwork2006.pdf
```

## Notes

- `mupdf_probe` built without default features, because the `fontconfig` headers are missing (`font-kit`). **AGPL-3.0.**
- `pdfium_probe` needs `libpdfium.so` at runtime.
- `compare.py`: random windows with a fixed seed of 1; results should match the tables in `04`.
- The measurement on the seminar PDFs (Dankar 2013) is not included because those PDFs aren't public.
