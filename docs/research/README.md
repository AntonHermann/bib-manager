# Recherche zum Bibliography Manager

Stand: 2026-09-14 bis 2026-09-16. Alle Dateien hier sind Rechercheergebnisse aus der
Brainstorming-Phase, keine Spezifikation. Die Spec entsteht separat.

## Inhalt

| Datei | Thema |
|---|---|
| [01-werkzeuglandschaft.md](01-werkzeuglandschaft.md) | Bestehende Tools: Zitationsgraphen, Smart Citations, Extraktions-Matrizen, Canvas-Werkzeuge |
| [02-semantic-reader.md](02-semantic-reader.md) | Semantic-Reader-Projekt: Konzepte und Zustand des Open-Source-Codes |
| [03-zitatpruefung.md](03-zitatpruefung.md) | Wie häufig Zitierfehler sind, und welche Prüfsysteme es gibt |
| [04-pdf-extraktion.md](04-pdf-extraktion.md) | **Eigene Messungen:** fünf Extraktoren im Vergleich, Lizenzen, Entscheidung |
| [05-datenquellen-apis.md](05-datenquellen-apis.md) | OpenAlex, Semantic Scholar (mit eigenem Test), S2ORC, SPECTER2, GROBID, OCR-Werkzeuge |
| [06-zotero.md](06-zotero.md) | Lokale API, Web API, Citation Keys, vorhandene Installation |
| [07-zed-typst.md](07-zed-typst.md) | Was Zed-Extensions können, Verhalten bei mehreren Language Servern, Typst-Bausteine |
| [08-rust-bausteine.md](08-rust-bausteine.md) | Crates mit Version, Pflegezustand und Lizenz |
| [09-hardware-lokale-llms.md](09-hardware-lokale-llms.md) | Laptop-Ausstattung und was lokal realistisch ist |
| [10-bestehender-workflow.md](10-bestehender-workflow.md) | Der heutige Arbeitsablauf im EHR-Seminar als Anforderungsquelle |
| [11-responsible-ai.md](11-responsible-ai.md) | Verantwortungsvolle KI-Nutzung als Designprinzip |
| [12-entscheidungen.md](12-entscheidungen.md) | Entscheidungsprotokoll der Brainstorming-Phase |
| [13-offene-fragen.md](13-offene-fragen.md) | Ungeprüfte Annahmen und offene Punkte |
| [probes/](probes/) | Die Testprogramme der PDF-Messungen, lauffähig |

## Wie zuverlässig ist das hier?

- **Gemessen:** alles in `04-pdf-extraktion.md`, der Semantic-Scholar-Test in `05`, die
  Repo-Zustände in `02`/`08`, die Hardware in `09`, die Workflow-Beobachtungen in `10`.
- **Aus Quellen gelesen:** der Rest. Jede Aussage hat einen Link.
- **Nicht geprüft:** siehe `13-offene-fragen.md`.
