"""Vergleicht die Ausgaben der Extraktoren: Wortanzahl und Übereinstimmung zufälliger 8-Wort-Fenster.

Aufruf: python3 compare.py <out-verzeichnis>
Erwartet Dateien <doc>_<tool>.txt für doc in {dwork2006, abadi2016}.
"""
import random
import re
import sys
import unicodedata

TOOLS = ["pdftotext", "mupdf", "pdfextract", "pdfium", "oxide"]


def norm(s):
    s = unicodedata.normalize("NFKC", s)
    s = re.sub(r"-\s*\n\s*", "", s)
    s = re.sub(r"\s+", " ", s)
    return s.strip().lower()


out = sys.argv[1] if len(sys.argv) > 1 else "out"
for doc in ["dwork2006", "abadi2016"]:
    texts = {t: norm(open(f"{out}/{doc}_{t}.txt", encoding="utf-8", errors="replace").read()) for t in TOOLS}
    print(f"== {doc}")
    for t in TOOLS:
        print(f"  {t:10} words={len(texts[t].split()):5}")
    random.seed(1)
    for ref in TOOLS:
        words = texts[ref].split()
        windows = [" ".join(words[i:i + 8]) for i in (random.randrange(0, len(words) - 8) for _ in range(200))]
        row = "  ".join(
            f"{other[:6]}={sum(w in texts[other] for w in windows) / 2:3.0f}%" for other in TOOLS if other != ref
        )
        print(f"  windows from {ref:10} found in: {row}")
