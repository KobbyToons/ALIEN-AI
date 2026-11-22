/**
 * AgentOverlay — Visual Presence Indicator
 *
 * Renders a subtle red edge glow along all four borders of the screen.
 * Excluded from screenshots on the OS level via SetWindowDisplayAffinity.
 * Completely click-through and non-interactive.
 */
export default function AgentOverlay() {
  return (
    <div className="fixed inset-0 pointer-events-none overflow-hidden select-none">
      {/* Edge glow container with slow 3s breathing pulse */}
      <div className="w-full h-full relative animate-presence-pulse">
        {/* Top Edge */}
        <div
          className="absolute top-0 left-0 right-0 h-28 pointer-events-none"
          style={{
            background:
              "linear-gradient(to bottom, rgba(255, 40, 40, 0.38) 0%, rgba(255, 40, 40, 0.15) 45%, transparent 100%)",
          }}
        />

        {/* Bottom Edge */}
        <div
          className="absolute bottom-0 left-0 right-0 h-28 pointer-events-none"
          style={{
            background:
              "linear-gradient(to top, rgba(255, 40, 40, 0.38) 0%, rgba(255, 40, 40, 0.15) 45%, transparent 100%)",
          }}
        />

        {/* Left Edge */}
        <div
          className="absolute top-0 bottom-0 left-0 w-28 pointer-events-none"
          style={{
            background:
              "linear-gradient(to right, rgba(255, 40, 40, 0.38) 0%, rgba(255, 40, 40, 0.15) 45%, transparent 100%)",
          }}
        />

        {/* Right Edge */}
        <div
          className="absolute top-0 bottom-0 right-0 w-28 pointer-events-none"
          style={{
            background:
              "linear-gradient(to left, rgba(255, 40, 40, 0.38) 0%, rgba(255, 40, 40, 0.15) 45%, transparent 100%)",
          }}
        />

        {/* Corner Accents for Smooth Blending */}
        <div
          className="absolute top-0 left-0 w-36 h-36 pointer-events-none"
          style={{
            background:
              "radial-gradient(circle at top left, rgba(255, 40, 40, 0.45) 0%, transparent 70%)",
          }}
        />
        <div
          className="absolute top-0 right-0 w-36 h-36 pointer-events-none"
          style={{
            background:
              "radial-gradient(circle at top right, rgba(255, 40, 40, 0.45) 0%, transparent 70%)",
          }}
        />
        <div
          className="absolute bottom-0 left-0 w-36 h-36 pointer-events-none"
          style={{
            background:
              "radial-gradient(circle at bottom left, rgba(255, 40, 40, 0.45) 0%, transparent 70%)",
          }}
        />
        <div
          className="absolute bottom-0 right-0 w-36 h-36 pointer-events-none"
          style={{
            background:
              "radial-gradient(circle at bottom right, rgba(255, 40, 40, 0.45) 0%, transparent 70%)",
          }}
        />
      </div>

      <style>{`
        @keyframes presence-pulse {
          0%, 100% {
            opacity: 0.65;
          }
          50% {
            opacity: 1.0;
          }
        }
        .animate-presence-pulse {
          animation: presence-pulse 3s ease-in-out infinite;
        }
        @media (prefers-reduced-motion: reduce) {
          .animate-presence-pulse {
            animation: none;
            opacity: 0.85;
          }
        }
      `}</style>
    </div>
  );
}
