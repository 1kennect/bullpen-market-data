import { motion } from "motion/react";
import { ArrowUpRight, Calculator, FileCheck2, LineChart, Search } from "lucide-react";
import Heading from "./Heading";
import { ARCHITECTURE } from "./content";

/** The four things a turn does, in order. */
const STAGES = [
  { icon: Search, title: "Find the symbol", body: "A company name becomes a ticker before anything is quoted." },
  { icon: LineChart, title: "Pull live data", body: "Quotes, profiles, news and exchange hours come from the data provider this turn." },
  { icon: Calculator, title: "Do the math", body: "Gains, returns and comparisons run in an isolated sandbox, not in the model's head." },
  { icon: FileCheck2, title: "Answer with the source", body: "Every figure carries its time, and every story its link." },
];

/** The turn, stage by stage. */
const HowItWorks = () => (
  <section id="how-it-works" aria-label="How it works" className="chart-grid relative py-24">
    <div className="mx-auto max-w-6xl px-5 sm:px-8">
      <Heading title="Grounded by construction, not by asking nicely." align="center" />

      <ol className="mt-16 grid gap-6 sm:grid-cols-2 lg:grid-cols-4">
        {STAGES.map((stage, i) => (
          <motion.li
            key={stage.title}
            className="rounded-xl border border-ink/10 bg-white p-6 shadow-sm"
            initial={{ opacity: 0, y: 30 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true, margin: "-15% 0px" }}
            transition={{ duration: 0.6, delay: i * 0.12, ease: [0.22, 1, 0.36, 1] }}
          >
            <div className="flex items-center gap-3">
              <span className="grid h-10 w-10 place-items-center rounded-lg bg-accent text-gold">
                <stage.icon className="h-5 w-5" aria-hidden />
              </span>
              <span className="font-mono text-xs font-semibold text-ink-soft">0{i + 1}</span>
            </div>
            <h3 className="mt-5 text-lg font-bold">{stage.title}</h3>
            <p className="mt-2 text-sm leading-6 text-ink-soft">{stage.body}</p>
          </motion.li>
        ))}
      </ol>

      <div className="mt-14 text-center">
        <a
          href={ARCHITECTURE}
          target="_blank"
          rel="noreferrer"
          className="inline-flex items-center gap-1.5 border-b-2 border-accent pb-0.5 text-sm font-semibold text-accent transition-colors hover:border-ink hover:text-ink"
        >
          Read the architecture
          <ArrowUpRight className="h-4 w-4" aria-hidden />
        </a>
      </div>
    </div>
  </section>
);

export default HowItWorks;
