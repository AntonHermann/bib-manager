# ADR-0002: Use central SQLite storage

Status: Accepted
Decision date: Unknown (confirmed during 2026-09-14–2026-09-16)
Supersedes: None

Provenance: Retrospectively reconstructed from the user-confirmed brainstorming
log and design spec. The log covers 2026-09-14 through 2026-09-16; it does not
give an individual approval date for this choice.

## Context

Literature and excerpts need to be usable across writing projects, rather than
tied to one repository. Some working data can be regenerated from Zotero and
PDFs, but authored excerpts, corrections, and review decisions cannot.
Configuration needs a different property: it should be hand-written,
versioned, and reviewable with the project it applies to.

## Decision

Store library and working data in one central SQLite database. Keep
project configuration in repository-local `bib.toml` files.

The CLI and language server access SQLite directly in write-ahead logging
(WAL) mode; the initial version has no daemon. Provide consistent database
backups and, as specified for later implementation, exports of irreplaceable
working data with stable identifiers.

## Alternatives considered

- **Repository-local working data as the primary store:** central storage was
  chosen so excerpts can be reused across projects; exports provide a way to
  bring data into a repository when needed.
- **A database accessed through a daemon:** the spec defers a daemon until
  loaded models would justify one. Direct SQLite access suffices initially.

The source records do not document a comparison against other database engines.

## Consequences

- Projects share one library and working-data store.
- Project configuration remains diffable and versioned separately from data.
- The database needs an explicit backup and export workflow; repository
  history alone does not protect irreplaceable working data.
- Schema changes require versioned migrations and backups before migration.
- Backup and export are distinct: backup copies the whole database
  consistently, while the planned export preserves irreplaceable data in a
  schema-independent format. Acceptance of this decision does not imply the
  export/import workflow is already implemented.

## References

- [User-confirmed brainstorming decision 10](../research/12-decisions.md)
- [Design spec, §2 Decisions, §5 Data Model, and §10 Projects and Configuration](../superpowers/specs/2026-09-16-library-core-typst-design.md)
- [Steps 1–2 plan, Global Constraints and schema scope](../superpowers/plans/2026-09-17-step-1-2-data-model-zotero-sync.md)
