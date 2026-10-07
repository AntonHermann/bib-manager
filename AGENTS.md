# Project instructions

Superpowers process skills are active in this project. Use the personal
installation; do not vendor its skills or bootstrap into this repository.

## Documentation

Write primarily for maintainers: explain responsibilities, contracts, and
reasoning without making readers reconstruct them from call sites and tests.
Keep public API docs useful; exhaustive generated reference coverage is not
the goal.

- Use `//!` for crate and substantial-module responsibilities, key concepts,
  and relationships.
- Use `///` for public types, traits, and functions: purpose, constraints,
  side effects, errors, and surprising behavior. Document non-obvious private
  contracts too; trivial helpers need no ceremonial comments.
- Explain field/variant meanings, units, conventions, and special values where
  unclear. Put shared conventions on the containing type.
- Implementation comments explain why: invariants, ordering, algorithm choices,
  and workarounds—not what each statement does.
- Add examples and `# Errors`, `# Panics`, or `# Safety` sections where useful
  or required. Avoid restating names/signatures or adding empty boilerplate.
- Explain unfamiliar terms and abbreviations (e.g. NFKC), their local relevance,
  and link authoritative references. Preserve external documentation supporting
  implementation decisions near the relevant code; prefer specific API sections,
  specifications, and upstream issues. Distinguish guarantees from observations;
  record versions for version-specific workarounds. Share links at module/type
  level rather than repeating them.
- Keep docs accurate and close to the code. Source docs explain current behavior;
  design docs provide background; user docs cover setup, workflows, and limitations.

Apply to new items and relevant changes; unrelated backfills are separate work.
Review usefulness and accuracy, not comment counts. No blanket `missing_docs` lint.

### Architecture decisions

Record consequential architectural choices, significant tradeoffs, and
costly-to-reverse decisions in `docs/decisions/`, following its README template
and lifecycle. Add the ADR in the same branch as the relevant spec or code
change, and update the index. Acceptance requires agreement on the decision,
not merely implementation. Supersede accepted decisions with linked new ADRs
rather than erasing their reasoning.

ADRs explain why; specs describe intended behavior, plans describe implementation,
and research or benchmarks provide evidence. Link those documents instead of
duplicating them. Routine implementation details do not need ADRs.

## Project commands

Run commands from the repository root with Rust 1.95 or newer.

- Install dependencies: `./.agents/prepare`
- Unit tests (no external PDF fixtures): `cargo test --workspace --lib --bins --locked`
- Lint/typecheck: `cargo clippy --workspace --all-targets --locked -- -D warnings`

Extraction integration tests additionally require the ignored PDF fixtures
listed in `.agents/linked`. Delta links them from the primary checkout on
this machine when creating new checkouts; existing checkouts are not updated.
The coordinate comparison also needs `mutool` to exercise both backends.
See `bench/README.md` for corpus setup and optional PDFium requirements.
