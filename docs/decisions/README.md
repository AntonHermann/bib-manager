# Architecture decision register

Architecture Decision Records (ADRs) preserve why consequential choices were
made: the problem, alternatives, chosen approach, and consequences. Use them for
architectural choices, significant tradeoffs, or decisions that would be costly
to reverse—not for every implementation detail.

## Index

| ADR | Status | Decision date |
|---|---|---|
| [0001: Keep Zotero as the source of truth](0001-keep-zotero-as-source-of-truth.md) | Accepted | Unknown; confirmed during 2026-09-14–2026-09-16 |
| [0002: Use central SQLite storage](0002-use-central-sqlite-storage.md) | Accepted | Unknown; confirmed during 2026-09-14–2026-09-16 |
| [0003: Select PDF extraction backends](0003-select-pdf-extraction-backends.md) | Accepted | 2026-09-17 |
| [0004: Separate diagnostics, reporting, and observability](0004-separate-diagnostics-reporting-and-observability.md) | Accepted | 2026-10-08 |

These first three records are retrospective summaries of existing decisions,
not new approvals of the whole design spec. The source log for ADRs 0001 and
0002 identifies only the brainstorming period, not individual approval dates.
Each record links its sources.

## Responsibilities

- **ADRs:** why consequential choices were made and their status.
- **Specs:** intended system behavior and interfaces.
- **Research and benchmarks:** supporting evidence and its limitations.
- **Plans:** how to implement the agreed design.
- **Code documentation:** current contracts and local implementation reasoning.

Link supporting documents rather than copying detailed specifications or
measurements into an ADR. Accepted does not mean fully implemented; specs,
plans, and user documentation track implementation scope and progress.

## Workflow and lifecycle

1. Allocate the next unused four-digit number and a descriptive filename:
   `NNNN-short-decision-title.md`. Keep numbers and filenames stable.
2. Add a **Proposed** record using the template below, in the same branch as
   the relevant spec or code change. Add it to this index.
3. Review the choice and its consequences alongside that change. Once agreed,
   mark it **Accepted** and record the decision date. Mark a proposal that is
   declined **Rejected**, preserving why it was declined.
4. To replace an accepted decision, write a new ADR explaining why. Once the
   replacement is accepted, mark the old one **Superseded by ADR-NNNN** with a
   link, link back from the new record, and update the index and affected specs.

Correct typos and clarify references in place, but do not rewrite accepted
records to make an old choice look like the new one. If implementation or new
evidence conflicts with an ADR, resolve that conflict explicitly rather than
silently treating code as a new decision.

## Template

```markdown
# ADR-NNNN: Short decision title

Status: Proposed
Decision date: Pending
Supersedes: None

## Context

What problem needs a decision? What constraints and evidence matter?

## Decision

What will we do, and what is the scope of this choice?

## Alternatives considered

What credible alternatives were considered, and why were they not chosen?

## Consequences

What benefits, costs, limitations, and follow-up obligations result?

## References

Link the relevant spec sections, research, benchmarks, or predecessor ADRs.
```

For retrospective records, also state their provenance and any uncertainty
about dates or approval. Do not invent alternatives or claim approval based
only on the existence of an implementation.
