import GithubMark from "./GithubMark";
import { REPO, ROADMAP } from "./content";

/** Bottom line: what it is, what it is not, and where the source is. */
const Footer = () => (
  <footer className="border-t border-ink/10 px-5 py-8 sm:px-8">
    <div className="mx-auto flex max-w-6xl flex-col items-center gap-4 text-xs text-ink-soft sm:flex-row sm:justify-between">
      <div className="flex flex-wrap items-center justify-center gap-2.5">
        <img src="/brand/bullpen-logo.svg" alt="" className="h-5 w-5" />
        <span className="font-semibold text-ink">Bullpen</span>
        <span>Market information, not investment advice. Data may be delayed.</span>
      </div>
      <div className="flex items-center gap-6">
        <a href={ROADMAP} target="_blank" rel="noreferrer" className="transition-colors hover:text-accent">
          Roadmap
        </a>
        <a
          href={REPO}
          target="_blank"
          rel="noreferrer"
          className="inline-flex items-center gap-1.5 transition-colors hover:text-accent"
        >
          <GithubMark className="h-3.5 w-3.5" />
          GitHub
        </a>
      </div>
    </div>
  </footer>
);

export default Footer;
