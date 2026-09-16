# Testprogramme der PDF-Messungen

Rekonstruiert aus den Läufen vom 2026-09-14 (siehe `../04-pdf-extraktion.md`). Wegwerf-Code,
nur zur Reproduktion. Alle Programme zählen `ε ϵ 𝜖 𝜀` und schreiben den Text nach `out/`.

## Vorbereitung

```sh
mkdir -p pdfs out
curl -sL -o pdfs/dwork2006.pdf https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/dwork.pdf
curl -sL -o pdfs/abadi2016.pdf https://arxiv.org/pdf/1607.00133
# nur für pdfium:
mkdir -p pdfium_bin && curl -sL https://github.com/bblanchon/pdfium-binaries/releases/latest/download/pdfium-linux-x64.tgz | tar xz -C pdfium_bin
```

## Läufe

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

## Hinweise

- `mupdf_probe` ohne Standard-Features gebaut, weil `fontconfig`-Header fehlen (`font-kit`). **AGPL-3.0.**
- `pdfium_probe` braucht `libpdfium.so` zur Laufzeit.
- `compare.py`: Zufallsfenster mit festem Seed 1; Ergebnisse sollten den Tabellen in `04` entsprechen.
- Die Messung an den Seminar-PDFs (Dankar 2013) ist nicht enthalten, weil die PDFs nicht öffentlich sind.
