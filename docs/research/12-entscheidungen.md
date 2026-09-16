# Entscheidungsprotokoll (Brainstorming)

Alle Punkte vom Nutzer entschieden oder bestätigt, 2026-09-14 bis 2026-09-16.

| # | Frage | Entscheidung | Begründung / Kontext |
|---|---|---|---|
| 1 | Schreibwerkzeug | **Typst** (Paper + Touying-Folien) | Zitate im Klartext, Kapitel als Überschriften erkennbar |
| 2 | Rolle von Zotero | **zwischen „ersetzen" und „Hybrid"**: Zotero bleibt Eingang (Connector) und mobiler Begleiter (Tablet-Sync); Tool ist Arbeitsplatz | Zotero komplett ersetzen zu aufwendig, Zotero-Datenmodell zu einschränkend |
| 3 | Zurückschreiben nach Zotero | **erst nur lesen**, Zotero-IDs speichern, Zurückschreiben später nachrüstbar | typisierte Notizen müssen nicht aufs Tablet |
| 4 | Fachgebiet und Größe | **Informatik**, etwas Medizin; **einige Hundert** Einträge | OpenAlex/Semantic Scholar decken das gut ab; alles lokal machbar |
| 5 | Editor | **Zed** | Language Server + CLI + eigene Oberfläche für Visuelles |
| 6 | MCP-Server | **nicht fest eingeplant**; CLI zuerst, MCP später als Hülle | Vorteil gegenüber CLI klein, kostet Kontext |
| 7 | LLM-Ausführung | **wählbar pro Aufgabe**, so viel lokal wie möglich | iGPU-Laptop, siehe `09` |
| 8 | Erste nutzbare Version | **Kern + Typst-Anbindung** (Option a) | direkter Nutzen beim Schreiben; liefert Satz-Extraktion für spätere Prüfung |
| 9 | Zuschnitt erste Version | Kern (Zotero lesen, Textschicht, Belegstellen mit Ankern, CLI) + Language Server (Hover, References, Wortlaut-Prüfung, Einfügen); **GROBID nicht**, Schichtenmodell aber vorbereitet | nach Semantic-Reader-Recherche |
| 10 | Datenablage | **zentrale Datenbank** (Option b) | Folgen: Backup, Export-Befehl, Projekt-Markierung im Repo |
| 11 | Referenzprojekt | EHR-Seminar **erst nach Abgabe (2026-09-23)**; bis dahin eigenes Testprojekt mit Open-Access-Papers | benotete Arbeit, KI-Regeln |
| 12 | Responsible AI | **Designprinzip von Anfang an**, siehe `11` | Nutzerwunsch |
| 13 | Programmiersprache | **Rust** (Ansatz A: ein Programm `bib`, CLI + `bib lsp`, Zed-Extension startet es neben tinymist; kein Daemon in v1, SQLite WAL) | Nutzer bevorzugt Rust; kein v1-Baustein braucht Python |
| 14 | Lizenz | **MIT OR Apache-2.0** | breite Wirkung für die Wissenschaft wichtiger als Copyleft |
| 15 | PDF-Extraktion | **`pdf_oxide` + eigener Reparaturschritt**, optional `mutool` als externer Prozess, **Benchmark als erster Planschritt** | Messungen in `04` |

## Vorgeschlagene Aufteilung in Teilprojekte

1. **Kern:** Datenmodell mit robusten Ankern, Zotero-Import, Textextraktion, CLI.
2. **Typst-Anbindung:** Parser (Zitate, Sätze, Kapitel), Language Server.
3. **Suche:** hybride Suche über Volltexte, Ergebnisse als Ausschnitte.
4. **Lesen und Notizen:** PDF-Leser mit Ankern, drei Notiz-Ebenen, typisierte Notizen/Links, Gruppen (eigene Oberfläche).
5. **Graph und Anreicherung:** OpenAlex/Semantic Scholar, Zitationsgraph, Co-Autoren, Keywords, Retraction-Check, Lücken.
6. **Zitat-Prüfung:** Satz ↔ Quellenstelle, Status, Bericht (braucht 1, 2, 3).
7. **Später:** Extraktions-Matrix, Canvas, MCP, Zurückschreiben nach Zotero.

Erste Version = 1 + 2.

## Stand des Designs

Das Design wurde in einer parallelen Session ausgearbeitet und liegt als Spec vor: [`2026-09-16-bibliothek-kern-typst-design.md`](../superpowers/specs/2026-09-16-bibliothek-kern-typst-design.md).
Diese Recherche-Dateien stammen aus einem Fork der Brainstorming-Session und spiegeln den Stand vor der Spec.
