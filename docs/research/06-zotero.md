# Zotero-Anbindung

Recherche 2026-09-14.

## Vorhandene Installation (gemessen)

- `zotero-snap` **9.0.1** (Snap, Rev. 128, Kanal latest/stable).
- Datenverzeichnis: `~/snap/zotero-snap/common/Zotero/`.
- Better BibTeX vorhanden; Datei `better-bibtex.migrated` → Citation Keys sind bereits in Zoteros eingebautes Feld migriert.
- Zotero Connector im Browser wird aktiv genutzt; Tablet-App mit Sync ebenfalls.

## Lokale API (Zotero 7+)

- `http://localhost:23119/api/` — lokale Implementierung der Web API v3 auf der lokalen Datenbank.
- Offline, **ohne Rate Limits**, schneller als die Web API.
- **Nur lesend.** Annotationen können gelesen werden.
- Aktivieren: Einstellungen → Erweitert → „Allow other applications on this computer to communicate with Zotero".
- Drittanbieter-Add-on `zotero-local-write-api` registriert Schreib-Endpunkte (Items, Notizen, Anhänge, Collections, Tags) auf demselben Server.

Quellen: [Zotero Local API](https://www.zotero.org/support/dev/web_api/v3/local_api) · [Web API Basics](https://www.zotero.org/support/dev/web_api/v3/basics) · [zotero-local-write-api](https://github.com/dzackgarza/zotero-local-write-api) · [zotero-dev: Local API Write access](https://groups.google.com/g/zotero-dev/c/xiUvjyYkQk4) · [Forum: pyzotero mit lokaler API](https://forums.zotero.org/discussion/116548/how-to-use-pyzotero-to-access-zotero-7-beta-local-api-server)

## Annotationen

Felder eines Annotation-Items:

- `annotationType` (`highlight`, `note`, `text`, …), `annotationText`, `annotationComment`, `annotationColor`, `annotationPageLabel`
- `annotationSortIndex`: Format `PPPPP|TTTTTT|OOOOO` = Seitenindex (5 Stellen) | Zeichenoffset (6) | y-Offset (5)
- `annotationPosition`: z. B. `{"pageIndex":8,"rects":[[68.604,246.586,174.372,267.047]]}`

Folgen:
- Zotero speichert **nur Seite + Rechtecke**, keinen robusten Textanker. Beim Import muss das Tool den Text an der Stelle samt Kontext aus der eigenen Textschicht lesen und einen Anker bauen; die Zotero-ID bleibt für spätere Abgleiche gespeichert.
- Anlegen per **Web API** (online, API-Key): POST auf `/users/{id}/items`, bis zu 50 Annotationen pro Request. In der Praxis genutzt, aber kaum offiziell dokumentiert; im Forum gibt es einen offenen Feature-Request für eine offizielle API.

Quellen: [Forum: Feature Request programmatische Annotationen](https://forums.zotero.org/discussion/132273/feature-request-expose-api-for-programmatic-pdf-annotation-creation-highlights-notes) · [Zoteus MCP: zotero_annotate](https://glama.ai/mcp/servers/oscardvs/zoteus/tools/zotero_annotate) · [zoteroRoam: Annotationen formatieren](https://alix-lahuec.gitbook.io/zotero-roam/customization/annotations)

## Citation Keys

- Zotero 8 hat ein **natives Citation-Key-Feld**; es ersetzt das Legacy-Feld von Better BibTeX.
- BBT migriert still, wenn noch keine nativen Keys existieren; sonst Dialog zur Auswahl.
- Vorteil: Keys werden jetzt mit synchronisiert.
- Es gab Probleme im Übergang (Zettlr-Beitrag, Forum-Threads zu fehlendem Feld im Info-Bereich und Dateinamen-Templates).

Quellen: [BBT Changelog](https://retorque.re/zotero-better-bibtex/changelog/index.html) · [BBT Discussion #3404](https://github.com/retorquere/zotero-better-bibtex/discussions/3404) · [Zettlr: Zotero 8 und Better BibTeX](https://www.zettlr.com/post/on-the-recent-issues-with-zotero-8-and-better-bibtex) · [Better BibTeX](https://retorque.re/zotero-better-bibtex/)

## Typst und Zotero

- Typst liest `.bib` (BibTeX/BibLaTeX) und das eigene YAML-Format Hayagriva.
- Zitierstile als CSL; Stile aus dem Zotero Style Repository funktionieren direkt.

Quellen: [Typst: Guide for LaTeX users](https://typst.app/docs/guides/for-latex-users/) · [TypeTeX: Typst Bibliography](https://www.typetex.app/guides/typst-bibliography)

## Entscheidungen (siehe `12`)

- Zotero bleibt Eingang (Connector) und mobiler Begleiter (Tablet-Sync).
- Das Tool liest vorerst **nur**; Zotero-IDs werden gespeichert, damit Zurückschreiben später ohne Umbau möglich ist.
- Typisierte Notizen und Links existieren nur im Tool; sie müssen nicht aufs Tablet.
- Die `.bib` ist ein Zotero-Export und wird vom Tool nicht umgeschrieben.
