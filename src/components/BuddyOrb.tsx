import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { getWatchModeStatus } from "../lib/tauri";
import type { BuddyState } from "../types";
import alienLogo from "../assets/alien-logo.png";

interface BuddyOrbProps {
  state: BuddyState;
  size: "sm" | "md" | "lg";
  isWindowMode?: boolean;
  audioLevel?: number;
  waveformBars?: number[];
  onClick?: (e: React.MouseEvent) => void;
  onDoubleClick?: (e: React.MouseEvent) => void;
  onMouseDown?: (e: React.MouseEvent) => void;
  onMouseUp?: (e: React.MouseEvent) => void;
  onMouseMove?: (e: React.MouseEvent) => void;
  onMouseLeave?: (e: React.MouseEvent) => void;
  onContextMenu?: (e: React.MouseEvent) => void;
  isHoldingVoice?: boolean;
}

const orbAnimClass: Record<BuddyState, string> = {
  idle: "orb-idle",
  listening: "orb-listening",
  thinking: "orb-thinking",
  speaking: "orb-speaking",
  working: "orb-working",
};

/**
 * The animated ALIEN entity — official glowing holographic ALIEN Logo.
 * Features:
 * - Hologram horizontal glitch in a random sequence (cyan/red split & horizontal slice jitter).
 * - Active glowing red aura reacting to voice audio levels in real time.
 * - Dynamic equalizer waveform bars under chin while listening.
 * - Glowing red ember particle effects drifting behind the logo in orb mode.
 * - Gentle levitating float animation in orb mode.
 */
export default function BuddyOrb({
  state,
  size,
  isWindowMode = false,
  audioLevel = 0,
  waveformBars,
  onClick,
  onDoubleClick,
  onMouseDown,
  onMouseUp,
  onMouseMove,
  onMouseLeave,
  onContextMenu,
  isHoldingVoice = false,
}: BuddyOrbProps) {
  const [watchModeActive, setWatchModeActive] = useState(false);

  useEffect(() => {
    getWatchModeStatus().then(setWatchModeActive).catch(() => {});
    const unlistenPromise = listen<{ active: boolean }>("buddy://watch-mode-state", (event) => {
      setWatchModeActive(event.payload.active);
    });
    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, []);

  const px =
    size === "lg" ? "w-28 h-28" : size === "md" ? "w-16 h-16" : "w-9 h-9";

  const effectiveState = isHoldingVoice ? "listening" : state;
  const isAudioActive = effectiveState === "listening" || effectiveState === "speaking" || isHoldingVoice;
  const activeLevel = isAudioActive ? Math.max(audioLevel, 0.08) : 0;

  // Real-time audio waveform scale & glow
  const reactiveScale = 1 + Math.min(activeLevel * 0.35, 0.4);
  const glowBlur = 18 + Math.round(activeLevel * 30);
  const ringScale = 1 + Math.min(activeLevel * 0.65, 0.7);

  const isOrbMode = !isWindowMode && size === "lg";

  return (
    <div
      onClick={onClick}
      onDoubleClick={onDoubleClick}
      onMouseDown={onMouseDown}
      onMouseUp={onMouseUp}
      onMouseMove={onMouseMove}
      onMouseLeave={onMouseLeave}
      onContextMenu={onContextMenu}
      className={`
        relative cursor-pointer select-none
        transition-transform duration-75 ease-out
        focus:outline-none group flex items-center justify-center
        ${isOrbMode ? "animate-levitate" : ""}
        ${px}
      `}
      style={{
        transform: `scale(${reactiveScale})`,
      }}
      role="button"
      tabIndex={0}
      title={
        isOrbMode
          ? "ALIEN: Click for Voice Mode • Hold to Speak • Double-click for Chat • Right-click for Settings • Drag to Move"
          : "ALIEN: Click to collapse to Orb"
      }
      aria-label={`ALIEN is ${effectiveState}. Click to toggle voice mode, hold to speak, double click for chat.`}
    >
      {/* ── Watch Mode Eye Indicator Badge ── */}
      {watchModeActive && (
        <div
          className="absolute -top-1 -right-1 z-30 flex items-center justify-center w-6 h-6 rounded-full bg-[#050d18] border border-cyan-400/90 text-cyan-300 text-xs shadow-[0_0_10px_rgba(34,211,238,0.6)] animate-pulse"
          title="Watch Mode: ALIEN is observing your screen"
        >
          👁️
        </div>
      )}

      {/* ── Red Glowy Particle Effects (drifting behind ALIEN when in orb mode) ── */}
      {isOrbMode && (
        <div className="absolute -inset-6 pointer-events-none -z-20 overflow-visible">
          <span className="absolute w-2 h-2 rounded-full bg-red-500 blur-[0.5px] shadow-[0_0_12px_#ef4444] ember-p1 left-[20%] top-[45%]" />
          <span className="absolute w-2.5 h-2.5 rounded-full bg-rose-500 blur-[1px] shadow-[0_0_14px_#f43f5e] ember-p2 right-[18%] top-[50%]" />
          <span className="absolute w-1.5 h-1.5 rounded-full bg-red-400 blur-[0.5px] shadow-[0_0_9px_#ef4444] ember-p3 left-[32%] top-[65%]" />
          <span className="absolute w-2 h-2 rounded-full bg-orange-500 blur-[0.5px] shadow-[0_0_11px_#f97316] ember-p4 right-[26%] top-[58%]" />
          <span className="absolute w-1.5 h-1.5 rounded-full bg-red-600 blur-[0.5px] shadow-[0_0_10px_#dc2626] ember-p5 left-[48%] top-[38%]" />
          <span className="absolute w-2 h-2 rounded-full bg-rose-600 blur-[1px] shadow-[0_0_12px_#e11d48] ember-p6 right-[42%] top-[68%]" />
        </div>
      )}

      {/* Outer Glow Halo (Audio-Reactive) */}
      <div
        className={`
          absolute -inset-3 rounded-full blur-xl pointer-events-none -z-10
          transition-all duration-75
          ${
            effectiveState === "listening"
              ? "bg-red-500/75"
              : effectiveState === "speaking"
              ? "bg-rose-500/60"
              : "bg-red-600/35 group-hover:bg-red-500/50"
          }
        `}
        style={{
          transform: `scale(${ringScale})`,
          filter: `blur(${glowBlur}px)`,
          opacity: 0.45 + Math.min(activeLevel * 0.55, 0.55),
        }}
      />

      {/* Audio Reactive Waveform Ripple Ring */}
      {isAudioActive && (
        <span
          className="absolute -inset-3 rounded-full border border-red-400/60 pointer-events-none transition-transform duration-75 -z-10"
          style={{
            transform: `scale(${1 + activeLevel * 0.5})`,
            opacity: 0.45 + activeLevel * 0.55,
          }}
        />
      )}

      {/* Dynamic Voice Waveform Equalizer Aura under Chin */}
      {effectiveState === "listening" && waveformBars && waveformBars.length > 0 && (
        <div className="absolute -bottom-2.5 left-1/2 -translate-x-1/2 flex items-end gap-1 h-3.5 pointer-events-none z-20">
          {waveformBars.map((b, idx) => (
            <span
              key={idx}
              className="w-1 bg-gradient-to-t from-red-600 to-rose-400 rounded-full transition-all duration-75 shadow-sm shadow-red-500"
              style={{ height: `${Math.max(3, Math.min(14, Math.round(b * 14)))}px` }}
            />
          ))}
        </div>
      )}

      {/* Listening Pulse Rings */}
      {effectiveState === "listening" && (
        <>
          <span className="orb-ring absolute -inset-2 rounded-full border border-red-500/60 pointer-events-none animate-pulse-ring -z-10" />
          <span className="orb-ring absolute -inset-4 rounded-full border border-red-600/35 pointer-events-none animate-pulse-ring [animation-delay:0.35s] -z-10" />
        </>
      )}

      {/* ── Holographic Glitch Chromatic Slice Layers ── */}
      {isOrbMode && (
        <>
          {/* Cyan Slice Layer */}
          <img
            src={alienLogo}
            alt=""
            draggable={false}
            className="absolute inset-0 w-full h-full object-contain pointer-events-none select-none hologram-cyan-layer opacity-0"
          />
          {/* Red Slice Layer */}
          <img
            src={alienLogo}
            alt=""
            draggable={false}
            className="absolute inset-0 w-full h-full object-contain pointer-events-none select-none hologram-red-layer opacity-0"
          />
          {/* Fine horizontal holographic scanline overlay */}
          <div className="absolute inset-0 rounded-full hologram-scanlines opacity-40 pointer-events-none" />
        </>
      )}

      {/* ── Official ALIEN Logo Entity ── */}
      <img
        src={alienLogo}
        alt="ALIEN"
        draggable={false}
        className={`
          w-full h-full object-contain pointer-events-none select-none
          transition-all duration-75 ease-out
          ${orbAnimClass[effectiveState]}
          ${isOrbMode ? "hologram-active" : ""}
        `}
        style={{
          filter: `drop-shadow(0 0 ${10 + Math.round(activeLevel * 25)}px rgba(239, 68, 68, ${
            0.75 + Math.min(activeLevel * 0.25, 0.25)
          }))`,
        }}
      />
    </div>
  );
}
