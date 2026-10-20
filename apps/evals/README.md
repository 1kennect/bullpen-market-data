# apps/evals

Deterministic engine evals with a baseline gate.

```bash
just eval run | baseline | compare
```

| Module | Holds |
|---|---|
| `runner.py` | posts each golden case to `/chat` and reads the engine's JSONL trace for that request |
| `suites/` | deterministic scorers: tool selection and arguments, grounding, refusal, permissions, clarification, memory, latency, voice |
| `cases/` | hand-written market questions with expectations; add a line and it runs |
| `baseline.json` | accepted suite rates, written by `just eval baseline`; absent until one is promoted |
| `core/` | settings (the `[evals]` section of `bullpen.toml`) and shared types |

Reports go under `.bullpen/evals/`. Engine traces are read from `.bullpen/traces/`. Golden cases and the promoted baseline are committed here. Evals need a live engine.

## Cases

Every `*.jsonl` under `cases/` is loaded.

- `market_golden.jsonl` is the core set: grounding, refusal of advice and predictions,
  permissions, memory, latency.
- `tool_routing.jsonl` is the regression set for tool choice and answer voice: which market
  tool answers a question, when arithmetic goes to the sandbox, and that the answer never names
  a tool.

## Suites

`tool_selection` and `tool_args` read the trace: which tool ran and what it was given.
`grounding` reads the answer and its citations, matching a citation by its source key rather
than its label. `voice` fails an answer that names its own machinery, which is what a user
can neither act on nor verify. `memory`, `permissions`, `clarification`, `refusal` and
`latency` are unchanged.

Run one with `just eval run --suite voice`.
