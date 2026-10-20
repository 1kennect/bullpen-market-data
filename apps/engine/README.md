# apps/engine

The agent and the HTTP surface in front of it. One loop: assemble a prompt, call the model, run the tools it asks for, repeat until it answers or hits a limit.

```bash
just engine                         # needs .env; see CONTRIBUTING.md for the first run
cargo test -p engine                # unit tests
cargo test -p engine -- --ignored   # tests that need a container runtime
```

## Routes

| Route | Does |
|---|---|
| `POST /chat` | one turn, answered whole |
| `POST /chat/stream` | the same turn as server-sent events: progress, then the answer |
| `POST /confirm` | approves or refuses an action the policy held, by token |
| `POST /conversation/reset` | ends the caller's conversations in a channel |
| `POST /profile/list`, `/profile/forget` | what the profile graph holds about the caller |
| `GET /sandbox`, `DELETE /sandbox/{name}`, `POST /sandbox/enabled` | sandbox containers and the `run_sandbox` switch |
| `GET /health/live` | the process is up |
| `GET /health/ready` | `{"store":bool,"model":bool}`, 503 until both are true |

Every route except health requires the bearer `BULLPEN_ENGINE__SERVICE_TOKEN`. An unreachable model is not a crash: `/health/ready` reports `model:false` and `/chat` answers 502 `the model is unavailable`.

## Layout

```
src/core/       config, telemetry, types, traits, tests. Imports nothing else in the crate.
src/runtime/    harness (loop, prompt assembly, memory, safety, compaction, tracing),
                model (Rig client, slot limit), tools (market, wiki, files, mcp, sandbox)
src/stores/     http/: conversations, memory, the profile graph, and confirmations over the
                store HTTP API (apps/store)
src/routes/     axum handlers
src/wiring/     builds every adapter from config and composes them; the boot checks live here
```

`runtime::harness`, `runtime::model`, `runtime::tools` and `stores` each import only `core`.

## The loop

`Agent::run` in `runtime/harness/agent/`. History and memory are loaded once before the first step; nothing is retrieved until the model calls a tool. A step calls the model, passes the response through the guardrail, authorizes the requested tool calls against `Policy`, runs the allowed ones, and continues. It stops on an answer, a confirmation, `agent.max_steps`, the request deadline, or cancellation.

Tools are `market_quote`, `market_search`, `market_profile`, `market_news`, `market_status`, `run_sandbox`, `wikipedia_lookup`, the tools of the hosted MCP server when `mcp.hosted.url` is set, and any configured MCP servers. The market tools read Finnhub through the `MarketData` trait (`runtime/tools/market/`). Each declares a `RiskClass`; anything at `policy.confirm_from` or above waits for the caller to approve.

Details: Request lifecycle, Agent loop, Capabilities, and Market data in [docs/ARCHITECTURE.md](../../docs/ARCHITECTURE.md).

## Configuration

`bullpen.toml` sections `agent`, `prompt`, `model`, `tools`, `policy`, `sandbox`, `guardrail`, `compaction`, `profile`, `mcp`, `market`, `wikipedia`, `trace`, `http`, and `store`. `.env` holds only secrets and per-machine URLs, including the required `BULLPEN_STORE__URL`, `BULLPEN_STORE__TOKEN`, and `BULLPEN_MARKET__API_KEY`. `Config::validate` rejects a bad combination at boot. `wiring/` adds the checks that need a live dependency: the prompt against one model slot, the capabilities against their budget, and the container runtime behind `run_sandbox`.
