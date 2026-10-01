import { thoughtStep, toolDone, type Args } from "./format";
import { PEOPLE, type Author, type LinkButton } from "./people";

/** A page a reply cites, shown as a link button under the answer. */
type Cited = { label: string; url: string };

/** One tool call of an example: the tool, its arguments, what it returned, and what it cites. */
export type Call = { tool: string; args: Args; text: string; cites?: Cited };

/** The link button a cited page puts under the answer. */
export const button = (cited: Cited): LinkButton => ({ label: cited.label, href: cited.url });

/** One question and the finished reply, as a user would see it in a thread. */
export type Example = {
  title: string;
  asker: Author;
  question: string;
  thought: string;
  calls: Call[];
  answer: string;
};

/** The examples the page shows, one per kind of question. Figures are illustrative. */
export const EXAMPLES: Example[] = [
  {
    title: "Quotes",
    asker: PEOPLE.devon,
    question: "how's NVDA doing today?",
    thought: "Prices move by the second, so pull the latest quote and today's headlines for NVDA together.",
    calls: [
      { tool: "market_quote", args: { symbol: "NVDA" }, text: "NVDA last 182.41, +3.12 (+1.74%) on the day, as of 2026-10-21 19:58 UTC." },
      {
        tool: "market_news",
        args: { symbol: "NVDA" },
        text: "2 recent stories about NVDA, newest first: Reuters: Nvidia extends rally on data center orders",
        cites: { label: "Nvidia extends rally on data center orders", url: "https://www.reuters.com/" },
      },
    ],
    answer: "NVDA is at **$182.41**, up **1.74%** today as of 3:58 PM ET. The move follows reports of new data center orders.",
  },
  {
    title: "Company lookup",
    asker: PEOPLE.ren,
    question: "what's the ticker for palantir and what's its market cap?",
    thought: "The user named a company, not a ticker. Find the symbol first, then read its profile.",
    calls: [
      { tool: "market_search", args: { query: "palantir" }, text: "3 symbols match \"palantir\": - PLTR: PALANTIR TECHNOLOGIES INC-A (Common Stock)" },
      {
        tool: "market_profile",
        args: { symbol: "PLTR" },
        text: "Palantir Technologies Inc (PLTR) exchange: NASDAQ industry: Technology market cap: 412.80B USD",
        cites: { label: "Palantir Technologies Inc", url: "https://www.palantir.com/" },
      },
    ],
    answer: "Palantir trades as **PLTR** on the Nasdaq. Its market cap is about **$412.8 billion**.",
  },
  {
    title: "Market hours",
    asker: PEOPLE.maya,
    question: "is the market open right now?",
    thought: "Check the US exchange status rather than guessing from the clock; holidays close it too.",
    calls: [{ tool: "market_status", args: {}, text: "The US market is closed, session: post-market (exchange time zone America/New_York)." }],
    answer: "The regular session is **closed**. US stocks are in **after-hours** trading until 8 PM ET.",
  },
  {
    title: "Position math",
    asker: PEOPLE.sam,
    question: "I bought 40 AAPL at 112.50, what am I up?",
    thought: "Quote AAPL, then compute the gain in the sandbox rather than in my head.",
    calls: [
      { tool: "market_quote", args: { symbol: "AAPL" }, text: "AAPL last 227.52, -1.23 (-0.54%) on the day, as of 2026-10-21 19:59 UTC." },
      { tool: "run_sandbox", args: { command: "python3 -c \"print(round((227.52-112.50)*40,2), round((227.52/112.50-1)*100,2))\"" }, text: "4600.8 102.24" },
    ],
    answer: "You're up **$4,600.80**, a **102.24%** gain, with AAPL at $227.52. That's information, not advice on what to do next.",
  },
];

/** The steps a finished example shows: the thought, then each call that returned. */
export const stepsOf = (example: Example) => [
  thoughtStep(example.thought),
  ...example.calls.map((c) => toolDone(c.tool, c.args, c.text)),
];
