# Zed und Typst

Recherche 2026-09-14.

## Was Zed-Extensions können

Laut Doku ([zed.dev/docs/extensions](https://zed.dev/docs/extensions)):

- Languages, **Language Server**, Themes, Icon Themes, Debugger, Snippets, **MCP-Server**.
- Extensions sind Rust, kompiliert zu WebAssembly, gegen versionierte WIT-Schnittstellen.
- **Keine eigenen Panels, Webviews oder freien UI-Elemente.** Dazu gibt es nur Diskussionen/RFCs (deklaratives Protokoll, nativ über GPUI gerendert statt Webview), nichts fest eingeplant.

Quellen: [Extension Capabilities](https://zed.dev/docs/extensions/capabilities) · [RFC: Visual Extension API](https://github.com/zed-industries/zed/discussions/53403) · [Discussion: UI via GPUI](https://github.com/zed-industries/zed/discussions/48015) · [Issue: Webview via Extensions](https://github.com/zed-industries/zed/issues/21208) · [Discussion: Custom read-only previews](https://github.com/zed-industries/zed/discussions/59598) · [Discussion: Custom rendering](https://github.com/zed-industries/zed/discussions/37270)

## Mehrere Language Server für dieselbe Sprache

Relevant, weil tinymist Typst schon bedient und unser Server daneben laufen soll.

| LSP-Funktion | Verhalten in Zed |
|---|---|
| Hover | fragt alle Server, zeigt Antworten kombiniert |
| Definition, References | `request_multiple_lsp_locally()`: an alle Server, Ergebnisse zusammengeführt und dedupliziert → **„Find All References" als Multibuffer funktioniert** |
| Diagnostics | von allen Servern |
| Document Highlights | **nur der erste fähige Server** ([Issue #61865](https://github.com/zed-industries/zed/issues/61865)) — für uns irrelevant |

Reihenfolge/Priorität über `languages.<Sprache>.language_servers` in den Settings.

**Nicht praktisch getestet**, nur aus Doku, Issue und Beschreibungen abgeleitet (siehe `13`).

Quellen: [Configuring Languages](https://zed.dev/docs/configuring-languages) · [Discussion #24100](https://github.com/zed-industries/zed/discussions/24100) · [PR #23473](https://github.com/zed-industries/zed/pull/23473) · [Tracking Issue #10906](https://github.com/zed-industries/zed/issues/10906)

## Zed-Tasks

- `.zed/tasks.json`, Variablen u. a. `ZED_FILE`, `ZED_ROW`, `ZED_COLUMN`, `ZED_SELECTED_TEXT`.
- Optionen wie `reveal_target: center`, `hide: on_success`, `save: current`.
- Im EHR-Seminar schon genutzt (fzf-basierter Zitat-Einfüger, siehe `10`).
- Günstiger Weg, CLI-Befehle ohne Extension an den Editor zu hängen.

## MCP vs. CLI (Diskussionsergebnis)

| MCP-Server | CLI |
|---|---|
| Agent kennt Funktionen automatisch (Name, Beschreibung, Parameter) | muss erklärt werden (Regeldatei, `--help`) |
| feinere Rechte ohne Shell-Freigabe | für Menschen direkt nutzbar, Skripte, Git-Hooks, Zed-Tasks |
| strukturierte Ein-/Ausgabe | JSON-Ausgabe möglich |
| laufender Prozess hält Modelle im Speicher | kostet keinen Kontext, bis es genutzt wird |

Entscheidung: **CLI zuerst**, MCP später als dünne Hülle, falls vermisst.

## Typst

- Installiert: `typst 0.15.0` (über cargo).
- Crate **`typst-syntax` 0.15.1** (offizieller Parser, Apache-2.0) passt zur Version.
- **tinymist** (Apache-2.0, aktiv) ist der gängige Language Server; vervollständigt Zitat-Keys schon.
- Alternative `tree-sitter-typst` (MIT, Community, letzter Push 2025-04-02) → nicht robust genug als Grundlage.

Zitatformen, die im EHR-Seminar vorkommen:

| Form | Beispiel |
|---|---|
| nackt | `@jonnagaddala2025` (227 Vorkommen) |
| Funktion | `#cite(<jonnagaddala2025>, form: "prose")` |
| Zitat mit Label-Attribution | `#quote(attribution: <dwork2006>)[…]` |
| Blockzitat mit Content-Attribution | `#quote(block: true, attribution: [@yoon2023])[…]` |

Weitere Eigenheiten:
- Literaturverzeichnis per `#bibliography("/ehr_privacy.bib", style: …)`; in Kapiteln über eine Template-Funktion `#chapter-bib()`, die nur bei Einzelkompilierung (auch im Language Server) ein Verzeichnis erzeugt.
- `#include` hat eigenen Scope → Imports pro Kapiteldatei.
- Folien mit Touying 0.7.3, zitiert aus derselben `.bib`.
- Ein Regex-basiertes Prüfskript meldet schon Fälle, die es nicht parsen kann (`PARSER:`-Zeilen) → echter Parser nötig.

Quellen: [typst-syntax](https://crates.io/crates/typst-syntax) · [tinymist](https://github.com/Myriad-Dreamin/tinymist) · [tree-sitter-typst](https://github.com/uben0/tree-sitter-typst)
