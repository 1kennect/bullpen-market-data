/** The thread glyph Discord puts beside a thread name. */
export const ThreadIcon = ({ className = "h-5 w-5" }: { className?: string }) => (
  <svg aria-hidden viewBox="0 0 24 24" className={className} fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
    <path d="M4 5.5A2.5 2.5 0 0 1 6.5 3h11A2.5 2.5 0 0 1 20 5.5v8a2.5 2.5 0 0 1-2.5 2.5H11l-4.5 4v-4h0A2.5 2.5 0 0 1 4 13.5v-8Z" />
    <path d="M8.5 8h7M8.5 11.5h4.5" />
  </svg>
);
