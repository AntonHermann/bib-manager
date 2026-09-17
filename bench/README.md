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

## Entscheidungsregel (Schritt 0a)

R(b) = gefundene Sätze / alle Sätze über den Korpus, O(b) = korrekte Reihenfolge-Paare / alle Paare.
`mutool` ist nur Referenz (AGPL, nicht als Standard wählbar).

1. **`pdf_oxide` bleibt Standard**, wenn R(pdf_oxide) ≥ max(R(pdfium), R(mutool)) − 0,05
   **und** O(pdf_oxide) ≥ max(O(pdfium), O(mutool)) − 0,05
   **und** bei jedem Dokument, in dem pdf_oxide Zeichen fehlen oder Steuerzeichen/`(cid:`/U+FFFD
   auftreten, `pdf-extract` alle erwarteten Zeichen liefert (die Kaskade trägt).
2. **Sonst pdfium**, wenn es Bedingung 1 mit pdfium an Stelle von pdf_oxide erfüllt.
3. **Sonst anhalten** und die Ergebnisse mit dem Nutzer besprechen.
