# ADR-0010: Defer extraction process isolation

Status: Accepted
Decision date: 2026-10-11
Supersedes: [ADR-0004](0004-separate-diagnostics-reporting-and-observability.md) (extraction failure containment, as part of the replacement by ADRs 0008–0010)

Provenance: Initially drafted as a split of ADR-0004, agreed with Anton on
2026-10-08. During review of this draft, Anton excluded hypothetical
process-level failures and forcibly stopping stuck extraction from the
first-version requirements. Anton accepted this revised decision on 2026-10-11.
It changes the earlier isolation decision rather than merely reorganizing it.
Together with accepted
[ADR-0008](0008-separate-diagnostics-from-cli-rendering.md) and
[ADR-0009](0009-use-tracing-for-runtime-observability.md), it supersedes
ADR-0004, whose original reasoning remains preserved.

## Context

The Rust extraction backends currently run as library calls inside the
application, with a guard that converts unwinding panics into extraction errors.
The [extraction research](../research/04-pdf-extraction.md#position-data)
identifies `expect` calls in `pdf-extract` and possible panics on malformed PDFs.
It does not document process-aborting crashes or establish that these libraries
are more likely to terminate the process than other dependencies.

Selecting an extraction backend does not guarantee that every failure will be
returned as `Err`. If the backend aborts or segfaults inside the application
process, the caller cannot handle the failure through `Result`. Running
extraction in a separate worker process allows the parent to detect the worker's
termination and report the failed job.

That capability alone does not justify implementing it now. The first version
does not require surviving hypothetical process-level failures or forcibly
stopping a stuck extraction while keeping the application running. During this
review, no observed delay that significantly disrupts normal program flow was
established as a reason to introduce a worker architecture.

## Decision

Defer extraction process isolation. Keep the current in-process Rust library
integration and ordinary error handling; do not introduce a supervised worker
protocol, enforced process timeouts, forced cancellation, or a worker retry
system solely to protect against hypothetical failures.

This does not remove the obligation to report errors the application receives.
It also does not prohibit existing external tools such as `mutool`, or defer
the extraction-quality fallback selected in
[ADR-0003](0003-select-pdf-extraction-backends.md). Selecting a better extraction
result is separate from supervising a crashed or stuck worker.

If actual failures occur, investigate them on their evidence. Revisit the need
for execution isolation when document-processing delays significantly disrupt
normal program flow. First establish the cause and required user-facing
behavior, then compare remedies; a separate process is not the predetermined
answer. The corresponding first-version scope is described in
[spec §13](../superpowers/specs/2026-09-16-library-core-typst-design.md#extraction-failure-containment).

## Alternatives considered

- **Keep ADR-0004's supervised process requirement:** gives a boundary for
  termination and forced cancellation, but requires a worker protocol,
  supervision, result transfer, and lifecycle handling without an established
  first-version need. Deferred.
- **Keep in-process extraction and investigate observed problems:** avoids that
  infrastructure now and matches the agreed scope. Chosen, with the limits below.
- **Treat `catch_unwind` as equivalent isolation:** not valid. It catches
  unwinding Rust panics, not process aborts or segmentation faults, and cannot
  forcibly stop an arbitrary synchronous library call.

These alternatives reflect this review's reconsideration, not a claim that
the original ADR reached the same conclusion.

## Consequences

- There is no guarantee that the application survives an extraction library
  terminating its process. Such failures are accepted as outside the current
  containment requirements, not assumed impossible.
- A synchronous extraction can take a long time; there is no requirement to
  forcibly cancel it and continue within the same application invocation.
- ADR-0004's worker-specific obligations—supervision, validated worker-result
  publication, bounded stderr capture, worker cleanup, and bounded worker
  retries—are not first-version work under this replacement.
- Resource sandboxing, guarantees against system-wide exhaustion, and
  crash-dump infrastructure remain out of scope.
- Future work starts from observed failures or disruptive delays, not a
  preselected process architecture or a promised implementation schedule.

## References

- [ADR-0004: original decision and provenance](0004-separate-diagnostics-reporting-and-observability.md)
- [Spec §13: extraction failure behavior](../superpowers/specs/2026-09-16-library-core-typst-design.md#extraction-failure-containment)
- [ADR-0003: backend selection](0003-select-pdf-extraction-backends.md)
- [Extraction research: possible panics in font parsing](../research/04-pdf-extraction.md#position-data)
- [Current in-process extraction error and panic guard](../../crates/bib-extract/src/lib.rs)
- [Limits of catch_unwind](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html)
