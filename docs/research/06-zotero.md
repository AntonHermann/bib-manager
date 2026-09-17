# Zotero integration

Research 2026-09-14.

## Existing installation (measured)

- `zotero-snap` **9.0.1** (Snap, rev. 128, latest/stable channel).
- Data directory: `~/snap/zotero-snap/common/Zotero/`.
- Better BibTeX present; file `better-bibtex.migrated` → citation keys have already been migrated into Zotero's built-in field.
- Zotero Connector actively used in the browser; tablet app with sync also in use.

## Local API (Zotero 7+)

- `http://localhost:23119/api/` — local implementation of the Web API v3 against the local database.
- Offline, **no rate limits**, faster than the Web API.
- **Read-only.** Annotations can be read.
- Enable via: Settings → Advanced → "Allow other applications on this computer to communicate with Zotero."
- Third-party add-on `zotero-local-write-api` registers write endpoints (items, notes, attachments, collections, tags) on the same server.

Sources: [Zotero Local API](https://www.zotero.org/support/dev/web_api/v3/local_api) · [Web API Basics](https://www.zotero.org/support/dev/web_api/v3/basics) · [zotero-local-write-api](https://github.com/dzackgarza/zotero-local-write-api) · [zotero-dev: Local API Write access](https://groups.google.com/g/zotero-dev/c/xiUvjyYkQk4) · [Forum: pyzotero with local API](https://forums.zotero.org/discussion/116548/how-to-use-pyzotero-to-access-zotero-7-beta-local-api-server)

## Annotations

Fields of an annotation item:

- `annotationType` (`highlight`, `note`, `text`, …), `annotationText`, `annotationComment`, `annotationColor`, `annotationPageLabel`
- `annotationSortIndex`: format `PPPPP|TTTTTT|OOOOO` = page index (5 digits) | character offset (6) | y-offset (5)
- `annotationPosition`: e.g. `{"pageIndex":8,"rects":[[68.604,246.586,174.372,267.047]]}`

Consequences:
- Zotero stores **only page + rectangles**, not a robust text anchor. On import, the tool must read the text at that spot plus context from its own text layer and build an anchor; the Zotero ID is kept for later syncing.
- Creating via **Web API** (online, API key): POST to `/users/{id}/items`, up to 50 annotations per request. Used in practice but barely officially documented; there's an open feature request in the forum for an official API.

Sources: [Forum: Feature Request programmatic annotations](https://forums.zotero.org/discussion/132273/feature-request-expose-api-for-programmatic-pdf-annotation-creation-highlights-notes) · [Zoteus MCP: zotero_annotate](https://glama.ai/mcp/servers/oscardvs/zoteus/tools/zotero_annotate) · [zoteroRoam: formatting annotations](https://alix-lahuec.gitbook.io/zotero-roam/customization/annotations)

## Citation keys

- Zotero 8 has a **native citation key field**; it replaces Better BibTeX's legacy field.
- BBT migrates silently if no native keys exist yet; otherwise it shows a dialog to choose.
- Advantage: keys are now synced as well.
- There were transition issues (a Zettlr post, forum threads about a missing field in the info panel and filename templates).

Sources: [BBT Changelog](https://retorque.re/zotero-better-bibtex/changelog/index.html) · [BBT Discussion #3404](https://github.com/retorquere/zotero-better-bibtex/discussions/3404) · [Zettlr: Zotero 8 and Better BibTeX](https://www.zettlr.com/post/on-the-recent-issues-with-zotero-8-and-better-bibtex) · [Better BibTeX](https://retorque.re/zotero-better-bibtex/)

## Typst and Zotero

- Typst reads `.bib` (BibTeX/BibLaTeX) and its own YAML format, Hayagriva.
- Citation styles as CSL; styles from the Zotero Style Repository work directly.

Sources: [Typst: Guide for LaTeX users](https://typst.app/docs/guides/for-latex-users/) · [TypeTeX: Typst Bibliography](https://www.typetex.app/guides/typst-bibliography)

## Decisions (see `12`)

- Zotero remains the entry point (connector) and mobile companion (tablet sync).
- The tool reads **only, for now**; Zotero IDs are stored so that writing back can be added later without rework.
- Typed notes and links exist only in the tool; they don't need to reach the tablet.
- The `.bib` is a Zotero export and is not rewritten by the tool.
