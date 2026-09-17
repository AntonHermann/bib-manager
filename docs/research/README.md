# Research for the Bibliography Manager

As of: 2026-09-14 to 2026-09-16. All files here are research findings from the
brainstorming phase, not a specification. The spec is written up separately.

## Contents

| File | Topic |
|---|---|
| [01-tool-landscape.md](01-tool-landscape.md) | Existing tools: citation graphs, smart citations, extraction matrices, canvas tools |
| [02-semantic-reader.md](02-semantic-reader.md) | Semantic Reader project: concepts and state of the open-source code |
| [03-quote-verification.md](03-quote-verification.md) | How common citation errors are, and what verification systems exist |
| [04-pdf-extraction.md](04-pdf-extraction.md) | **Own measurements:** five extractors compared, licenses, decision |
| [05-data-source-apis.md](05-data-source-apis.md) | OpenAlex, Semantic Scholar (with own test), S2ORC, SPECTER2, GROBID, OCR tools |
| [06-zotero.md](06-zotero.md) | Local API, Web API, citation keys, existing installation |
| [07-zed-typst.md](07-zed-typst.md) | What Zed extensions can do, behavior with multiple language servers, Typst building blocks |
| [08-rust-building-blocks.md](08-rust-building-blocks.md) | Crates with version, maintenance status and license |
| [09-hardware-local-llms.md](09-hardware-local-llms.md) | Laptop specs and what's realistic locally |
| [10-existing-workflow.md](10-existing-workflow.md) | Today's workflow in the EHR seminar as a requirements source |
| [11-responsible-ai.md](11-responsible-ai.md) | Responsible AI use as a design principle |
| [12-decisions.md](12-decisions.md) | Decision log of the brainstorming phase |
| [13-open-questions.md](13-open-questions.md) | Unchecked assumptions and open points |
| [probes/](probes/) | The test programs from the PDF measurements, runnable |

## How reliable is this?

- **Measured:** everything in `04-pdf-extraction.md`, the Semantic Scholar test in `05`, the
  repo states in `02`/`08`, the hardware in `09`, the workflow observations in `10`.
- **Read from sources:** the rest. Every statement has a link.
- **Not checked:** see `13-open-questions.md`.
