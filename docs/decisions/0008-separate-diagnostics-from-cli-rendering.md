# ADR-0008: Separate diagnostic contracts from CLI rendering

Status: Accepted
Decision date: 2026-10-10
Supersedes: [ADR-0004](0004-separate-diagnostics-reporting-and-observability.md) (diagnostic contracts and presentation, as part of the replacement by ADRs 0008–0010)

Provenance: Extracted from ADR-0004, agreed with Anton on 2026-10-08.
Anton accepted this separate record on 2026-10-10. It preserves the original
decision; it neither selects new libraries nor claims completed implementation.
Together with
[ADR-0009](0009-use-tracing-for-runtime-observability.md) and
[ADR-0010](0010-isolate-crash-prone-extraction.md), it supersedes ADR-0004.
The complete replacement took effect on 2026-10-11 with acceptance of ADR-0010.

## Context

The CLI is a substantial user interface: checking quotes, inspecting citations,
searching, and reviewing findings. Source-oriented diagnostics also serve the
language server. Both frontends need meaningful error data without parsing
rendered messages.

Expected failures should be concise and actionable, with relevant paths and
suggestions. Developer detail is optional. Scripts need a separate JSON
contract, and the CLI distinguishes success, findings requiring attention,
and inability to complete an operation.

At the original decision, core commonly returned `anyhow::Error`; the CLI
formatted cause chains and downcast selected Zotero errors. Migration must
preserve those useful distinctions, not flatten them into strings.

Separating this record from runtime logging and extraction containment lets
the rendering choice evolve without reopening those independent decisions.
The split preserves the existing architecture.

## Decision

- Use explicit, operation-appropriate error types, with `thiserror` removing
  implementation boilerplate, where callers need meaningful distinctions.
  Preserve typed causes and structured context such as paths and identifiers.
  This is not a mechanical core-wide replacement of `anyhow`, nor a requirement
  for one exhaustive error enum or bespoke types for every internal helper.
- Keep findings separate from operation failure. A completed check can return
  findings with error severity; that does not itself make the operation a Rust
  `Err`. An operation failure can also carry diagnostics, such as configuration
  validation problems. Each API defines what completing its operation means.
- Keep core errors and findings renderer-neutral. Preserve source identity and
  ranges where available, so CLI and LSP adapters can reuse the same data.
  Locations must correspond to the analyzed source snapshot; editor buffers
  need not match disk contents.
- Core owns canonical messages (`Display`/`#[error(...)]` for errors), causes,
  and diagnostic meaning. Frontends reuse those messages and add presentation
  and frontend-specific recovery actions rather than duplicating error text.
- Choose **miette for human-facing CLI diagnostics**, including long-term
  labelled excerpts in configuration, bibliography, and Typst files. Its
  diagnostic types do not define the core contract. Ordinary command results
  and review lists need not use diagnostic rendering.
- Keep **JSON a separate application contract**, not serialized terminal
  reports. Keep explicit exit codes: `0` success, `1` completed with diagnostic
  findings requiring attention, `2` failure to complete. Clap may continue to
  handle parsing failures itself without JSON output.

Runtime instrumentation and debugging facilities are covered by
[ADR-0009](0009-use-tracing-for-runtime-observability.md), not by the choice
of terminal renderer.

## Alternatives considered

These alternatives and their rationale come from
[ADR-0004](0004-separate-diagnostics-reporting-and-observability.md#alternatives-considered):

- **color-eyre as the primary reporter:** its standard layout, custom sections,
  and suggestions fit the desired style; no custom formatter would be needed
  merely for aesthetic precision. Its developer-reporting facilities are
  stronger, but it lacks miette's source-label model and excerpt renderer.
  The substantial CLI scope makes source diagnostics the stronger requirement.
- **Make miette the core diagnostic contract:** viable, but couples shared
  analysis to a reporting protocol. Renderer-neutral data better separates
  CLI, JSON, and LSP consumers, including future editor actions.
- **String-only errors:** lose reliable classification, causes, and source
  metadata. Erased containers such as `anyhow` can preserve typed causes and
  downcasting, but do not make operation-specific distinctions explicit in API
  signatures. Preserve existing typed downcasting during incremental migration;
  do not bridge reporting libraries by flattening errors into strings.

Adding both reporting stacks immediately is addressed in
[ADR-0009](0009-use-tracing-for-runtime-observability.md#alternatives-considered).

## Consequences

- Core diagnostic data is reusable across CLI and LSP, while excerpts and
  editor locations require accurate producer-side ranges and source snapshots.
  Suggestions are not automatically executable editor fixes.
- Adapters and focused error types require work, but keep classification,
  canonical messages, presentation, and exit policy independently testable.
  Test semantic data and preserved causes separately from a small set of
  deterministic rendering tests and CLI JSON/exit-code checks.
- Miette's diagnostic rendering does not provide color-eyre-equivalent
  debugging automatically. In the inspected miette 7.6.0 implementation,
  backtrace machinery serves the optional panic hook; ordinary returned
  reports do not automatically capture/render backtraces through the standard
  handler in the same way as color-eyre.
- This selects direction, not a rollout plan. Dependencies, diagnostic schemas,
  and frontend adapters remain implementation work.

## References

- [ADR-0004: original decision and provenance](0004-separate-diagnostics-reporting-and-observability.md)
- [Spec §9: language server](../superpowers/specs/2026-09-16-library-core-typst-design.md#9-language-server-and-zed)
- [Spec §10: CLI](../superpowers/specs/2026-09-16-library-core-typst-design.md#10-cli)
- [Spec §13: failure behavior](../superpowers/specs/2026-09-16-library-core-typst-design.md#13-error-handling)
- [Project idea and long-term capabilities](../../IDEA.md)
- [thiserror: typed error implementations](https://docs.rs/thiserror/2/thiserror/)
- [miette 7.6.0: diagnostic protocol and rendering](https://docs.rs/miette/7.6.0/miette/)
- [miette 7.6.0 panic-hook implementation](https://docs.rs/crate/miette/7.6.0/source/src/panic.rs)
- [color-eyre 0.6.5: report sections, suggestions, and traces](https://docs.rs/color-eyre/0.6.5/color_eyre/)
