# Semantic Reader Project (Allen AI und Partner)

Recherche 2026-09-14. Zusammenarbeit von NLP- und HCI-Forschenden (AI2, UC Berkeley,
University of Washington u. a.). Ergebnis ist die Semantic-Reader-Anwendung auf
semanticscholar.org plus eine Reihe von Forschungsprototypen.

Überblick: [Semantic Reader Project (arXiv)](https://ar5iv.labs.arxiv.org/html/2303.14334) · [CACM-Artikel](https://dl.acm.org/doi/10.1145/3659096) · [Open Research Platform](https://openreader.semanticscholar.org/)

## Konzepte aus den Prototypen

| Prototyp | Idee | Relevanz für uns |
|---|---|---|
| **CiteSee** | Zitate im PDF farbig nach persönlichem Kontext: schon gelesen, gespeichert, selbst zitiert. Paper-Karten erklären, wie die zitierte Arbeit zur eigenen Lesehistorie steht. Als Chrome-Extension auf Basis von ScholarPhi gebaut (~5.000 neue, ~17.000 Zeilen TypeScript gesamt). | Einfärbung nach A/B/C-Stufe, zitiert, ungelesen, fehlt |
| **CiteRead** | Zeigt am Rand der Quelle, was spätere zitierende Papers **über genau diese Stelle** sagen. Drei Beiträge: wichtige Citer auswählen, Kommentar in der Quelle lokalisieren, Randnotiz-Interaktion. Studie mit 12 Forschenden: besseres Verständnis und Erinnern als mit einer Liste von Citern. | Gegenrichtung zu „wo zitiere ich das?" |
| **ScholarPhi** | Definitionen von Begriffen und Symbolen direkt an der Stelle; bevorzugt die Definition kurz vor der Verwendung; Formel-Erklärungen mit Symboldefinitionen am Rand. | ε und Co. in DP-Formeln |
| **Scim** | Beim Überfliegen werden Ziel, Neuheit, Methode, Ergebnis farbig hervorgehoben. | Belegquellen, die man nicht ganz liest |
| **Relatedly** | Lesen und Querverweisen von Related-Work-Absätzen, die sich über Papers hinweg überschneiden. | Überblick über ein Feld |
| **Threddy** | Beim Lesen Passagen (v. a. aus Related Work anderer Papers) sammeln und zu eigenen hierarchischen „Threads" ordnen. | Gliederung entsteht beim Lesen |
| **Synergi** | Wie Threddy, plus LLM-generierte hierarchische Zusammenfassungen; KI als Gerüst, Urteil bleibt beim Menschen. | Gruppierung nach Kapiteln |
| **PaperWeaver** | Empfiehlt neue Papers basierend auf der eigenen Sammlung. | Lücken in der Bibliothek |
| **Papeos** | Verknüpft Paper-Stellen mit Vortragsvideos. | eher nicht |
| **ReaderQuizzer** | Just-in-time-Verständnisfragen beim Lesen. | eher nicht |

Quellen: [CiteSee](https://arxiv.org/pdf/2302.07302) · [CiteRead](https://dl.acm.org/doi/fullHtml/10.1145/3490099.3511162) · [Threddy](https://arxiv.org/html/2208.03455) · [Synergi](https://ar5iv.labs.arxiv.org/html/2308.07517) · [Papeos](https://arxiv.org/pdf/2308.15224) · [ReaderQuizzer](https://arxiv.org/pdf/2308.07988) · [Semantic Reader Produktseite](https://www.semanticscholar.org/product/semantic-reader)

## Zustand des Open-Source-Codes

Abgefragt über die GitHub-API am 2026-09-14.

| Repo | Zweck | Letzter Push | Sterne | Lizenz | Einschätzung |
|---|---|---|---|---|---|
| `allenai/papermage` | PDF → Dokument aus Schichten (Symbole, Tokens, Zeilen, Sätze, Absätze, Abschnitte, Literaturverzeichnis, Gleichungen, Tabellen …; 27 Schichten), Parser + Rasterizer + Predictors | 2024-11-08 | 803 | Apache-2.0 | README: *„research prototype for EMNLP 2023 … unlikely to be addressing issues / maintaining this on a regular cadence“*, Nachfolge unter AI2s Dolma-Projekt angekündigt. **Parser ist pdfplumber** → erbt dessen ε-Problem (siehe `04`). |
| `allenai/pdf-component-library` („PaperCraft") | React-Komponenten für PDF-Leser mit Overlays, Citation Cards, Thumbnails, Notizen; auf React-PDF aufgebaut | 2024-02-15 | 93 | Repo ohne Lizenzdatei; `ui/library/package.json`: Apache-2.0, Paket `@allenai/pdf-components` 0.0.1 | eingeschlafen |
| `allenai/scholarphi` | interaktiver PDF-Leser, Basis von CiteSee | 2023-07-19 | 428 | Apache-2.0 | eingeschlafen |
| `allenai/s2orc-doc2json` | PDF/LaTeX → JSON (S2ORC-Format) | 2024-04-11 | 476 | Apache-2.0 | eingeschlafen |
| `allenai/olmocr` | PDF → Markdown mit 7B-Vision-Language-Modell; Formeln, Tabellen, Leserichtung, entfernt Kopf-/Fußzeilen | 2026-03-25 | 19.468 | Apache-2.0 | **aktiv**, aber lokal nur mit NVIDIA-GPU ≥ 12 GB VRAM (getestet RTX 4090, L40S, A100, H100), ~30 GB Platz. Gehostet bei Cirrascale, DeepInfra, Parasail für ca. 0,07–0,20 $ pro Million Tokens. |

Auf der Open-Research-Platform-Seite werden PaperMage und PaperCraft weiterhin als „aktiv" dargestellt, das deckt sich nicht mit den Repos.

Quellen: [papermage](https://github.com/allenai/papermage) · [pdf-component-library](https://github.com/allenai/pdf-component-library) · [scholarphi](https://github.com/allenai/scholarphi) · [olmocr](https://github.com/allenai/olmocr) · [olmOCR-Paper](https://arxiv.org/html/2502.18443)

## Schlussfolgerungen

1. **Konzepte übernehmen, Code nicht.** Besonders wertvoll: PaperMages Schichtenmodell (ein kanonischer Text, jede Schicht ist eine Menge von Bereichen plus Boxen), CiteSee-Färbung, CiteRead-Randnotizen.
2. Das Schichtenmodell vereinheitlicht Notiz-Anker, Wortlaut-Prüfung und spätere Zitatmarker-Färbung.
3. Die CiteSee-Färbung braucht die Verknüpfung „[18]" ↔ 18. Literaturverzeichnis-Eintrag. Das liefert GROBID (siehe `05`).
