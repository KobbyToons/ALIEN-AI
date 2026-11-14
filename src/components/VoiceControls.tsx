import { useRef, useCallback } from "react";

interface VoiceControlsProps {
  isListening: boolean;
  isSpeaking: boolean;
  onStartListening: () => void;
  onStopListening: () => void;
  onStopSpeaking: () => void;
}

/**
 * Microphone voice button for the chat input bar.
 * Supports:
 * - Tap to toggle voice listening on/off
 * - Press & hold (> 260ms) for Push-to-Talk
 * - Tap while speaking to mute
 */
export default function VoiceControls({
  isListening,
  isSpeaking,
  onStartListening,
  onStopListening,
  onStopSpeaking,
}: VoiceControlsProps) {
  const holdTimerRef = useRef<NodeJS.Timeout | null>(null);
  const isHoldingRef = useRef(false);

  const handleMouseDown = useCallback(() => {
    if (isSpeaking) {
      onStopSpeaking();
      return;
    }

    if (isListening) {
      onStopListening();
      return;
    }

    // Set hold timer for push-to-talk
    holdTimerRef.current = setTimeout(() => {
      isHoldingRef.current = true;
      onStartListening();
    }, 260);
  }, [isListening, isSpeaking, onStartListening, onStopListening, onStopSpeaking]);

  const handleMouseUp = useCallback(() => {
    if (holdTimerRef.current) {
      clearTimeout(holdTimerRef.current);
      holdTimerRef.current = null;
    }

    if (isHoldingRef.current) {
      // Was push-to-talk hold -> stop listening upon release
      isHoldingRef.current = false;
      onStopListening();
    } else if (!isListening && !isSpeaking) {
      // Was a quick tap -> toggle listening on
      onStartListening();
    }
  }, [isListening, isSpeaking, onStartListening, onStopListening]);

  if (isSpeaking) {
    return (
      <button
        type="button"
        onClick={onStopSpeaking}
        className="
          p-2 rounded-xl transition-all shadow-sm
          bg-red-600 hover:bg-red-700 text-white animate-pulse
        "
        title="Stop ALIEN from speaking (Mute)"
        aria-label="Stop speaking"
      >
        {/* Stop / Mute Icon */}
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" className="w-4 h-4">
          <rect x="5" y="5" width="10" height="10" rx="1.5" />
        </svg>
      </button>
    );
  }

  return (
    <button
      type="button"
      onMouseDown={handleMouseDown}
      onMouseUp={handleMouseUp}
      onTouchStart={handleMouseDown}
      onTouchEnd={handleMouseUp}
      className={`
        p-2 rounded-xl transition-all shadow-sm
        ${
          isListening
            ? "bg-red-600 text-white ring-4 ring-red-500/40 animate-pulse"
            : "bg-[#141418] hover:bg-[#202028] text-neutral-300 hover:text-white border border-neutral-700"
        }
      `}
      title={isListening ? "Listening... (Click to send or hold)" : "Voice: Click to speak or hold (Push to talk)"}
      aria-label="Voice input"
    >
      {/* Microphone Icon */}
      <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20" fill="currentColor" className="w-4 h-4">
        <path d="M10 2a3 3 0 0 0-3 3v6a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z" />
        <path d="M13 11a3 3 0 1 1-6 0 1 1 0 0 0-2 0 5 5 0 0 0 4 4.9V17H8a1 1 0 1 0 0 2h4a1 1 0 1 0 0-2h-1v-1.1a5 5 0 0 0 4-4.9 1 1 0 0 0-2 0Z" />
      </svg>
    </button>
  );
}
