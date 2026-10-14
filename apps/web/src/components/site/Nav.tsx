import { Link } from "react-router-dom";
import GithubMark from "./GithubMark";
import { REPO } from "./content";

const LINKS = [
  { href: "#examples", label: "Examples" },
  { href: "#how-it-works", label: "How it works" },
  { href: "#open-source", label: "Open source" },
];

/** Top bar: mark, section links, and the repository. */
const Nav = () => (
  <header className="fixed inset-x-0 top-0 z-30 border-b border-ink/10 bg-paper/85 backdrop-blur-md">
    <nav
      aria-label="Primary navigation"
      className="mx-auto flex h-16 max-w-6xl items-center justify-between px-5 sm:px-8"
    >
      <Link to="/" className="flex items-center gap-2.5" aria-label="Bullpen home">
        <img src="/brand/bullpen-logo.svg" alt="" className="h-8 w-8" />
        <span className="text-lg font-bold tracking-tight">Bullpen</span>
      </Link>

      <div className="flex items-center gap-7">
        <ul className="hidden items-center gap-7 md:flex">
          {LINKS.map((link) => (
            <li key={link.href}>
              <a href={link.href} className="text-sm text-ink-soft transition-colors hover:text-accent">
                {link.label}
              </a>
            </li>
          ))}
        </ul>
        <a
          href={REPO}
          target="_blank"
          rel="noreferrer"
          aria-label="GitHub repository"
          className="grid h-9 w-9 place-items-center rounded-full border border-ink/15 text-ink transition-colors hover:border-accent hover:text-accent"
        >
          <GithubMark className="h-4 w-4" />
        </a>
      </div>
    </nav>
  </header>
);

export default Nav;
