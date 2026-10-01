/** Who wrote a message, as Discord shows them. */
export type Author = { name: string; bot?: boolean; color?: string; initial?: string };

/** A button under a bot message: a link when it has href, else a press button in its style. */
export type LinkButton = { label: string; href?: string; style?: "danger" | "secondary" };

/** The bot, as its Discord application shows it. */
export const BULLPEN: Author = { name: "Bullpen", bot: true };

/** People who appear in the examples. */
export const PEOPLE = {
  maya: { name: "maya", color: "#f0b232" },
  devon: { name: "devon", color: "#23a55a" },
  ren: { name: "ren", color: "#eb459e" },
  sam: { name: "sam", color: "#00a8fc" },
  jordan: { name: "jordan", color: "#f0b232" },
} satisfies Record<string, Author>;
