# ADR-0004: Separate diagnostics, reporting, and observability

Status: Accepted
Decision date: 2026-10-08
Supersedes: None

Provenance: Agreed with Anton during the color-eyre/miette source-walkthrough
discussion. Records the intended architecture, not completed implementation.

## Context

The CLI is a substantial user interface, not merely project setup: the design
includes checking quotes, inspecting citations, searching, and reviewing
findings. Source-oriented diagnostics will also serve the language server.
Both frontends need meaningful error data without parsing rendered messages.

Expected failures should be concise and actionable, with relevant paths and
suggestions. Developer detail should be optional. Scripts need separate JSON
output, and the CLI distinguishes success, diagnostic findings, and inability
to complete an operation.

At the time of this decision, core commonly returns `anyhow::Error`; the CLI
formats cause chains and downcasts selected Zotero errors. There is no
application-wide runtime logging setup. Existing sync history and planned AI
usage records are domain/audit data, not developer logs.

## Decision

### Core contracts and frontend presentation

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

### Runtime observability

- Use **`tracing`** for runtime events and operation spans. Libraries emit
  instrumentation; executables configure collection, filtering, and output.
- Ordinary CLI use remains quiet; developer logs are opt-in. Avoid logging the
  same propagated failure at every layer and again at the reporting boundary.
- CLI logs go to stderr or an explicitly configured file, never JSON stdout.
  LSP stdout is exclusively protocol traffic; developer logs go to stderr or
  a file, with selected user-facing messages sent through LSP notifications.
- Prefer identifiers and carefully selected fields. Do not dump document
  contents, excerpts, credentials, or full request payloads by default.
- Durable sync history and AI provenance remain application records,
  independent of runtime logging filters.
- Retain ordinary Rust panic reporting and optional backtraces. Do not add
  color-eyre as a second reporting stack solely for hypothetical debugging
  benefits. Add deeper instrumentation when a concrete debugging need warrants
  it, rather than capturing full traces everywhere.

### Extraction failure containment

Run crash-prone extraction behind a supervised process boundary, as already
required by the design spec. The parent detects abnormal termination, timeout,
or incomplete/invalid output, and reports failure without crashing itself.
It accepts only complete, validated results and owns their publication.

Provide bounded captured stderr, job/attachment and backend identity, backend
version, and termination information where available. Support cancellation,
reap terminated workers, and bound retries or fallback attempts. Worker
lifecycle and job granularity remain implementation design choices.

This is **basic crash containment, not a resource sandbox**. Resource limits,
guaranteed survival of system-wide exhaustion, and crash-dump infrastructure
are not requirements of this decision. Do not overengineer isolation or infer
an internal cause from termination alone: for example, `SIGKILL` does not
prove an out-of-memory kill.

## Alternatives considered

- **color-eyre as the primary reporter:** its standard layout, custom sections,
  and suggestions fit the desired style; no custom formatter would be needed
  merely for aesthetic precision. Its developer-reporting facilities are
  stronger, but it lacks miette's source-label model and excerpt renderer.
  The substantial CLI scope makes source diagnostics the stronger requirement.
- **Use both reporting stacks immediately:** adds integration and configuration
  costs without an established need. Rust already provides optional panic
  backtraces, and tracing addresses execution context independently.
- **Make miette the core diagnostic contract:** viable, but couples shared
  analysis to a reporting protocol. Renderer-neutral data better separates
  CLI, JSON, and LSP consumers, including future editor actions.
- **String-only errors:** lose reliable classification, causes, and source
  metadata. Erased containers such as `anyhow` can preserve typed causes and
  downcasting, but do not make operation-specific distinctions explicit in API
  signatures. Preserve existing typed downcasting during incremental migration;
  do not bridge reporting libraries by flattening errors into strings.
- **Use `catch_unwind` as crash isolation:** catches unwinding Rust panics, not
  aborts, segmentation faults, or OS kills. It is not a substitute for a
  process boundary around crash-prone backends.

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
- A trace captured only after a core error returns to the CLI cannot recover
  core frames that have already returned. Origin backtraces require capture
  at the origin. Span traces require instrumentation and are not chronological
  event logs. Neither replaces useful causes, context, or reproducible inputs.
- A crashing worker may produce no diagnostic output. Containment and a
  truthful failure report are required; explaining every crash is not.
- This ADR selects direction, not a rollout plan. Dependencies, diagnostic
  schemas, worker protocol, and logging configuration remain to be implemented.

## References

- [Design spec, §9–11, §13, and §15: LSP, CLI, acceptance, failure handling, and implementation order](../superpowers/specs/2026-09-16-library-core-typst-design.md)
- [Project idea and long-term capabilities](../../IDEA.md)
- [ADR-0003: Select PDF extraction backends](0003-select-pdf-extraction-backends.md)
- [thiserror: typed error implementations](https://docs.rs/thiserror/2/thiserror/)
- [miette 7.6.0: diagnostic protocol and rendering](https://docs.rs/miette/7.6.0/miette/)
- [miette 7.6.0 panic-hook implementation](https://docs.rs/crate/miette/7.6.0/source/src/panic.rs)
- [color-eyre 0.6.5: report sections, suggestions, and traces](https://docs.rs/color-eyre/0.6.5/color_eyre/)
- [tracing: events, spans, and subscribers](https://docs.rs/tracing/0.1/tracing/)
- [Rust's default panic hook](https://doc.rust-lang.org/std/panic/fn.set_hook.html)
- [Limits of catch_unwind](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html)
