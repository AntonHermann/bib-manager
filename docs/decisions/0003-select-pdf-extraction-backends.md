# ADR-0003: Select PDF extraction backends

Status: Accepted
Decision date: 2026-09-17
Supersedes: None

Provenance: Retrospectively reconstructed from the step 0a benchmark decision
and the updated design spec. This records the measured selection and planned
cascade, not a claim that the cascade is already implemented.

## Context

PDF extraction must preserve readable text, including mathematical characters,
and provide geometry where available. Measurements found missing ε characters
in some `pdf_oxide` output, so one backend cannot be assumed reliable for all
pages. The project uses MIT OR Apache-2.0 licensing and avoids AGPL
dependencies.

Step 0a compared extractors on a curated corpus against an explicit decision
rule. This is evidence for that corpus, not a universal accuracy guarantee.

## Decision

Keep extraction behind a backend interface. Use **`pdf_oxide` as the default**,
with **`pdf-extract` as the fallback on suspicious pages**. The spec includes
optional **`mutool` as a third stage**; it also serves as a benchmark yardstick.
Record which backend produced each page's selected extraction.

Geometry is optional: a text-only result is usable, but must be marked as
having no geometry. A custom repair step for missing Unicode mappings is
desirable, not a requirement of the initial version.

## Alternatives considered

- **`pdf-extract` as default:** the benchmark's first rule selected
  `pdf_oxide`, whose sentence-match and reading-order scores were within the
  required margin of competing backends. `pdf-extract` recovered the missing
  characters in the affected documents and therefore serves as fallback.
- **PDFium as default:** rule 2 was not evaluated because rule 1 already
  selected `pdf_oxide`. The report notes that PDFium would have failed rule 2
  on sentence matching (18/21 < 0.95), despite passing on reading order (7/7).
- **`mutool` as default:** excluded by the benchmark rule because of its AGPL
  license. It is a reference backend and optional external process, not a
  required linked dependency.

## Consequences

- Backend selection remains revisable without changing the core data model.
- Suspicion detection must actually flag silent character loss; the benchmark
  proves fallback recovery for the affected documents, not detector coverage.
- Text-only fallback limits location display to page-level navigation and
  requires text-based anchor resolution.
- Per-page backend provenance enables selective re-extraction later.
- Corpus limitations and detector validation remain explicit follow-up work;
  they must not be hidden behind the accepted backend choice.

## References

- [Benchmark decision rule](../../bench/README.md)
- [2026-09-17 benchmark results and decision](../../bench/results/2026-09-17.md)
- [Initial extraction measurements and license constraints](../research/04-pdf-extraction.md)
- [Design spec, §7 Extraction and Normalization and §18 Open Points](../superpowers/specs/2026-09-16-library-core-typst-design.md)
- [Steps 1–2 plan, Follow-up plans](../superpowers/plans/2026-09-17-step-1-2-data-model-zotero-sync.md)
