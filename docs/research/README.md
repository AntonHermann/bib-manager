# Recherche zum Bibliography Manager

Stand: 2026-09-14 bis 2026-09-16. Alle Dateien hier sind Rechercheergebnisse aus der
Brainstorming-Phase, keine Spezifikation. Die Spec entsteht separat.

## Inhalt

| Datei | Thema |
|---|---|
| [01-tool-landscape.md](01-tool-landscape.md) | Bestehende Tools: Zitationsgraphen, Smart Citations, Extraktions-Matrizen, Canvas-Werkzeuge |
| [02-semantic-reader.md](02-semantic-reader.md) | Semantic-Reader-Projekt: Konzepte und Zustand des Open-Source-Codes |
| [03-quote-verification.md](03-quote-verification.md) | Wie häufig Zitierfehler sind, und welche Prüfsysteme es gibt |
| [04-pdf-extraction.md](04-pdf-extraction.md) | **Eigene Messungen:** fünf Extraktoren im Vergleich, Lizenzen, Entscheidung |
| [05-data-source-apis.md](05-data-source-apis.md) | OpenAlex, Semantic Scholar (mit eigenem Test), S2ORC, SPECTER2, GROBID, OCR-Werkzeuge |
| [06-zotero.md](06-zotero.md) | Lokale API, Web API, Citation Keys, vorhandene Installation |
| [07-zed-typst.md](07-zed-typst.md) | Was Zed-Extensions können, Verhalten bei mehreren Language Servern, Typst-Bausteine |
| [08-rust-building-blocks.md](08-rust-building-blocks.md) | Crates mit Version, Pflegezustand und Lizenz |
| [09-hardware-local-llms.md](09-hardware-local-llms.md) | Laptop-Ausstattung und was lokal realistisch ist |
| [10-existing-workflow.md](10-existing-workflow.md) | Der heutige Arbeitsablauf im EHR-Seminar als Anforderungsquelle |
| [11-responsible-ai.md](11-responsible-ai.md) | Verantwortungsvolle KI-Nutzung als Designprinzip |
| [12-decisions.md](12-decisions.md) | Entscheidungsprotokoll der Brainstorming-Phase |
| [13-open-questions.md](13-open-questions.md) | Ungeprüfte Annahmen und offene Punkte |
| [probes/](probes/) | Die Testprogramme der PDF-Messungen, lauffähig |

## Wie zuverlässig ist das hier?

- **Gemessen:** alles in `04-pdf-extraction.md`, der Semantic-Scholar-Test in `05`, die
  Repo-Zustände in `02`/`08`, die Hardware in `09`, die Workflow-Beobachtungen in `10`.
- **Aus Quellen gelesen:** der Rest. Jede Aussage hat einen Link.
- **Nicht geprüft:** siehe `13-open-questions.md`.
