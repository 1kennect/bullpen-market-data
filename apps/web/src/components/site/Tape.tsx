/** Symbols on the decorative tape. The moves are illustrative, not market data. */
const TAPE = [
  { symbol: "AAPL", move: 0.42 },
  { symbol: "MSFT", move: -0.31 },
  { symbol: "NVDA", move: 1.74 },
  { symbol: "AMZN", move: 0.88 },
  { symbol: "GOOGL", move: -0.12 },
  { symbol: "TSLA", move: -2.05 },
  { symbol: "META", move: 0.63 },
  { symbol: "SPY", move: 0.27 },
  { symbol: "QQQ", move: 0.51 },
  { symbol: "BRK.B", move: -0.08 },
];

/** A ticker tape that scrolls under the navigation bar. */
const Tape = () => (
  <div aria-hidden className="overflow-hidden border-y border-ink/10 bg-ink py-2 font-mono text-xs text-paper">
    <div className="flex w-max gap-8 motion-safe:animate-tape">
      {[...TAPE, ...TAPE].map((row, i) => (
        <span key={i} className="flex gap-2">
          <span className="font-semibold">{row.symbol}</span>
          <span className={row.move >= 0 ? "text-gain" : "text-loss"}>
            {row.move >= 0 ? "▲" : "▼"} {Math.abs(row.move).toFixed(2)}%
          </span>
        </span>
      ))}
    </div>
  </div>
);

export default Tape;
