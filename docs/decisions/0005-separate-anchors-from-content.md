# ADR-0005: Separate anchors from content

Status: Accepted
Decision date: 2026-10-09
Supersedes: None

Provenance: Retrospectively reconstructed from the September 2026 design spec
(§5 and §16) and extraction research on robust anchors. Those sources describe
the intended model, but do not establish an individual approval date for this
combined decision. Anton accepted this record during the documentation review
on 2026-10-09, after clarifying bibliographic sources and the evidence for
text-offset fragility. The date records that approval, not a reconstructed
September decision or implementation milestone.

## Context

Excerpts, text corrections, and later notes need to refer to passages or regions
in the papers and other literature being cited, without each inventing a different
location model. Here, "source" means a bibliographic work, not program source code.
The excerpts, corrections, and notes are authored, irreplaceable data, unlike an
extracted text layer that can be regenerated.

An extraction update can change text order, normalization, or the serialization
of a table, invalidating character offsets in the extracted text. A rectangle
can still identify a region of an unchanged PDF, but does not by itself identify
the corresponding passage in a newly extracted text layer. The
[extraction research](../research/04-pdf-extraction.md#robust-anchors-w3c-web-annotation)
points to W3C quote and position selectors and Hypothesis-style re-anchoring as
the basis for resilient text references.

The model also needs to represent a source without extracted text and a passage
with several possible matches. Neither case justifies discarding the authored
content or silently claiming that its location is known.

## Decision

Keep anchors as independent records, separate from the content that refers to
them. Excerpts, corrections, and later notes or figures use the same anchor
model. Keep an excerpt's source content separate from its project-specific usage.

For text-range anchors, retain wording, surrounding context, and position.
Resolve them against the canonical text layer, using wording and context to
recover from position changes. Version text layers and re-resolve their anchors
when the text changes; unresolved or conflicting results require explicit state
and review, not silent reassignment. Retain the spec's page-range anchor kind
for page/rectangle targets; text recovery does not give those targets the same
re-anchoring guarantees.

Use the shared `node_type` plus `node_id` convention for references to different
kinds of nodes, rather than defining a separate reference vocabulary for every
feature. This is the logical reference convention, not a choice here of ID
representation, SQL constraints, or a generic graph-storage implementation.

The spec remains authoritative for behavioral details: exact → normalized →
fuzzy resolution, thresholds, context and page hints, ambiguity handling, and
the distinction between verifying wording and creating an anchored excerpt.

## Alternatives considered

The [W3C Text Position Selector specification](https://www.w3.org/TR/annotation-model/#text-position-selector)
explicitly describes character-position selection as "very brittle with regards
to changes to the resource." This supports the concern about text offsets, not a
claim that re-extraction necessarily invalidates PDF rectangles. Our
[research notes](../research/04-pdf-extraction.md#robust-anchors-w3c-web-annotation)
make that broader coordinate-only claim, but do not document a PDF experiment
establishing it.

[Spec §16](../superpowers/specs/2026-09-16-library-core-typst-design.md#16-future-work)
explains the local design concern: changed table serialization shifts text
offsets. The proposed model combines text evidence with position so that a
text-range anchor is not dependent on offsets or geometric selection alone.

The sources do not record a systematic comparison of separate anchor records
against feature-specific embedded locations, or of the shared node-reference
convention against other identifier schemes. No such historical evaluation is
claimed here.

## Consequences

- Features share location semantics and can evolve without duplicating anchor
  resolution rules in excerpts, corrections, and notes.
- Regenerating a text layer entails re-resolution and potentially human review;
  preserving authored content does not guarantee automatic recovery of a location.
- Missing text and ambiguous matches are valid states. They must remain visible
  rather than being confused with verified locations.
- Polymorphic node references need target-type and target-existence validation;
  the convention alone does not provide referential integrity.
- Anchor records are part of the irreplaceable data covered by the export
  contract in [spec §5](../superpowers/specs/2026-09-16-library-core-typst-design.md#5-data-model)
  and the storage decision in [ADR-0002](0002-use-central-sqlite-storage.md).
- This records intended architecture, not completed functionality. The current
  steps 1–2 schema does not yet implement the anchor model; later steps introduce
  the tables and resolution behavior.

## References

- [Spec §5: data model and anchor behavior](../superpowers/specs/2026-09-16-library-core-typst-design.md#5-data-model)
- [Spec §16: changing table serialization and future targets](../superpowers/specs/2026-09-16-library-core-typst-design.md#16-future-work)
- [Extraction research: robust anchors](../research/04-pdf-extraction.md#robust-anchors-w3c-web-annotation)
- [ADR-0002: central storage, backups, and export](0002-use-central-sqlite-storage.md)
- [W3C Web Annotation: Text Quote Selector](https://www.w3.org/TR/annotation-model/#text-quote-selector)
- [W3C Web Annotation: Text Position Selector](https://www.w3.org/TR/annotation-model/#text-position-selector)
- [Hypothesis: Fuzzy Anchoring](https://web.hypothes.is/blog/fuzzy-anchoring/)
