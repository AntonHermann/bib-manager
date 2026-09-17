# PDF-Textextraktion: Messungen und Entscheidung

Gemessen am 2026-09-14 auf dem Entwicklungs-Laptop (siehe `09`). Die Testprogramme liegen
in [`probes/`](probes/).

## Ausgangsproblem

Im EHR-Seminar wurden Volltexte mit `pdftotext -nopgbrk` erzeugt. Dokumentierte Fallen:

- **Tabellen** zerfallen ohne `-layout` in Zahlenkolonnen ohne Zeilenzuordnung (führte zu einer falschen Zahl in den Notizen).
- **ε verschwindet** in mehreren DP-Papers („-differential privacy").
- **Mathematisch-kursive Zeichen:** Ein Buch setzt `𝜖` (U+1D716) statt `ε`, ein `grep 'ε'` findet fast nichts.

## Test 1: drei Extraktoren an zwei Seminar-PDFs (lokal, nicht veröffentlicht)

Gezählt: Vorkommen von `ε ϵ 𝜖 𝜀` im Gesamttext.

| PDF | pdftotext | pdfplumber | PyMuPDF |
|---|---|---|---|
| Dwork 2006, *Differential Privacy* | 0 (`\x0f`) | 0 (`(cid:15)`) | **28** |
| Dankar 2013, *Practicing Differential Privacy in Health Care* | 0 | 0 | 0 |

Schriften:
- Dwork 2006: LaTeX-Computer-Modern als **Type1** (`CMMI9`, `CMSY10`, `CMR10` …), die Mathe-Schrift ohne `ToUnicode`-Tabelle.
- Dankar 2013: Palatino, `SymbolMT`, `MT-Extra`, Wingdings (Word/MathType-Stil). Hier scheitern alle drei; nur OCR/Vision-Modell oder Anzeige der PDF-Stelle helfen.

Folgerung: **PaperMage (nutzt pdfplumber) hätte das ε-Problem geerbt.**

## Test 2: fünf Extraktoren an zwei öffentlichen PDFs

- `dwork2006.pdf`: Dwork 2006, [Microsoft-Research-Kopie](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/dwork.pdf), 6 Seiten, PDF 1.4, Type1-Schriften.
- `abadi2016.pdf`: Abadi et al. 2016, *Deep Learning with Differential Privacy*, [arXiv:1607.00133](https://arxiv.org/pdf/1607.00133), zweispaltig, modernes pdfTeX. **Achtung:** Laut `file` hatte der Download nur 2 Seiten, möglicherweise nicht das vollständige Paper. Für den Benchmark neu prüfen.

### ε-Erhalt

| Bibliothek | Sprache | Lizenz | Dwork 2006 | Abadi 2016 | Ausgabe an der ε-Stelle (Dwork) |
|---|---|---|---|---|---|
| pdftotext (Poppler) | C++ | GPL | 0 | – | `K gives \x0f-differential privacy` |
| pdfplumber | Python | MIT | 0 | – | `(cid:15)-differential privacy` |
| PyMuPDF | Python/C | AGPL-3.0 | 28 | – | – |
| **mupdf-rs** 0.8 | Rust/C | AGPL-3.0 | **28** | 92 | `K gives ϵ-differential privacy` |
| **pdfium-render** 0.9.4 (+ libpdfium 155.0.8057) | Rust/C++ | MIT OR Apache-2.0, pdfium BSD-3 | **0** | 92 | `K gives \u{f}-differential privacy` |
| pypdfium2 (pdfium 153.0.7999) | Python | Apache/BSD | 0 | – | `\x0f-differential privacy` |
| **pdf-extract** 0.12.0 | Rust | MIT | **28** | 92 | ε vorhanden, Phrase wegen Zeilenumbruch nicht per Suche gefunden |
| **pdf_oxide** 0.3.78 | Rust | MIT OR Apache-2.0 | **0** | 92 | `K gives -differential privacy` (Zeichen fehlt ganz) |
| mutool 1.25.1 (`draw -F stext.json`) | CLI | AGPL-3.0 | 28 | – | – |

Beobachtung: Alle erkannten ε in Dwork 2006 sind `ϵ` (U+03F5, „lunate epsilon"), nicht `ε` (U+03B5). Die Normalisierung muss beide gleichsetzen. NFKC tut das **nicht** automatisch für U+03F5 → U+03B5; eigene Äquivalenztabelle nötig (auch für `𝜖` U+1D716, `𝜀` U+1D700, die NFKC auf `ϵ`/`ε` abbildet).

### Warum das ε bei manchen fehlt

- Die Type1-Mathe-Schrift hat keine Unicode-Zuordnung. Der Zeichencode 0x0F entspricht dem Glyphennamen `epsilon1` in der **eingebetteten Schrift-Encoding-Tabelle**.
- **pdf-extract** liest diese Tabelle mit `type1_encoding_parser` aus `FontFile` und bildet Glyphennamen über die Adobe Glyph List (`glyphnames.rs`) auf Unicode ab (`src/lib.rs` ab Zeile ~381).
- **pdf_oxide** besitzt die Bausteine auch (`src/fonts/type1_encoding.rs`, `glyph_name_to_unicode` in `src/fonts/font_dict.rs:5377`, sogar `builtin_encoding_looks_like_cipher` zur Plausibilitätsprüfung), wendet sie hier aber nicht an bzw. verwirft das Zeichen. **Vermutung:** überschaubare Lücke, geeignet für einen Upstream-Beitrag. Nicht weiter untersucht.
- **pdfium** nutzt die eingebettete Encoding in diesem Fall nicht.

### Textqualität jenseits von ε

Normalisierung vor dem Vergleich: NFKC, Trennstriche am Zeilenende entfernen, Leerraum zusammenfassen, Kleinschreibung.

**Wortanzahl:**

| | pdftotext | mupdf-rs | pdf-extract | pdfium | pdf_oxide |
|---|---|---|---|---|---|
| Dwork 2006 | 6121 | 5904 | 6052 | 6043 | 6156 |
| Abadi 2016 | 11946 | 11321 | 11647 | 11642 | 11781 |

**Übereinstimmung:** 200 zufällige 8-Wort-Fenster aus Bibliothek X (Zeile), gefunden als zusammenhängende Folge in Bibliothek Y (Spalte).

Dwork 2006:

| aus \ in | pdftotext | mupdf | pdf-extract | pdfium | pdf_oxide |
|---|---|---|---|---|---|
| pdftotext | – | 74 % | 78 % | 80 % | 70 % |
| mupdf | 81 % | – | 88 % | 79 % | 64 % |
| pdf-extract | 84 % | 86 % | – | 77 % | 70 % |
| pdfium | 86 % | 82 % | 84 % | – | 69 % |
| pdf_oxide | 76 % | 66 % | 72 % | 69 % | – |

Abadi 2016:

| aus \ in | pdftotext | mupdf | pdf-extract | pdfium | pdf_oxide |
|---|---|---|---|---|---|
| pdftotext | – | 78 % | 82 % | 66 % | 76 % |
| mupdf | 84 % | – | 89 % | 76 % | 77 % |
| pdf-extract | 82 % | 83 % | – | 74 % | 74 % |
| pdfium | 70 % | 73 % | 76 % | – | 64 % |
| pdf_oxide | 72 % | 69 % | 72 % | 60 % | – |

- `pdf_oxide` weicht am stärksten von den anderen ab. **Ohne Referenztext ist nicht entscheidbar, wer bei Leserichtung und Worttrennung richtig liegt.**
- Ein bekannter Satz aus dem Abstract von Abadi 2016 wurde von **keinem** Extraktor exakt gefunden (vermutlich Zeilenumbruch/Trennung oder unvollständiger Download) → der Benchmark braucht eine robustere Methode.
- Ligaturen: `pdf-extract` gibt 96 Ligatur-Zeichen (`ﬁ ﬂ ﬀ ﬃ ﬄ`) aus, mupdf-rs 0. NFKC löst sie auf.
- Wortlaut-Vergleich über `difflib` (Dwork 2006): pdf-extract vs mupdf 0,949; pdftotext vs mupdf 0,932.

### Positionsdaten

| Bibliothek | Was man bekommt |
|---|---|
| mupdf-rs / mutool | Blöcke → Zeilen → Zeichen mit Quad, Ursprung, Schrift, Größe, Farbe, Flags |
| pdfium-render | Zeichen mit Box, Textbereiche, Schrift |
| pdf_oxide | `TextChar` mit `bbox`, Schriftname, Größe, Gewicht, kursiv, monospace, Farbe, MCID (Tagged PDF), Ursprung, Rotation, Advance, Ascent/Descent, Matrix; zusätzlich `extract_spans`, Profile wie `ExtractionProfile.academic()` |
| pdf-extract | nur Rohereignisse über den `OutputDev`-Trait (`output_character(trm, width, spacing, font_size, char)`, `begin_word`, `end_line`); Wörter, Zeilen, Leserichtung selbst bauen |

Weitere Eigenschaften:
- `pdf-extract` enthält `expect`-Aufrufe im Font-Parsing (z. B. `.expect("encoding")`) → mögliche Abstürze bei kaputten PDFs.
- `mupdf-rs` Standard-Features ziehen `font-kit` → `fontconfig`-Headers nötig. Mit `default-features = false, features = ["base14-fonts"]` baut es ohne; Release-Build ~42 s auf 16 Threads.
- `pdfium-render` braucht eine mitgelieferte `libpdfium.so` (z. B. von [bblanchon/pdfium-binaries](https://github.com/bblanchon/pdfium-binaries), MIT-Build-Skripte, pdfium selbst BSD-3).

## Lizenzen (Kurzfassung aus dem Gespräch)

| Typ | Beispiele | Pflicht bei Weitergabe |
|---|---|---|
| permissiv | MIT, BSD, Apache-2.0 | Lizenzhinweis; Apache zusätzlich Patentklausel |
| schwaches Copyleft | LGPL, MPL-2.0 | Änderungen an der Bibliothek selbst offenlegen |
| starkes Copyleft | GPL | ganzes weitergegebenes Programm unter GPL, Quellcode mitliefern |
| Netzwerk-Copyleft | AGPL-3.0 | wie GPL, zusätzlich bei Bereitstellung einer geänderten Version über ein Netzwerk |

- Pflichten entstehen erst bei Weitergabe (AGPL: auch Netzwerkzugriff Dritter). Privatnutzung ist frei.
- MuPDF ist doppelt lizenziert (AGPL oder kommerziell über Artifex).
- Ein separat gestartetes Programm (z. B. `mutool` als Subprozess) macht das aufrufende Programm üblicherweise nicht zum abgeleiteten Werk.
- Keine Rechtsberatung.

## Entscheidung (vom Nutzer bestätigt)

- Projektlizenz **MIT OR Apache-2.0**. Begründung des Nutzers: Breite Wirkung auf die Wissenschaft, auch über Closed-Source-Software, ist wichtiger als Copyleft.
- Extraktion **hinter einer Schnittstelle**; der Kern kennt nur Zeichen mit Unicode, Box, Schrift, Seite.
- Standard-Backend **`pdf_oxide`** plus **eigener Reparaturschritt** für Zeichen ohne Unicode-Zuordnung (Type1-Encoding + Glyphennamen, nach Vorbild `pdf-extract`), möglichst als Pull Request upstream.
- **Optional `mutool`** als externer Prozess, wenn installiert (Vergleich/Rückfall).
- **Erster Planschritt: Benchmark** mit ~10 Open-Access-Papers unterschiedlichen Alters und Layouts, Referenz aus arXiv-LaTeX-Quellen. Metriken: Wortlaut-Treffer, ε/Sonderzeichen, Ligaturen, Leserichtung. Schneidet `pdf_oxide` schlecht ab → Wechsel auf pdfium + Reparaturschritt.

## Andere Extraktionswerkzeuge (nicht gemessen)

| Werkzeug | Stand 2026-09-14 | Notiz |
|---|---|---|
| GROBID | aktiv, Apache-2.0, CPU | Struktur, Sätze, Zitatmarker, Referenzliste mit Koordinaten (siehe `05`) |
| Docling | aktiv, MIT, 66k Sterne | jedes Element mit Provenance, Seite, Box; Formel-Anreicherung nur CPU/CUDA |
| Marker | aktiv, Apache-2.0 (Modellgewichte eigene Lizenz prüfen) | olmOCR-Bench „balanced" 76,0 |
| MinerU | aktiv, eigene Lizenz | stark bei Chinesisch und Wissenschaft |
| olmOCR 2 | aktiv, Apache-2.0 | olmOCR-Bench 82,4; lokal nur NVIDIA ≥ 12 GB |
| Chandra 2 (Datalab) | – | olmOCR-Bench 85,8 |

Benchmarks: **olmOCR-Bench** (~1.400 Dokumente, >7.000 Unit-Test-artige Prüfungen, ODC-BY) und **OmniDocBench** (1.651 Seiten, 10 Dokumenttypen).

Quellen: [olmOCR-Paper](https://olmocr.allenai.org/papers/olmocr.pdf) · [OmniDocBench](https://github.com/opendatalab/OmniDocBench) · [Benchmark Formel-Extraktion](https://arxiv.org/html/2512.09874v1) · [Benchmark Tabellen-Extraktion](https://arxiv.org/html/2603.18652v1) · [MinerU vs Docling vs Marker](https://builderai.tools/blog/pdf-parsing-for-rag-mineru-docling-marker-compared) · [Docling Technical Report](https://arxiv.org/pdf/2408.09869) · [DoclingDocument](https://docling-project.github.io/docling/reference/docling_document/) · [mupdf-rs](https://github.com/messense/mupdf-rs) · [pdfium-render](https://github.com/ajrcarey/pdfium-render) · [pdf-extract](https://crates.io/crates/pdf-extract) · [pdf_oxide](https://crates.io/crates/pdf_oxide)

## Robuste Anker (W3C Web Annotation)

- **TextQuoteSelector:** exakter Text + Präfix + Suffix; in fast allen Annotationssystemen, reicht meistens.
- Kombiniert mit **TextPositionSelector** für Robustheit.
- **Fuzzy Anchoring** (Hypothesis): verbatim → deterministische Normalisierung → Levenshtein innerhalb ~5 % Toleranz.
- Bei PDFs besonders wichtig: instabile Extraktionsreihenfolge, Ligaturen, OCR-Änderungen, Mehrspalten-Layouts, unterschiedliche Leerzeichen je Viewer.
- Bibliothek: `anchor-quote` (Robert Knight), Nachfolger der Hypothesis-Logik.
- **Designfolge:** Anker sind eine Datenmodell-Entscheidung, nicht nachrüstbar. Reine Koordinaten-Anker gehen bei Neu-Extraktion verloren.

Quellen: [Hypothesis: Fuzzy Anchoring](https://web.hypothes.is/blog/fuzzy-anchoring/) · [W3C Web Annotation Data Model](https://www.w3.org/TR/annotation-model/) · [anchor-quote](https://github.com/robertknight/anchor-quote) · [Jon Udell: Notes for an annotation SDK](https://blog.jonudell.net/2021/09/03/notes-for-an-annotation-sdk/) · [Semiont: W3C-Selektoren](https://github.com/The-AI-Alliance/semiont/blob/main/docs/protocol/W3C-SELECTORS.md)
