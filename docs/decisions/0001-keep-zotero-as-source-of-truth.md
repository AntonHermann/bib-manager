# ADR-0001: Keep Zotero as the source of truth

Status: Accepted
Decision date: Unknown (confirmed during 2026-09-14–2026-09-16)
Supersedes: None

Provenance: Retrospectively reconstructed from the user-confirmed brainstorming
log and design spec. The log covers 2026-09-14 through 2026-09-16; it does not
give an individual approval date for this choice.

## Context

Zotero already supplies the browser connector and tablet sync. The project
needs a richer workspace for using literature, but replacing these capture and
mobile capabilities would expand the initial scope substantially.

## Decision

Keep Zotero as the source of truth for bibliographic sources and attachments.
`bib` reads from Zotero and stores its own working data locally. Every source
originates in Zotero; a source found only in a `.bib` file is not created in the
database. Writing back to Zotero is outside the initial version.

This does not make Zotero the authority for all `bib` data: excerpts,
corrections, review decisions, and later notes belong to the local workspace.

## Alternatives considered

- **Replace Zotero entirely:** replacing the connector and tablet workflow is
  too costly for the first version.
- **Write back to Zotero in the initial version:** deferred; the brainstorming
  log records that typed notes do not need to reach the tablet. Zotero
  identifiers are retained so writing back remains possible later.

## Consequences

- The existing capture and tablet workflows remain available.
- Sync preserves Zotero identity as library plus item key, not item key alone.
- Changes to local working data do not update Zotero.
- With the local API unavailable, `bib` uses the last synced state. The planned
  read-only `zotero.sqlite` fallback is deferred.
- Sources present only in `.bib` files must be added to Zotero, rather than
  becoming a second source authority.

## References

- [User-confirmed brainstorming decisions 2–3](../research/12-decisions.md)
- [Design spec, §2 Decisions and §6 Zotero Sync](../superpowers/specs/2026-09-16-library-core-typst-design.md)
- [Steps 1–2 plan, Decisions made while writing this plan](../superpowers/plans/2026-09-17-step-1-2-data-model-zotero-sync.md)
