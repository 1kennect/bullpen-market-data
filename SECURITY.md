# Security

Bullpen answers market questions from live data, calls tools on a user's behalf, and remembers the people who talk to it, including what they hold. Report anything that could expose what it remembers, put a figure it did not fetch into an answer, or make it act without permission.

## Reporting

Use GitHub private vulnerability reporting on this repository (Security tab, Report a vulnerability), not a public issue. Expect an acknowledgement within a week. Bullpen is pre-1.0; only the latest release and `main` are supported.

## In scope

- one guild or user reaching another's conversations, profile graph or remembered facts
- a token, key or secret reaching a model prompt, a log line, a trace, a metric or a Discord message
- personal memory or the profile graph reaching an answer others can read
- a write-side tool running without its confirmation, or confirmed by someone without the role
- prompt injection through a retrieved page or a tool result that leads to an action nobody asked for
- model output stored or cited as if it were market data
- the market data API key reaching a prompt, a log, a trace, or a request URL

## Model

- The tenant is the guild. One engine serves one guild. Every store route carries the tenant and the user in its path, and every query is scoped by both; conversations, memory, the profile graph and confirmations of one user are never read for another.
- The store is reachable only from the engine: compose publishes no port for it, and every route except `/health` requires `BULLPEN_STORE__TOKEN`, compared in constant time. A database error reaches the caller as a 500 without its cause.
- A direct message is `Visibility::Private`; a guild thread is `Visibility::Public`. `agent.recall_in_public` keeps personal memory and the profile graph out of a public answer.
- Secrets are `SecretString`, come from the environment only, and are never logged.
- Write-side tools go through `Policy`. `policy.confirm_from` decides what needs a confirmation, which expires; the model never decides whether one is required.
- Market data is read per call and never stored by the engine. The Finnhub key travels in a request header, never in a URL. Model output is never stored as a source.
- `run_sandbox` runs without network, or with `sandbox.egress` through a proxy that refuses private, loopback, link-local and reserved addresses.
- `apps/engine`, `apps/discord` and `apps/cli` never depend on each other (`scripts/check-deps.sh`).
- Each deployment is self-hosted; no telemetry is sent to the project.

## Dependencies

cargo-deny and pip-audit on dependency changes, dependency review on pull requests, secret scanning on every push, Dependabot weekly.
