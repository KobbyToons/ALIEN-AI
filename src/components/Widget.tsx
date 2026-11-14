import { useState, useEffect, useRef, useCallback } from "react";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { listen } from "@tauri-apps/api/event";
import TitleBar from "./TitleBar";
import BuddyOrb from "./BuddyOrb";
import ChatPanel from "./ChatPanel";
import ApprovalCard from "./ApprovalCard";
import TaskProgress from "./TaskProgress";
import QuestionCard from "./QuestionCard";
import SettingsModal from "./SettingsModal";
import { usePermissions } from "../hooks/usePermissions";
import { useChat } from "../hooks/useChat";
import { useVoice } from "../hooks/useVoice";
import { answerQuestion, getNextQuestion, getVaultInfo, resizeWindow } from "../lib/tauri";
import type { BuddyState, ScenarioQuestion, WidgetMode } from "../types";

/**
 * The main floating widget container for ALIEN.
 * Controls:
 * - Hold LMB to Speak through Mic (with realtime transcription)
 * - LMB to open Chat and Type
 * - LMB on Orb to close Chat when opened
 * - "`" Global shortcut / hotkey to suddenly summon voice transcription
 * - Holographic glitch entity in orb mode
 * - Soft rounded gradient window glow aura
 */
export default function Widget() {
  const [mode, setMode] = useState<WidgetMode>("orb");
  const [buddyState, setBuddyState] = useState<BuddyState>("idle");
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [activeQuestion, setActiveQuestion] = useState<ScenarioQuestion | null>(null);
  const [bubbleDismissed, setBubbleDismissed] = useState(false);

  // Push-to-talk hold and window dragging tracking
  const [isHoldingVoice, setIsHoldingVoice] = useState(false);
  const isHoldingVoiceRef = useRef(false);
  const holdTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const dragStartPosRef = useRef<{ x: number; y: number } | null>(null);
  const isDraggingWindowRef = useRef(false);

  const { requests, respond } = usePermissions();

  const {
    messages,
    agentSteps,
    isStreaming,
    modelConfig,
    updateModelConfig,
    sendMessage,
    clearMessages,
  } = useChat((st) => {
    setBuddyState(st);
  });

  const {
    isListening,
    isSpeaking,
    audioLevel,
    waveformBars,
    liveTranscript,
    voiceConfig,
    availableVoices,
    updateVoiceConfig,
    startListening,
    stopListening,
    speak,
    stopSpeaking,
  } = useVoice({
    onTranscript: (text) => {
      setBubbleDismissed(false);
      sendMessage(text);
    },
    onListeningChange: (listening) => {
      if (listening) {
        setBubbleDismissed(false);
        setBuddyState("listening");
      } else if (buddyState === "listening") {
        setBuddyState("idle");
      }
    },
    onSpeakingChange: (speaking) => {
      if (speaking) setBuddyState("speaking");
      else if (buddyState === "speaking") setBuddyState("idle");
    },
  });

  // Re-open bubble whenever new messages or streaming events occur
  useEffect(() => {
    if (messages.length > 0 || isStreaming || isListening) {
      setBubbleDismissed(false);
    }
  }, [messages.length, isStreaming, isListening]);

  // Initial load: verify vault connection
  useEffect(() => {
    getVaultInfo().catch(() => { });
  }, []);

  const lastMsg = messages[messages.length - 1];
  const hasBubbleContent =
    isListening ||
    isStreaming ||
    (lastMsg && lastMsg.role === "assistant" && lastMsg.content.trim().length > 0);

  const showBubbleInOrb = mode === "orb" && !bubbleDismissed && hasBubbleContent;

  // Dynamically resize Tauri window based on mode and bubble visibility
  // Providing comfortable padding so outer glow shadow curves without hitting rectangular window bounds
  useEffect(() => {
    const resize = async () => {
      try {
        if (mode === "orb") {
          if (showBubbleInOrb) {
            await resizeWindow(380, 260);
          } else {
            await resizeWindow(140, 140);
          }
        } else {
          await resizeWindow(380, 520);
        }
      } catch (err) {
        console.error("Window resize error:", err);
      }
    };
    resize();
  }, [mode, showBubbleInOrb]);

  // Global "`" shortcut listener from Rust backend & window-level keyboard listener
  useEffect(() => {
    const unlistenPromise = listen("buddy://start-voice", () => {
      if (isSpeaking) stopSpeaking();
      setMode("chat");
      if (!isListening) {
        setBubbleDismissed(false);
        startListening();
      } else {
        stopListening();
      }
    });

    const handleKeyDown = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement | null;
      const isEditing =
        target?.tagName === "INPUT" ||
        target?.tagName === "TEXTAREA" ||
        target?.isContentEditable;

      if (e.key === "`" && !isEditing) {
        e.preventDefault();
        if (isSpeaking) stopSpeaking();
        setMode("chat");
        if (!isListening) {
          setBubbleDismissed(false);
          startListening();
        } else {
          stopListening();
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);

    return () => {
      unlistenPromise.then((f) => f());
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [isListening, isSpeaking, startListening, stopListening, stopSpeaking]);

  // Reveal window if permission request or autonomous task arrives
  useEffect(() => {
    if (requests.length > 0 || agentSteps.length > 0) {
      if (mode === "orb") {
        setMode("chat");
      }
    }
  }, [requests.length, agentSteps.length, mode]);

  // 2-Hour Cooldown for "Getting to know you" questions
  useEffect(() => {
    const timer = setTimeout(() => {
      if (mode === "full" && requests.length === 0 && agentSteps.length === 0 && !isStreaming) {
        const lastQuestionTime = localStorage.getItem("alien_last_question_time");
        const TWO_HOURS_MS = 2 * 60 * 60 * 1000;
        const now = Date.now();

        if (lastQuestionTime && now - Number(lastQuestionTime) < TWO_HOURS_MS) {
          return;
        }

        getNextQuestion()
          .then((q) => {
            if (q) {
              setActiveQuestion(q);
              localStorage.setItem("alien_last_question_time", now.toString());
            }
          })
          .catch(() => { });
      }
    }, 4000);
    return () => clearTimeout(timer);
  }, [mode, requests.length, agentSteps.length, isStreaming]);

  const handleAnswerQuestion = async (qId: string, ans: string) => {
    try {
      setActiveQuestion(null);
      localStorage.setItem("alien_last_question_time", Date.now().toString());
      await answerQuestion(qId, ans);
    } catch (e) {
      console.error("Failed to record answer:", e);
    }
  };

  const handleDismissQuestion = () => {
    setActiveQuestion(null);
    localStorage.setItem("alien_last_question_time", Date.now().toString());
  };

  // Automatically speak the assistant's finished reply
  useEffect(() => {
    if (!isStreaming && messages.length > 0) {
      const last = messages[messages.length - 1];
      if (
        last &&
        last.role === "assistant" &&
        last.id !== "welcome" &&
        !last.isStreaming &&
        !last.content.startsWith("⚠️")
      ) {
        speak(last.content);
      }
    }
  }, [isStreaming, messages, speak]);

  // ── Orb Mouse User Controls ─────────────────────────────────────────
  // - Hold LMB to Speak through Mic (with realtime transcription)
  // - LMB to open Chat and Type
  // - LMB on Orb to close Chat when opened
  const handleOrbMouseDown = useCallback(
    (e: React.MouseEvent) => {
      if (e.button !== 0) return; // Only primary (left) button
      dragStartPosRef.current = { x: e.screenX, y: e.screenY };
      isDraggingWindowRef.current = false;

      // Start push-to-talk hold timer
      holdTimerRef.current = setTimeout(() => {
        if (!isDraggingWindowRef.current) {
          isHoldingVoiceRef.current = true;
          setIsHoldingVoice(true);
          setMode("chat");
          startListening();
        }
      }, 240);
    },
    [startListening]
  );

  const handleOrbMouseMove = useCallback((e: React.MouseEvent) => {
    if (!dragStartPosRef.current) return;

    const dx = e.screenX - dragStartPosRef.current.x;
    const dy = e.screenY - dragStartPosRef.current.y;
    const dist = Math.hypot(dx, dy);

    if (dist > 6) {
      isDraggingWindowRef.current = true;
      if (holdTimerRef.current) {
        clearTimeout(holdTimerRef.current);
        holdTimerRef.current = null;
      }
      if (isHoldingVoiceRef.current) {
        isHoldingVoiceRef.current = false;
        setIsHoldingVoice(false);
      }
      try {
        const win = getCurrentWebviewWindow();
        win.startDragging();
      } catch (_) {}
    }
  }, []);

  const handleOrbMouseUp = useCallback(
    (e: React.MouseEvent) => {
      if (e.button !== 0) return;

      if (holdTimerRef.current) {
        clearTimeout(holdTimerRef.current);
        holdTimerRef.current = null;
      }

      const wasDragging = isDraggingWindowRef.current;
      dragStartPosRef.current = null;
      isDraggingWindowRef.current = false;

      if (wasDragging) {
        return;
      }

      // If user was actively holding to talk (push-to-talk):
      if (isHoldingVoiceRef.current) {
        isHoldingVoiceRef.current = false;
        setIsHoldingVoice(false);
        stopListening();
        return;
      }

      // Standard Click / Tap:
      if (mode === "orb") {
        // LMB to open Chat and Type
        setMode("chat");
      } else {
        // LMB on Orb to close Chat when opened
        setMode("orb");
      }
    },
    [mode, stopListening]
  );

  const handleOrbDoubleClick = useCallback((e: React.MouseEvent) => {
    e.stopPropagation();
    setMode("chat");
  }, []);

  // Right Click on Orb -> Toggle Full App Window / Settings
  const handleOrbContextMenu = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setMode((prev) => (prev === "full" ? "orb" : "full"));
  }, []);

  const hasRequests = requests.length > 0;
  const isWorking = agentSteps.length > 0;
  const hasQuestion = activeQuestion !== null;

  // ── Mode 1: ORB-ONLY (With Floating Dialog Speech Bubble & Hologram Glitch) ─
  if (mode === "orb") {
    return (
      <div
        className={`flex flex-col items-center justify-center select-none bg-transparent ${
          showBubbleInOrb ? "w-[380px] h-[260px] pb-2 justify-end" : "w-[140px] h-[140px]"
        }`}
      >
        {/* Floating Dialog Speech Bubble */}
        {showBubbleInOrb && (
          <div
            onClick={() => setMode("chat")}
            className="
              relative w-[340px] mb-2 p-3.5 rounded-2xl
              bg-[#0c0c0e] border border-red-700/80
              shadow-2xl shadow-black/90
              cursor-pointer hover:border-red-500
              transition-all duration-200 animate-fade-in
            "
            title="Click to open chat & type"
          >
            {/* Header */}
            <div className="flex items-center justify-between pb-1.5 border-b border-red-950/80">
              <span className="text-red-500 font-extrabold text-[11px] tracking-wider flex items-center gap-1.5">
                <span className="w-2 h-2 rounded-full bg-red-500 animate-pulse" />
                ALIEN
              </span>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  setBubbleDismissed(true);
                }}
                className="text-neutral-400 hover:text-red-400 text-xs px-1 rounded transition-colors"
                title="Dismiss bubble"
              >
                ✕
              </button>
            </div>

            {/* Bubble Message Text & Audio Waveform / Live Speech */}
            <div className="mt-2 text-neutral-100 text-[12px] font-normal leading-relaxed max-h-[85px] overflow-y-auto chat-scroll pr-1">
              {isListening ? (
                <div className="flex flex-col gap-1.5 py-0.5">
                  <div className="flex items-center justify-between">
                    <span className="flex items-center gap-1.5 text-red-400 font-bold text-[11px]">
                      <span className="w-2 h-2 rounded-full bg-red-500 animate-ping" />
                      Listening to your voice...
                    </span>
                    {/* Live equalizer waveform bars */}
                    <div className="flex items-center gap-1 h-3.5">
                      {waveformBars.map((bar, i) => (
                        <span
                          key={i}
                          className="w-1 bg-gradient-to-t from-red-600 to-rose-400 rounded-full transition-all duration-75"
                          style={{
                            height: `${Math.max(4, Math.min(16, Math.round(bar * 16)))}px`,
                          }}
                        />
                      ))}
                    </div>
                  </div>
                  {/* Live speech transcription */}
                  <div className="text-neutral-100 text-[11px] font-medium italic min-h-[20px] bg-red-950/40 px-2 py-1.5 rounded border border-red-800/40">
                    {liveTranscript ? `“${liveTranscript}”` : "Speak now — words transcribe live…"}
                  </div>
                </div>
              ) : isStreaming ? (
                <div>
                  {lastMsg?.content}
                  <span className="inline-block w-1.5 h-3.5 ml-1 bg-red-500 animate-pulse align-middle" />
                </div>
              ) : (
                lastMsg?.content
              )}
            </div>

            {/* Footer Hint */}
            <div className="mt-2 pt-1.5 border-t border-neutral-900 flex items-center justify-between text-[10px] text-neutral-400">
              <span className="hover:text-red-400 font-medium">
                💬 LMB to open chat & type
              </span>
              <span className="text-[9px] text-neutral-400">
                Hold LMB or press ` to speak
              </span>
            </div>

            {/* Speech bubble arrow pointer pointing to orb */}
            <div className="absolute -bottom-1.5 left-1/2 -translate-x-1/2 w-3 h-3 bg-[#0c0c0e] border-r border-b border-red-700/80 rotate-45" />
          </div>
        )}

        {/* The Audio-Reactive Glowing Holographic ALIEN Logo */}
        <div data-tauri-drag-region>
          <BuddyOrb
            state={buddyState}
            size="lg"
            isWindowMode={false}
            audioLevel={audioLevel}
            waveformBars={waveformBars}
            isHoldingVoice={isHoldingVoice}
            onMouseDown={handleOrbMouseDown}
            onMouseMove={handleOrbMouseMove}
            onMouseUp={handleOrbMouseUp}
            onDoubleClick={handleOrbDoubleClick}
            onContextMenu={handleOrbContextMenu}
          />
        </div>
      </div>
    );
  }

  // ── Mode 2: FULL CHAT VIEW (Expanded on Click or Voice Activation) ──
  if (mode === "chat") {
    return (
      <div
        className="
          relative flex flex-col
          w-[380px] h-[520px]
          rounded-[28px] overflow-hidden
          bg-[#08080b]
          border border-red-900/80
          transition-all duration-200
        "
      >
        {/* Compact Chat Header */}
          <div
            data-tauri-drag-region
            className="
              flex items-center justify-between
              h-11 px-3.5 select-none cursor-move
              bg-[#050507] border-b border-red-950/80 text-neutral-400
            "
          >
            <div className="flex items-center gap-2.5" data-tauri-drag-region>
              <div
                title="LMB to close Chat back to Orb"
                className="cursor-pointer"
              >
                <BuddyOrb
                  state={buddyState}
                  size="sm"
                  isWindowMode={true}
                  audioLevel={audioLevel}
                  waveformBars={waveformBars}
                  isHoldingVoice={isHoldingVoice}
                  onMouseDown={handleOrbMouseDown}
                  onMouseMove={handleOrbMouseMove}
                  onMouseUp={handleOrbMouseUp}
                  onDoubleClick={handleOrbDoubleClick}
                  onContextMenu={handleOrbContextMenu}
                />
              </div>
              <div>
                <div className="text-red-500 font-black text-xs tracking-wider">ALIEN</div>
                <div className="text-[9px] text-neutral-400">
                  {isListening
                    ? "Listening to voice…"
                    : isSpeaking
                      ? "Speaking (tap to mute)…"
                      : isStreaming
                        ? "Thinking…"
                        : "Chat"}
                </div>
              </div>
            </div>

            <div className="flex items-center gap-1">
              <button
                onClick={() => setMode("full")}
                className="p-1 rounded-md hover:bg-neutral-800 text-neutral-400 hover:text-red-400 text-xs"
                title="Open Full Window & Settings"
              >
                ⚙️
              </button>
              <button
                onClick={() => setMode("orb")}
                className="p-1 rounded-md hover:bg-neutral-800 text-neutral-400 hover:text-red-400 text-xs"
                title="Collapse to Orb (or click logo)"
              >
                ⚪
              </button>
              <button
                onClick={async () => {
                  const win = getCurrentWebviewWindow();
                  await win.hide();
                }}
                className="p-1 px-1.5 rounded-md hover:bg-neutral-800 text-neutral-400 hover:text-red-400 text-xs font-bold"
                title="Minimize to Tray"
              >
                ─
              </button>
            </div>
          </div>

          {/* Task Progress Timeline */}
          <TaskProgress steps={agentSteps} />

          {/* Permission Requests */}
          {hasRequests && (
            <div className="px-3.5 py-1.5 space-y-2 overflow-y-auto max-h-[140px] chat-scroll">
              {requests.map((req) => (
                <ApprovalCard key={req.id} request={req} onRespond={respond} />
              ))}
            </div>
          )}

          {/* Chat Panel with multiline word processing & caret always available */}
          <ChatPanel
            messages={messages}
            onSend={(text, attachments) => {
              stopSpeaking();
              sendMessage(text, attachments);
            }}
            isStreaming={isStreaming}
            isListening={isListening}
            isSpeaking={isSpeaking}
            liveTranscript={liveTranscript}
            waveformBars={waveformBars}
            onStartListening={startListening}
            onStopListening={stopListening}
            onStopSpeaking={stopSpeaking}
          />
      </div>
    );
  }

  // ── Mode 3: FULL APP WINDOW (Opened via Right Click) ──────────────
  return (
    <div
      className="
        relative flex flex-col
        w-[380px] h-[520px]
        rounded-[28px] overflow-hidden
        bg-[#08080b]
        border border-red-900/80
        transition-all duration-200
      "
    >
      <TitleBar
          onOpenSettings={() => setSettingsOpen(true)}
          onCollapse={() => setMode("orb")}
        />

        {/* Main Orb Status Area */}
        <div className="flex flex-col items-center px-4 pt-2 pb-3">
          <BuddyOrb
            state={buddyState}
            size="md"
            isWindowMode={true}
            audioLevel={audioLevel}
            waveformBars={waveformBars}
            isHoldingVoice={isHoldingVoice}
            onMouseDown={handleOrbMouseDown}
            onMouseMove={handleOrbMouseMove}
            onMouseUp={handleOrbMouseUp}
            onDoubleClick={handleOrbDoubleClick}
            onContextMenu={handleOrbContextMenu}
          />
          <p className="mt-2 text-[11px] font-medium text-neutral-300 select-none">
            {isListening
              ? "Listening to you…"
              : isSpeaking
                ? "Speaking (tap orb to mute)…"
                : hasRequests
                  ? "Requires your approval"
                  : hasQuestion
                    ? "Quick question for you ✨"
                    : isWorking
                      ? "Executing autonomous plan…"
                      : isStreaming
                        ? "ALIEN is thinking…"
                        : "LMB on logo to chat • Right-click to collapse"}
          </p>

          <div className="flex gap-2 mt-2">
            <button
              onClick={() => setMode("chat")}
              className="px-3 py-1 rounded-xl bg-red-600 hover:bg-red-500 text-white text-[11px] font-semibold transition-colors shadow-md shadow-red-600/30"
            >
              Open Chat
            </button>
            <button
              onClick={() => setSettingsOpen(true)}
              className="px-3 py-1 rounded-xl bg-[#141418] hover:bg-[#1d1d24] text-neutral-200 text-[11px] font-medium border border-neutral-700 transition-colors"
            >
              Brain & Settings
            </button>
          </div>
        </div>

        {/* Getting to Know You Card (Randomized & 2-Hour Cooldown) */}
        {hasQuestion && (
          <QuestionCard
            question={activeQuestion!}
            onAnswer={handleAnswerQuestion}
            onDismiss={handleDismissQuestion}
          />
        )}

        {/* Task Progress Timeline */}
        <TaskProgress steps={agentSteps} />

        {/* Permission Requests list */}
        {hasRequests && (
          <div className="px-3.5 py-1.5 space-y-2 overflow-y-auto max-h-[140px] chat-scroll">
            {requests.map((req) => (
              <ApprovalCard key={req.id} request={req} onRespond={respond} />
            ))}
          </div>
        )}

        {/* Settings Modal Drawer */}
        <SettingsModal
          isOpen={settingsOpen}
          onClose={() => setSettingsOpen(false)}
          onClearChat={clearMessages}
          modelConfig={modelConfig}
          onUpdateModelConfig={updateModelConfig}
          voiceConfig={voiceConfig}
          availableVoices={availableVoices}
          onUpdateVoiceConfig={updateVoiceConfig}
        />
    </div>
  );
}
