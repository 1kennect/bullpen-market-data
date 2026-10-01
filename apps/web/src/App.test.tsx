import { render, screen, within } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";
import App from "./App";
import { REPO } from "./components/site/content";

describe("website routes", () => {
  beforeEach(() => {
    window.history.pushState(null, "", "/");
  });

  it("leads with what Bullpen is and how to run it", () => {
    render(<App />);

    const main = within(screen.getByRole("main"));
    expect(main.getByRole("heading", { level: 1 })).toHaveTextContent(/market answers,\s*with receipts/i);
    expect(main.getByRole("link", { name: /run bullpen/i })).toHaveAttribute("href", `${REPO}#run-it`);
    expect(main.getByText(/not investment advice/i)).toBeInTheDocument();
  });

  it("shows every section of the product", () => {
    render(<App />);

    for (const label of [/introduction/i, /what people ask/i, /how it works/i, /open source/i]) {
      expect(screen.getByRole("region", { name: label })).toBeInTheDocument();
    }
  });

  it("shows example threads with the market tools and links to their sources", () => {
    render(<App />);

    const examples = within(screen.getByRole("region", { name: /what people ask/i }));
    expect(examples.getAllByText(/market_profile/).length).toBeGreaterThan(0);
    expect(examples.getByRole("link", { name: /palantir technologies inc/i })).toHaveAttribute(
      "href",
      "https://www.palantir.com/",
    );
  });

  it("links the navigation to each section and the repository", () => {
    render(<App />);

    const nav = within(screen.getByRole("navigation", { name: /primary/i }));
    for (const id of ["examples", "how-it-works", "open-source"]) {
      expect(document.getElementById(id)).not.toBeNull();
      expect(nav.getByRole("link", { name: new RegExp(id.replace(/-/g, " "), "i") })).toHaveAttribute(
        "href",
        `#${id}`,
      );
    }
    expect(nav.getByRole("link", { name: /github/i })).toHaveAttribute("href", REPO);
  });

  it("serves a not found page for unknown paths", () => {
    window.history.pushState(null, "", "/old");

    render(<App />);

    expect(screen.getByRole("heading", { name: /page not found/i })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: /back to home/i })).toHaveAttribute("href", "/");
  });
});
