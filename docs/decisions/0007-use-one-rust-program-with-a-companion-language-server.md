# ADR-0007: Use one Rust program with a companion language server

Status: Accepted
Decision date: 2026-10-10
Supersedes: None

Provenance: Retrospectively reconstructed from brainstorming decisions 5 and 13,
spec §§2, 4, and 9, and the 2026-09-17 Zed coexistence experiment. The
[brainstorming log](../research/12-decisions.md) records the choice of Rust and
`bib lsp` alongside tinymist during 2026-09-14–2026-09-16, without an individual
approval date. Anton accepted this record on 2026-10-10 during the documentation
review. That date records this ADR's approval; the September experiment supports
feasibility, not completion of the production integration.

## Context

The bibliography tool needs both commands usable from a shell and interactive
features while writing Typst in Zed. Both interfaces need the same library,
citation, and verification behavior. Keeping that logic independent of the
editor also makes it usable from scripts and tasks.

The author prefers Rust, and the
[brainstorming decision log](../research/12-decisions.md)
records that no component of the first version requires Python. The official
Typst parser, `typst-syntax`, is a Rust library. Meanwhile, tinymist already
provides general Typst language support; the bibliography tool needs to
complement it, not take over that responsibility.

Zed's language-server extension interface provides a way to add this specialized
functionality without requiring a custom editor UI. The
[step-0b experiment](../research/14-zed-two-language-servers.md) tested a separate
server alongside tinymist before relying on that integration.

## Decision

Implement the first version as one Rust workspace with shared core libraries
and a user-facing program, `bib`. Expose shell commands and the production
language-server mode through that program, with `bib lsp` running as the
language server. "One program" means one application entry point, not one
process for every concurrent use: CLI invocations and editor-launched servers
can run separately.

Use a small Zed extension to register and launch `bib lsp` for Typst alongside
tinymist. Keep bibliography behavior in the Rust libraries and language server,
not in the editor adapter. Tinymist remains responsible for general Typst
support; `bib lsp` supplies the bibliography-specific features in
[spec §9](../superpowers/specs/2026-09-16-library-core-typst-design.md#9-language-server-and-zed).

The first-version installation contract is to find `bib` on `PATH` or at a
configured path, with an installation hint when it is missing. Automatic
release downloads are later work, as described in
[spec §9](../superpowers/specs/2026-09-16-library-core-typst-design.md#9-language-server-and-zed).
The extension is a separate editor artifact; benchmark tools and experimental
programs are not additional application entry points for users.

Direct CLI/server access to SQLite and the absence of a daemon are already
decided in [ADR-0002](0002-use-central-sqlite-storage.md). This record neither
reopens that choice nor prevents supervised extraction workers under
[ADR-0004](0004-separate-diagnostics-reporting-and-observability.md).

## Alternatives considered

The [Zed research](../research/07-zed-typst.md#zed-tasks) identifies editor tasks
as a cheap way to invoke CLI commands. They remain useful, but do not provide
the continuously available hover, diagnostics, and navigation required by
[spec §9](../superpowers/specs/2026-09-16-library-core-typst-design.md#9-language-server-and-zed).

The [same research](../research/07-zed-typst.md#mcp-vs-cli-discussion-outcome)
compares MCP with a CLI and records CLI-first, with MCP a possible later wrapper.
Neither replaces the language-server integration for writing in the editor.

The sources record the Rust preference and lack of a Python requirement, but
do not preserve a systematic comparison of runtime architectures. They also do
not record a comparison with modifying or replacing tinymist. No such historical
evaluation is claimed here.

## Consequences

- Shell and editor features can reuse the same core behavior without separate
  implementations or an additional cross-language service boundary.
- Users must install both the program and the Zed extension for editor features.
  CLI use does not require the extension or a running editor.
- Coexistence does not guarantee that every response from both servers will
  merge as intended. The experiment used Zed 1.20.2 and tinymist 0.15.8 on
  Linux/Wayland: diagnostics, hover, and completion merged, but merging competing
  definition, reference, and code-action results was not tested. The sample
  also lacked a pinned main file; its results are not universal guarantees.
- Overlapping features need deliberate coordination. For example,
  [spec §9](../superpowers/specs/2026-09-16-library-core-typst-design.md#9-language-server-and-zed)
  allows the bibliography server's unknown-key diagnostic to be disabled because
  tinymist can report it too.
- The workspace and CLI exist; the coexistence server and extension are
  experiments, not the production `bib lsp` and Zed integration. Their
  implementation remains in later steps of
  [spec §15](../superpowers/specs/2026-09-16-library-core-typst-design.md#15-implementation-order).

## References

- [Spec §2: language and program choice](../superpowers/specs/2026-09-16-library-core-typst-design.md#2-decisions)
- [Spec §4: workspace responsibilities](../superpowers/specs/2026-09-16-library-core-typst-design.md#4-architecture)
- [Spec §9: language server and Zed behavior](../superpowers/specs/2026-09-16-library-core-typst-design.md#9-language-server-and-zed)
- [Historical brainstorming decisions 5 and 13](../research/12-decisions.md)
- [Zed and Typst research](../research/07-zed-typst.md)
- [Zed coexistence experiment and limitations](../research/14-zed-two-language-servers.md)
