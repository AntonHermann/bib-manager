"""pdftotext-Datei, pdfplumber und PyMuPDF (AGPL) im Vergleich: Anzahl ε und Schriftarten.

Aufruf: uv run -q --with pymupdf --with pdfplumber python python_extractors.py <pdf> [pdftotext.txt]
"""
import re
import sys

import pdfplumber
import pymupdf

EPS = r"[εϵ𝜖𝜀]"


def report(label, text):
    m = re.search(r".{0,12}-differential privacy", text)
    sample = repr(m.group(0)) if m else "-"
    print(f"{label:10} eps={len(re.findall(EPS, text)):4}  sample={sample}")


path = sys.argv[1]
doc = pymupdf.open(path)
report("pymupdf", "".join(page.get_text() for page in doc))
with pdfplumber.open(path) as pdf:
    report("pdfplumber", "".join((page.extract_text() or "") for page in pdf.pages))
if len(sys.argv) > 2:
    report("pdftotext", open(sys.argv[2], encoding="utf-8", errors="replace").read())
print("fonts:", sorted({f[3] for page in doc for f in page.get_fonts()}))
