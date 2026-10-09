# ADR-0006: Enforce responsible-AI invariants

Status: Accepted
Decision date: 2026-10-09
Supersedes: None

Provenance: Retrospectively reconstructed from the user-confirmed responsible-AI
research of 2026-09-14, brainstorming decision 12, and spec §12. Those sources
establish the earlier principles. Anton accepted this record on 2026-10-09,
including the clarification that cloud-call disclosure covers all transmitted
data regardless of origin. The date records this ADR's approval, not a
reconstructed September decision or implementation milestone.

## Context

The tool supports academic work: researching literature, organizing evidence,
and verifying quotations. Future model-assisted features must not turn it into
a ghostwriter or silently promote an uncertain model assessment into a verified
fact.

The [responsible-AI research](../research/11-responsible-ai.md) derives these
constraints from the author's academic workflow and its disclosure requirements.
They are intended to apply across subprojects. Leaving enforcement to each
feature's interface or to user discipline would not provide the technical
guarantees required by [spec §12](../superpowers/specs/2026-09-16-library-core-typst-design.md#12-responsible-ai-use).

## Decision

Treat the following as architectural invariants for future model-assisted
features, not optional interface guidance:

1. **Keep model-generated wording out of paper content.** There must be no code
   path from LLM-generated wording into a user's `.typ` document. The tool may
   insert wording taken from cited literature; model-assisted research and
   verification must not become a document-writing path.
2. **Preserve provenance.** Entries distinguish user-authored content, imported
   content, deterministic verification, model assessments, and user confirmation.
   A model assessment references its `llm_call` record and is never automatically
   treated as verified truth. Confirmation must not disguise its model origin.
3. **Log every LLM call.** All later LLM features must use `llm_call`, recording
   the model/version, parameters, prompt, input checksum, result, and the policy
   in effect. The log supports traceability and documentation; it does not promise
   bit-for-bit reproducibility of model output. Detailed disclosure fields and
   deterministic appendix generation remain in
   [spec §12](../superpowers/specs/2026-09-16-library-core-typst-design.md#12-responsible-ai-use).
4. **Enforce the project's policy.** Allowed uses and cloud permissions come from
   the project's `[ai]` policy, not merely interface warnings. Features must not
   bypass those restrictions or the mandatory logging requirement. Record the
   effective policy checksum and, for cloud calls, what data left the device,
   regardless of authorship or origin. This includes prompts, file contents,
   extracted passages, metadata, and any other transmitted context—not just
   text written by the user.

This records constraints, not a new LLM feature or a choice of model/provider.
The first version remains deterministic and includes no LLM. The remaining
principles in [spec §12](../superpowers/specs/2026-09-16-library-core-typst-design.md#12-responsible-ai-use),
including library-grounded suggestions and reading aids rather than replacements
for reading, continue to apply; omission from this ADR does not relax them.

## Alternatives considered

The [responsible-AI brainstorming notes from 2026-09-14](../research/11-responsible-ai.md)
and [spec §12](../superpowers/specs/2026-09-16-library-core-typst-design.md#12-responsible-ai-use)
call for technical enforcement and traceability. Neither records a systematic
comparison with optional logging, warning-only policy checks, or a writing
assistant. These are not presented as previously evaluated alternatives.

For this documentation review, the choices were to keep all principles only in
the spec, or also record the rules that every LLM feature must obey in an ADR.
This decision uses the latter: the ADR makes those shared requirements easy to
find, while the spec retains behavioral details rather than duplicating them here.

## Consequences

- Each future LLM integration needs policy enforcement, provenance, and call
  logging as part of the feature, not as a later reporting enhancement.
- A policy allowing model use does not override the prohibition on inserting
  generated wording or make logging optional.
- Logs contain prompts, outputs, and potentially private research or writing.
  Their handling must account for that sensitivity; this ADR does not select
  retention, redaction, or publication behavior.
- A deterministic appendix documents usage; it is not permission to insert
  model-generated prose into a paper.
- The current schema contains an empty `llm_call` table and configuration
  parsing supports `[ai]`. These are foundations, not evidence of working
  runtime enforcement or a complete future provenance workflow.

## References

- [Spec §12: responsible AI use](../superpowers/specs/2026-09-16-library-core-typst-design.md#12-responsible-ai-use)
- [Spec §5: provenance in the data model](../superpowers/specs/2026-09-16-library-core-typst-design.md#5-data-model)
- [User-confirmed responsible-AI research](../research/11-responsible-ai.md)
- [Historical brainstorming decision 12](../research/12-decisions.md)
- [ADR-0002: repository-local project configuration](0002-use-central-sqlite-storage.md)
