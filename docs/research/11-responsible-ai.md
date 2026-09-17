# Responsible AI use as a design principle

Result of the brainstorming (2026-09-14), confirmed by the user as the breakdown below.
Starting point was the EHR seminar's AI rules (FU Berlin, "Guidelines on AI usage,
presentation & report"), but the principles apply generally.

## A. The tool never writes text for the user

1. **Basic rule in the data model:** LLM features only research, organize, and verify. Only wording taken from sources ever ends up in documents, never model-generated wording. Technically enforced: there is no path from LLM output into a `.typ` file.
2. **Similarity warning:** the language server compares the user's own sentences with logged LLM outputs and warns ("resembles model output from …: mark it or rephrase"). Enforces the labeling requirement directly in the editor.

## B. Traceability instead of trust

3. **Provenance on every data object:** *from the user*, *deterministically verified*, *imported, provenance unknown*, *LLM assessment* (model, version, date), *confirmed by the user*. LLM judgments are never automatically treated as "true," always show the excerpt, and wait for confirmation.
4. **No fabricated sources:** suggestions only from one's own library, always with an anchored passage.
5. **Reproducibility:** prompt, model, parameters, and a hash of the input are stored with every LLM assessment.

## C. Automating the documentation requirement

6. **Complete usage log** of every LLM call: tool, version, date, URL, prompt, result, type of use (research, verification, structuring, …).
7. **Appendix generator:** a Typst table built deterministically from the log, not phrased by an LLM → "polished" entries structurally excluded.
8. **Capture Claude Code sessions (idea):** detect which sessions under `~/.claude/projects/` touched a project, and count them.

## D. Rules per project

9. **Per-project policy:** allowed LLM features, cloud yes/no, logging requirement; enforced by the tool.
10. **Visible data flow:** for cloud calls, record which of the user's own text left the device.

## E. Don't automate reading away

11. **"Cited but never read" indicator;** skimming aids (Scim-style) as a reading aid, not a replacement.

## Scope in the first version

The first version contains no LLM. Nonetheless, from the start:

- **Provenance fields in the data model (3).**
- **Table for the usage log (6),** initially empty; every later LLM feature must use it.

Everything else stays a binding design principle in the spec, implemented alongside the LLM features.
