# Bullpen Roadmap

## 1 — Market data v0.1

- [x] Quotes, symbol search, company profiles and figures, news, and exchange status over Finnhub, behind the `MarketData` trait
- [x] Prompt, evals and guardrail wording for information, not advice
- [x] Scanned uploads read by PaddleOCR in the sandbox, tesseract as the fallback
- [ ] Promote a first eval baseline against a live engine (`just eval run`, then `just eval baseline`)

## 2 — Market data v0.2

- [ ] Price history over a range, with a chart rendered in the sandbox and attached to the reply
- [ ] Earnings dates and dividends
- [ ] A short per-symbol quote cache, so a busy channel stays under the provider rate limit
- [ ] A second `MarketData` adapter, so one provider failing does not take the tools down

## 3 — Memory v0.3

- [ ] Watchlists from the profile graph: "how are my stocks doing" quotes what the user said they hold or watch
- [ ] Admin surface: tools, instructions, limits, trace inspection, approvals, rollback

## 4 — Alerts v0.4

- [ ] Price and news alerts posted to a channel, each one created through `Policy` with a confirmation

## 5 — Public beta v0.5

- [ ] Run on existing finance QA benchmarks
- [ ] Staging + prod; canary releases; alerting; inference that scales under load

## 6 — v1.0

- [ ] Stable API, documented traits, published evals, a template for adding a market data provider

## Out of scope until stated otherwise

Placing trades or linking brokerage accounts · personalized investment advice or price predictions · browser automation of any kind · one agent per domain · RL before evals exist.
