/** The repository every source link on the site points to. Set VITE_REPO_URL at build time. */
export const REPO: string = (import.meta.env.VITE_REPO_URL || "https://github.com/OWNER/bullpen").replace(/\/$/, "");

/** How to run it, in the README. */
export const RUN = `${REPO}#run-it`;

/** The architecture write-up. */
export const ARCHITECTURE = `${REPO}/blob/main/docs/ARCHITECTURE.md`;

/** The roadmap. */
export const ROADMAP = `${REPO}/blob/main/docs/ROADMAP.md`;
