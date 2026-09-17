# Hardware and local LLMs

Measured on 2026-09-14 on the development laptop.

## Specs

| Component | Value |
|---|---|
| CPU | AMD Ryzen 7 PRO 5850U, 8 cores / 16 threads |
| GPU | integrated only: AMD Radeon Vega (Cezanne) |
| VRAM | 1.0 GiB fixed (`mem_info_vram_total`) + up to 15.1 GiB dynamically from RAM (`mem_info_gtt_total`) |
| RAM | 30 GiB, 24 GiB in use at measurement time, 8 GiB swap |
| Driver | Vulkan ICDs present (`radeon_icd.json` = RADV) → llama.cpp with Vulkan backend possible |
| ROCm | not installed; Cezanne not officially supported |
| LLM runtimes | none installed (neither Ollama nor llama.cpp) |
| Kernel | Linux 6.17 |

## What's realistic locally (estimate, not measured)

Bottleneck: DDR4 memory bandwidth for generation; CPU/iGPU for ingesting long inputs.

| Task | Local? | Assessment |
|---|---|---|
| Embeddings, search index, reranking | yes | small models; index for a few hundred papers within minutes |
| Quote verification pre-stage (locate the passage, rough classification) | yes | SemanticCite shows that small specialized models suffice |
| 3–4B LLM | yes, interactively | roughly 10–20 tokens/s |
| 7–8B LLM or MoE with ~3B active parameters | background only | long inputs take a while; MoE needs ~17 GB RAM, tight given current usage |
| Extraction matrix across many papers, long summaries | cloud | cloud or a local overnight run |
| olmOCR locally | no | needs NVIDIA ≥ 12 GB VRAM |

## Resulting pattern

- Everything interactive (hover, search, suggestions) without a large LLM, just index and embeddings.
- LLM tasks in a queue, processed locally in the background.
- Cloud deliberately selectable per task (user preference: "c), but the more that runs locally, the better").
- Measure real throughput later with a short benchmark.
