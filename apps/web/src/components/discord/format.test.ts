import { describe, expect, it } from "vitest";
import { finishedCard, runningCard, sourceLabel, threadName, toolDone, toolStarted } from "./format";

describe("reply format", () => {
  it("writes a running tool call as the engine does, arguments sorted by key", () => {
    expect(toolStarted("market_news", { symbol: "TSLA", exchange: "US" })).toBe(
      "\u{1f527} `market_news` (exchange: US, symbol: TSLA) — reading the news",
    );
    expect(toolStarted("some_mcp_tool", {})).toBe("\u{1f527} `some_mcp_tool` — running");
  });

  it("clips a returned call to 160 characters on one line", () => {
    const line = toolDone("market_news", { symbol: "AAPL" }, `2 recent stories about AAPL: ${"x ".repeat(200)}`);
    const shown = line.split(" → ")[1];
    expect(Array.from(shown).length).toBeLessThanOrEqual(160);
    expect(shown.endsWith("…")).toBe(true);
    expect(shown.startsWith("2 recent stories about AAPL: x")).toBe(true);
  });

  it("puts the spinner header over the steps while running, and drops it when finished", () => {
    expect(runningCard(["\u{1f914} thinking"], 1)).toBe("◓ **Bullpen is working on it…**\n-# \u{1f914} thinking");
    expect(runningCard([], 0, "Draft")).toBe("◐ **Bullpen is answering…**\n\nDraft");
    expect(finishedCard(["a", "b"], "Answer.")).toBe("-# a\n-# b\n\nAnswer.");
  });

  it("names threads and buttons within Discord limits", () => {
    expect(threadName("  what   is  open ")).toBe("what is open");
    expect(threadName("")).toBe("Question");
    expect(sourceLabel("a".repeat(50))).toHaveLength(40);
  });
});
