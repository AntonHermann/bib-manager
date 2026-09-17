# Zed and Typst

Research 2026-09-14.

## What Zed extensions can do

Per the docs ([zed.dev/docs/extensions](https://zed.dev/docs/extensions)):

- Languages, **language servers**, themes, icon themes, debuggers, snippets, **MCP servers**.
- Extensions are Rust, compiled to WebAssembly, against versioned WIT interfaces.
- **No custom panels, webviews, or free-form UI elements.** Only discussions/RFCs exist for this (a declarative protocol, rendered natively via GPUI instead of a webview), nothing firmly planned.

Sources: [Extension Capabilities](https://zed.dev/docs/extensions/capabilities) · [RFC: Visual Extension API](https://github.com/zed-industries/zed/discussions/53403) · [Discussion: UI via GPUI](https://github.com/zed-industries/zed/discussions/48015) · [Issue: Webview via Extensions](https://github.com/zed-industries/zed/issues/21208) · [Discussion: Custom read-only previews](https://github.com/zed-industries/zed/discussions/59598) · [Discussion: Custom rendering](https://github.com/zed-industries/zed/discussions/37270)

## Multiple language servers for the same language

Relevant because tinymist already serves Typst and our server is meant to run alongside it.

| LSP feature | Behavior in Zed |
|---|---|
| Hover | queries all servers, shows combined answers |
| Definition, References | `request_multiple_lsp_locally()`: sent to all servers, results merged and deduplicated → **"Find All References" as a multibuffer works** |
| Diagnostics | from all servers |
| Document Highlights | **only the first capable server** ([Issue #61865](https://github.com/zed-industries/zed/issues/61865)) — irrelevant for us |

Order/priority via `languages.<language>.language_servers` in settings.

**Not practically tested**, derived only from docs, issues, and descriptions (see `13`).

Sources: [Configuring Languages](https://zed.dev/docs/configuring-languages) · [Discussion #24100](https://github.com/zed-industries/zed/discussions/24100) · [PR #23473](https://github.com/zed-industries/zed/pull/23473) · [Tracking Issue #10906](https://github.com/zed-industries/zed/issues/10906)

## Zed tasks

- `.zed/tasks.json`, variables including `ZED_FILE`, `ZED_ROW`, `ZED_COLUMN`, `ZED_SELECTED_TEXT`.
- Options such as `reveal_target: center`, `hide: on_success`, `save: current`.
- Already used in the EHR seminar (an fzf-based citation inserter, see `10`).
- A cheap way to hook CLI commands into the editor without an extension.

## MCP vs. CLI (discussion outcome)

| MCP server | CLI |
|---|---|
| agent knows functions automatically (name, description, parameters) | must be explained (rules file, `--help`) |
| finer-grained permissions without shell access | directly usable by humans, scripts, git hooks, Zed tasks |
| structured input/output | JSON output possible |
| running process keeps models in memory | costs no context until used |

Decision: **CLI first**, MCP later as a thin wrapper if it turns out to be missed.

## Typst

- Installed: `typst 0.15.0` (via cargo).
- Crate **`typst-syntax` 0.15.1** (official parser, Apache-2.0) matches the version.
- **tinymist** (Apache-2.0, active) is the standard language server; already completes citation keys.
- Alternative `tree-sitter-typst` (MIT, community, last push 2025-04-02) → not robust enough as a foundation.

Citation forms occurring in the EHR seminar:

| Form | Example |
|---|---|
| bare | `@jonnagaddala2025` (227 occurrences) |
| function | `#cite(<jonnagaddala2025>, form: "prose")` |
| quote with label attribution | `#quote(attribution: <dwork2006>)[…]` |
| block quote with content attribution | `#quote(block: true, attribution: [@yoon2023])[…]` |

Further quirks:
- Bibliography via `#bibliography("/ehr_privacy.bib", style: …)`; in chapters via a template function `#chapter-bib()` that produces a list only on standalone compilation (also within the language server).
- `#include` has its own scope → imports per chapter file.
- Slides with Touying 0.7.3, citing from the same `.bib`.
- A regex-based check script already reports cases it can't parse (`PARSER:` lines) → a real parser is needed.

Sources: [typst-syntax](https://crates.io/crates/typst-syntax) · [tinymist](https://github.com/Myriad-Dreamin/tinymist) · [tree-sitter-typst](https://github.com/uben0/tree-sitter-typst)
