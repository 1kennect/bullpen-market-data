# Contributing

[AGENTS.md](AGENTS.md) has the commands, layering rules, and conventions every change follows. Read [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) before changing a boundary. Both apply to people and coding agents.

## Setup

Needs [just](https://just.systems), a Rust toolchain, [uv](https://docs.astral.sh/uv), Node, and Docker. `just doctor` names anything missing and how to install it.

```
just doctor        # what is missing
just bootstrap     # .env, hooks, dependencies
just check         # the gate: every unit
```

`bootstrap` copies `.env` from `.env.example`, points `core.hooksPath` at `.githooks`, and installs each unit's dependencies. The engine needs the store running (`just postgres`, then `just store`) and `BULLPEN_STORE__URL` and `BULLPEN_STORE__TOKEN` in `.env`; the store needs the same token and `BULLPEN_STORE__DATABASE_URL`. `just check` runs the store's database tests only when `BULLPEN_STORE_TEST_DATABASE_URL` names a Postgres to use. The pre-commit hook and CI run the gate only for the units a change touches.

### A model

`bootstrap` does not start one, and the engine answers nothing without it.

| | |
|---|---|
| `just model` | `llama-server` on CUDA. Needs an NVIDIA GPU and the container toolkit. |
| `just model-cpu` | the same GGUFs on the processor. Slow; fit for a smoke test. |
| hosted | set `BULLPEN_MODEL__BASE_URL` and `BULLPEN_MODEL__API_KEY` in `.env` to any OpenAI-compatible endpoint. |

Without one the engine still starts: `/health/live` answers, `/health/ready` reports `{"store":true,"model":false}`, and `/chat` returns 502 `the model is unavailable`.

### The sandbox

`run_sandbox` needs a container runtime. With Docker running it works; without one the engine logs a warning at boot and does not offer the tool. `BULLPEN_SANDBOX__REQUIRED=true` makes a missing runtime a boot failure, as in production.

## Branches and pull requests

Branch from `main` and name the branch for the change (`reply-threads`, `fix-login-link`). One logical change per pull request, with a test for new behaviour that fails without the change.

Commit subjects are imperative, at most 72 characters, with no trailing period. The body explains why.

A change is not done until `just check` passes: `cargo fmt --check`, `cargo clippy --workspace --all-targets -D warnings`, `cargo test --workspace` and `scripts/check-deps.sh` for Rust, ruff and pytest for `apps/evals`, and eslint, tsc, vitest and a build for the website.

CI clippy runs on current stable. If a lint fails in CI but not locally, run `rustup update stable`.

### A stacked branch

CI runs on pull requests into `main` only. For a pull request into another branch, run **CI** by hand from the Actions tab (`workflow_dispatch`); a manual run gates every unit.

## Rules that are not style

- `core` imports nothing else in its crate; `engine`, `discord` and `cli` never depend on each other (`scripts/check-deps.sh`).
- No `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `dbg!` or `println!`. The workspace lints deny them, in tests too.
- Every tunable value goes in `bullpen.toml` at its default, in the same change. `.env` holds only secrets, per-machine URLs, and what compose and the justfile read.
- Market figures come from a tool result in the same turn. Model output is never stored as if it were market data.
- Write-side tools go through `Policy`; consequential actions need a confirmation.
- Comments are plain ASCII and say what the code does. The why belongs in the commit message. See Conventions in AGENTS.md.

## CI and CD

| Workflow | Runs on | Does |
|---|---|---|
| CI | pull requests into main, main, manual | the gate for each unit that changed, or every unit on a manual run |
| CD | main, tags | builds and publishes the images whose inputs changed |

## Reporting

Bugs, wrong answers and feature requests: the issue templates. Vulnerabilities: privately, see [SECURITY.md](SECURITY.md).
