## What and why

<!-- One logical change. What it does, and why; the diff shows how. -->

## Checklist

- [ ] `just check` passes (the pre-commit hook runs the recipes for the units the change touches)
- [ ] New behaviour has a test, and the test fails without the change
- [ ] No contradiction with `docs/ARCHITECTURE.md` or `docs/ROADMAP.md`, or the doc is updated here
- [ ] A new tunable is in `bullpen.toml` at its default; a new secret is in `.env.example`
- [ ] A change to a store route changes the engine adapter in `stores/http/` and the store in `apps/store` together, with a migration for any schema change
- [ ] Comments follow the style in `AGENTS.md`: plain ASCII, what and not why
- [ ] Nothing here lets a credential reach a prompt, a log, a trace or a metric label
- [ ] Nothing here lets personal memory or the profile graph reach an answer others can read
