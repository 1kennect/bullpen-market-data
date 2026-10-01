import { Link } from "react-router-dom";

/** Any path the site does not serve. */
const NotFound = () => (
  <main className="chart-grid grid min-h-screen place-items-center px-5">
    <div className="flex flex-col items-center rounded-xl border border-ink/15 bg-white px-10 py-12 text-center">
      <img src="/brand/bullpen-logo.svg" alt="" width={64} height={64} className="h-16 w-16" />
      <h1 className="mt-6 text-3xl font-black">Page not found</h1>
      <p className="mt-3 text-sm text-ink-soft">That symbol is not listed here.</p>
      <Link
        to="/"
        className="mt-8 inline-flex h-11 items-center rounded-full bg-accent px-6 text-sm font-semibold text-paper transition-transform hover:-translate-y-0.5"
      >
        Back to home
      </Link>
    </div>
  </main>
);

export default NotFound;
