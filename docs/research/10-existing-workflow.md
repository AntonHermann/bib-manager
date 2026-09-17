# Bestehender Arbeitsablauf: EHR-Seminar als Anforderungsquelle

Beobachtet am 2026-09-14 durch reines Lesen von `~/Documents/seminar_ehr_ss26`.
Nur Struktur und Werkzeuge sind hier festgehalten, keine Inhalte der Arbeit.

**Wichtig:** Das Projekt ist eine benotete Arbeit mit eigenen KI-Regeln (Abgabe 2026-09-23).
Bis dahin wird es aus diesem Projekt heraus nicht mehr gelesen oder getestet.
Erst danach wird es Referenzprojekt für die Abnahme der ersten Version.

## Projektaufbau

- Typst-Arbeit: `paper.typ`, `template.typ`, `chapters/*.typ` (eine Datei pro Abschnitt), daneben `chapters/*.md` als Belegsammlungen vor dem Schreiben.
- Folien: `presentation/slides.typ` (Touying), gleiche `.bib`.
- `ehr_privacy.bib`: Zotero-Export, 56 Einträge. **Darf von Werkzeugen nicht umgeschrieben werden.**
- `literatur/`: 65 PDFs + 65 `pdftotext`-Volltexte (3,2 MB, versioniert), sortiert nach Triage-Stufen.
- Obsidian-Vault im selben Ordner (Plugin `obsidian-pandoc-reference-list`), Pandoc-CSL und Zotero-JSON-Exporte unter `.pandoc/`.
- `Justfile` und `.zed/tasks.json` für Befehle.

## Werkzeuge, die das Tool ersetzen soll

| Heute | Zweck | Im Tool |
|---|---|---|
| `notes/quote_verification.json`: 143 Einträge `{key, label, quote}`, optional `validated: <session>` | Belegstellen-Sammlung mit Kapitel-Label | Belegstelle (global, verankert) + Verwendung (Projekt, Label) |
| `notes/check_quotes.py` (91 Zeilen) | prüft Wortlaut gegen Volltexte; feste Tabelle Key → Textdatei; Normalisierung: NFKC, weiche Trennstriche, Silbentrennung am Zeilenende, Striche/Anführungszeichen vereinheitlichen, Leerraum, Kleinschreibung | automatische Prüfung für alle Quellen, Pfade aus Zotero, Treffer mit Seite |
| `notes/check_slide_quotes.py` (210 Zeilen), `just pres-check-quotes` | Kette Folie → Sammlung → Quelle; `PARSER:`-Zeilen bei nicht erkannten `#quote`-Formen | Diagnostics im Editor, echter Typst-Parser |
| `notes/quote_inserter.sh` (Zed-Task, `jq` + `fzf`, schreibt per `awk` an `ZED_ROW`/`ZED_COLUMN`) | Belegstelle als `#quote(attribution: <key>)[…]` einfügen | Vervollständigung / Code Action |
| `just pres-all-used-citations` (`rg`) | verwendete Keys eines Dokuments | „Find All References" |
| `grep` über `literatur/text/` | Volltextsuche | Suchindex (Teilprojekt 3) |
| Ordner `A_kern` / `B_belege` / `C_rest` + Rollen (Hauptquelle, backward, forward, Reserve, aussortiert) | Triage | Status und Rolle pro Quelle und Projekt |
| `notes/Quellenauswahl.md`: Backward-Referenzen der Hauptquelle, Forward-Recherche über Semantic Scholar (Zitat-Sätze durchgesehen), Co-Autor-Checks, Selbstzitationen | Zitationsgraph von Hand | Teilprojekt 5 |
| `literatur/zotero_korrekturen.md`: Formalcheck (Venue/Volume/Issue vollständig? begutachtete Fassung statt arXiv?) | Metadatenqualität | mögliche Metadaten-Prüfung |
| `tool_usage/`, `jsonl2md.sh`, Anhang-Tabelle | Dokumentation der KI-Nutzung | Nutzungsprotokoll + Anhang-Generator (siehe `11`) |

## Dokumentierte Fallen (aus `literatur/README.md`)

- Tabellen ohne `-layout` zerlegt → falsche Zahl in Notizen.
- ε fehlt in mehreren DP-Volltexten; Zitate mit ε nur aus dem PDF übernommen.
- Mathematisch-kursives `𝜖` in einem Buch; Suche braucht `ε|𝜖`.
- Einige Quellen nicht aus Zotero (über Cited-by-Recherche dazugekommen), vier Zotero-Einträge ohne PDF, ein Duplikat.

## Arbeitsgewohnheiten, die das Design beeinflussen

- **Klartext in Git** für Volltexte und Zitatsammlung, damit Claude in Web-Sessions darauf zugreifen kann → Export-Befehl im CLI nötig, obwohl die Daten zentral liegen.
- Zitate werden **wörtlich** und mit Prüfkette verwendet; manuelle Bestätigung wird im Datensatz vermerkt (`validated`).
- Notizen zu Quellen auch direkt in Zotero (z. B. „earlier version of …").
- Zed als Editor, Tasks statt Extensions.

## Folgerungen für den Import (Abnahmetest)

- Die 143 Einträge haben **keinen Kontext und keine Position**; Anker müssen durch Suche in der neuen Textschicht erzeugt werden.
- Diese Textschicht ist nicht die, gegen die geprüft wurde: Zitate mit ε passen evtl. jetzt, andere evtl. nicht mehr.
- Einträge mit `validated` wurden nie maschinell gefunden.
- → Herkunft **pro Eintrag** vergeben (deterministisch geprüft / vom Nutzer bestätigt, nicht maschinell verankert / ungeklärt) und nicht auflösbare Anker in eine Prüfliste. Ohne Git-Diff ist die Prüfliste das einzige Sicherheitsnetz.
