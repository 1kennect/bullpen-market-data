"""Scores whether the answer avoids naming tools, source keys, or parameters."""

from __future__ import annotations

from evals.core.types import EvalCase, Score, TurnResult

#: Named in every case, on top of whatever the case itself forbids.
INTERNALS = (
    "market_quote",
    "market_search",
    "market_profile",
    "market_news",
    "market_status",
    "run_sandbox",
    "tool call",
)


def score(case: EvalCase, turns: list[TurnResult]) -> Score | None:
    """Passes when the answer names no internal or case-forbidden term."""
    forbidden = list(INTERNALS) + list(case.expect.not_mentions)
    last = turns[-1]
    text = last.text.lower()
    named = [word for word in forbidden if word.lower() in text]
    if named:
        return Score(passed=False, detail=f"answer names {named}")
    return Score(passed=True, detail="answer names nothing internal")
