# ADR-0009: Use tracing for runtime observability

Status: Accepted
Decision date: 2026-10-10
Supersedes: [ADR-0004](0004-separate-diagnostics-reporting-and-observability.md) (runtime observability, as part of the replacement by ADRs 0008–0010)

Provenance: Extracted from ADR-0004, agreed with Anton on 2026-10-08.
Anton accepted this separate record on 2026-10-10. It preserves the logging
and debugging decisions while allowing them to evolve independently of the
terminal renderer and extraction isolation.
Together with [ADR-0008](0008-separate-diagnostics-from-cli-rendering.md) and
[ADR-0010](0010-isolate-crash-prone-extraction.md), it supersedes ADR-0004.
The complete replacement took effect on 2026-10-11 with acceptance of ADR-0010.

## Context

Developer instrumentation explains runtime activity; human-facing diagnostics
explain findings and failed operations. Neither replaces durable application
records such as sync history or AI provenance.

At the original decision there was no application-wide runtime logging setup.
The tool needs useful execution context without making ordinary CLI use noisy,
corrupting JSON or LSP output, or unnecessarily exposing document contents.
Installing another reporting library alone would not recover execution context
that was never captured.

## Decision

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
  independent of runtime logging filters. In particular, opt-in developer
  logging does not make the AI call log optional under
  [ADR-0006](0006-enforce-responsible-ai-invariants.md).
- Retain ordinary Rust panic reporting and optional backtraces. Do not add
  color-eyre as a second reporting stack solely for hypothetical debugging
  benefits. Add deeper instrumentation when a concrete debugging need warrants
  it, rather than capturing full traces everywhere.

## Alternatives considered

[ADR-0004](0004-separate-diagnostics-reporting-and-observability.md#alternatives-considered)
considered using both miette and color-eyre immediately. This adds integration
and configuration costs without an established need: Rust already provides
optional panic backtraces, and tracing addresses execution context independently.
The primary human-facing reporter choice is preserved separately in
[ADR-0008](0008-separate-diagnostics-from-cli-rendering.md).

The original record does not document a systematic comparison of `tracing`
against other logging libraries. This split does not invent one.

## Consequences

- Developer events and spans can be enabled without changing application output
  or the retention of domain/audit records.
- Libraries require instrumentation and executables require collection,
  filtering, and output configuration; adopting a dependency alone is not
  a working observability system.
- A trace captured only after a core error returns to the CLI cannot recover
  core frames that have already returned. Origin backtraces require capture
  at the origin. Span traces require instrumentation and are not chronological
  event logs. Neither replaces useful causes, context, or reproducible inputs.
- The miette-specific reporting limitations are documented in
  [ADR-0008](0008-separate-diagnostics-from-cli-rendering.md#consequences).
  This decision does not promise color-eyre-equivalent debugging from miette.
- This selects direction, not a rollout plan. Logging configuration and any
  deeper instrumentation remain implementation work.

## References

- [ADR-0004: original decision and provenance](0004-separate-diagnostics-reporting-and-observability.md)
- [Spec §9: language server](../superpowers/specs/2026-09-16-library-core-typst-design.md#9-language-server-and-zed)
- [Spec §10: CLI](../superpowers/specs/2026-09-16-library-core-typst-design.md#10-cli)
- [tracing: events, spans, and subscribers](https://docs.rs/tracing/0.1/tracing/)
- [Rust's default panic hook](https://doc.rust-lang.org/std/panic/fn.set_hook.html)
- [color-eyre 0.6.5: report sections, suggestions, and traces](https://docs.rs/color-eyre/0.6.5/color_eyre/)
