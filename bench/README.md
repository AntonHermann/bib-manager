# Extraktions-Benchmark

Misst PDF-Backends an handkuratierten Aussagen (Stil olmOCR-Bench): Stehen bekannte Sätze
zusammenhängend und in der richtigen Reihenfolge im normalisierten Text? Sind erwartete
Sonderzeichen vorhanden? Wie viel Datenmüll entsteht?

```sh
cargo run -p extract-bench -- fetch
BIB_PDFIUM_LIB_DIR=$PWD/bench/cache/pdfium/lib \
  cargo run --release -p extract-bench --features pdfium -- run --out bench/results/<datum>.md
```

## Kuration

Pro Dokument:

- **Drei Sätze:** (a) der erste Satz des Abstracts, (b) ein Satz aus der rechten Spalte
  bzw. der zweiten Hälfte von Seite 2, (c) ein Satz aus dem letzten Absatz vor den Referenzen.
- **Regeln für Sätze:** mindestens 8 Wörter, keine Formeln, keine Zitatmarker, keine
  Fußnotenzeichen, endet mit Punkt. Wörtlich aus der LaTeX-Quelle, nicht aus einer
  PDF-Extraktion kopiert (sonst misst der Benchmark das Werkzeug, mit dem kuratiert wurde).
  Kein Satz, der im PDF an einem echten Bindestrich umbricht (`fine-⏎tuned`): Die
  Normalisierung verbindet `-` am Zeilenende, der Satz wäre für jedes Backend unauffindbar.
- **Ein Reihenfolge-Paar:** `before = [["<Anfang von b>", "<Anfang von c>"]]`, je ein
  eindeutiges Stück von mindestens 5 Wörtern.
- **Zeichen:** griechische Buchstaben, die im Fließtext gerendert werden (`\epsilon`,
  `\varepsilon` → `ε`; `\delta` → `δ`).
- **Prüfung gegen die Quelle:** jeder Satz muss in der LaTeX-Quelle stehen:
  `tr -s '[:space:]' ' ' < <datei>.tex | grep -F -c '<satz>'` ≥ 1.
- **Prüfung im PDF:** jeden Satz im PDF-Viewer suchen und sichtbar finden (Makros können den
  gerenderten Text verändern).
- Dokumente ohne LaTeX-Quelle (Dwork 2006): Sätze nur aus dem PDF-Viewer, per Augenschein
  gegen die Seite geprüft.

Findet nach dem ersten Lauf **kein einziges** Backend einen Satz, ist vermutlich die Aussage
falsch: im PDF prüfen und korrigieren, bevor Ergebnisse gewertet werden.
