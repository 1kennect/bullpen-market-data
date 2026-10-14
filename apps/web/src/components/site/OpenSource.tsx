import { ArrowUpRight } from "lucide-react";
import GithubMark from "./GithubMark";
import Heading from "./Heading";
import { ARCHITECTURE, REPO } from "./content";

/** What the repository is made of. */
const STACK = [
  { label: "Engine", value: "Rust, the agent loop and its HTTP surface" },
  { label: "Bot", value: "Rust, a Discord client of the engine" },
  { label: "Market", value: "Finnhub REST: quotes, profiles, figures, news, exchange status" },
  { label: "Sandbox", value: "Isolated containers for math and files, PaddleOCR for scans" },
  { label: "Model", value: "Any OpenAI-compatible server, llama.cpp by default" },
  { label: "Traces", value: "Phoenix over OTLP, every turn as a tree" },
];

/** How to run it yourself. */
const OpenSource = () => (
  <section id="open-source" aria-label="Open source" className="relative bg-paper py-24">
    <div className="mx-auto max-w-3xl px-5 sm:px-8">
      <div className="flex justify-center">
        <img src="/brand/bullpen-logo.svg" alt="" width={64} height={64} className="h-20 w-20" />
      </div>
      <div className="mt-8">
        <Heading title="Run it in your own server." align="center" />
      </div>

      <div className="mt-10 flex flex-wrap justify-center gap-3">
        <a
          href={REPO}
          target="_blank"
          rel="noreferrer"
          className="inline-flex h-12 items-center gap-2 rounded-full bg-accent px-7 text-sm font-semibold text-paper transition-transform hover:-translate-y-0.5"
        >
          <GithubMark className="h-4 w-4" />
          View on GitHub
        </a>
        <a
          href={ARCHITECTURE}
          target="_blank"
          rel="noreferrer"
          className="inline-flex h-12 items-center gap-2 rounded-full border-2 border-ink px-7 text-sm font-semibold transition-colors hover:bg-ink hover:text-paper"
        >
          Architecture
          <ArrowUpRight className="h-4 w-4" aria-hidden />
        </a>
      </div>

      <div className="mt-14 overflow-hidden rounded-xl border border-ink/15 bg-white">
        <dl className="divide-y divide-ink/10">
          {STACK.map((row) => (
            <div key={row.label} className="flex gap-4 px-5 py-3">
              <dt className="w-20 shrink-0 font-mono text-xs font-semibold uppercase tracking-[0.1em] text-accent">
                {row.label}
              </dt>
              <dd className="text-sm text-ink-soft">{row.value}</dd>
            </div>
          ))}
        </dl>
        <div className="bg-ink px-5 py-4 font-mono text-xs text-paper">
          <span className="select-none text-gold">$ </span>
          just up
          <span className="ml-1 inline-block h-3.5 w-1.5 translate-y-0.5 bg-paper motion-safe:animate-caret" />
        </div>
      </div>
    </div>
  </section>
);

export default OpenSource;
