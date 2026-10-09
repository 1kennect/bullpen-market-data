import { ArrowRight } from "lucide-react";
import Thread from "@/components/discord/Thread";
import { EXAMPLES } from "@/components/discord/examples";
import GithubMark from "./GithubMark";
import { REPO, RUN } from "./content";

/** Opening screen: the promise, the two ways in, and a real-looking thread beside them. */
const Hero = () => (
  <section aria-label="Introduction" className="chart-grid relative isolate overflow-hidden">
    <div className="mx-auto grid max-w-6xl items-center gap-12 px-5 pb-20 pt-32 sm:px-8 lg:grid-cols-[1fr_1fr] lg:pt-36">
      <div>
        <p className="font-mono text-xs font-semibold uppercase tracking-[0.18em] text-accent motion-safe:animate-rise">
          Open-source market assistant for Discord
        </p>
        <h1 className="mt-5 text-balance text-[clamp(2.8rem,6.5vw,5.2rem)] font-black leading-[0.98] tracking-[-0.035em] motion-safe:animate-rise">
          Market answers, <span className="text-gain">with receipts.</span>
        </h1>
        <p
          className="mt-7 max-w-lg text-pretty text-lg leading-8 text-ink-soft motion-safe:animate-rise"
          style={{ animationDelay: "0.15s" }}
        >
          Ask about a price, a company, today&rsquo;s headlines, or what your position is worth.
          Bullpen pulls live market data, does the math in a sandbox, and answers with the time and
          the source.
        </p>

        <div
          className="mt-10 flex flex-wrap items-center gap-3 motion-safe:animate-rise"
          style={{ animationDelay: "0.25s" }}
        >
          <a
            href={RUN}
            target="_blank"
            rel="noreferrer"
            className="group inline-flex h-12 items-center gap-2 rounded-full bg-accent px-7 text-sm font-semibold text-paper shadow-[0_14px_30px_-12px_rgba(11,61,46,0.7)] transition-transform hover:-translate-y-0.5"
          >
            Run Bullpen
            <ArrowRight className="h-4 w-4 transition-transform group-hover:translate-x-0.5" aria-hidden />
          </a>
          <a
            href={REPO}
            target="_blank"
            rel="noreferrer"
            className="inline-flex h-12 items-center gap-2 rounded-full border-2 border-ink px-6 text-sm font-semibold transition-colors hover:bg-ink hover:text-paper"
          >
            <GithubMark className="h-4 w-4" />
            GitHub
          </a>
        </div>
        <p className="mt-6 text-xs text-ink-soft">Information, not investment advice.</p>
      </div>

      <div className="motion-safe:animate-rise" style={{ animationDelay: "0.3s" }}>
        <Thread example={EXAMPLES[0]} />
      </div>
    </div>
  </section>
);

export default Hero;
