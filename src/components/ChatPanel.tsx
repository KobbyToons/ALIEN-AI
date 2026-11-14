import { useState, useRef, useEffect } from "react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { convertFileSrc } from "@tauri-apps/api/core";
import VoiceControls from "./VoiceControls";
import type { ChatMessage, ChatAttachment } from "../types";

interface ChatPanelProps {
  messages: ChatMessage[];
  onSend: (text: string, attachments?: ChatAttachment[]) => void;
  isStreaming?: boolean;
  isListening: boolean;
  isSpeaking: boolean;
  liveTranscript?: string;
  waveformBars?: number[];
  onStartListening: () => void;
  onStopListening: () => void;
  onStopSpeaking: () => void;
}

/**
 * Expandable chat transcript panel with Black & Red styling, streaming tokens,
 * real-time voice waveform visualization, and live speech transcription.
 * Features:
 * - Caret (incision point) auto-focused on mount & when ALIEN finishes responding
 * - Textarea is NEVER disabled so clicking always reveals the caret immediately
 * - Shift+Enter for new line, Enter to send
 * - Highlightable & copyable chat text
 * - "+" Button with dropdown menu to attach images, files, or folders
 */
export default function ChatPanel({
  messages,
  onSend,
  isStreaming,
  isListening,
  isSpeaking,
  liveTranscript,
  waveformBars = [0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1],
  onStartListening,
  onStopListening,
  onStopSpeaking,
}: ChatPanelProps) {
  const [input, setInput] = useState("");
  const [attachments, setAttachments] = useState<ChatAttachment[]>([]);
  const [showAttachMenu, setShowAttachMenu] = useState(false);

  const scrollRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const attachMenuRef = useRef<HTMLDivElement>(null);

  // Auto-focus the incision point (caret) by default once chat mounts
  useEffect(() => {
    const timer = setTimeout(() => {
      textareaRef.current?.focus();
    }, 60);
    return () => clearTimeout(timer);
  }, []);

  // When ALIEN finishes streaming/responding, automatically return focus to the textarea
  useEffect(() => {
    if (!isStreaming) {
      textareaRef.current?.focus();
    }
  }, [isStreaming]);

  // Close attachment dropdown when clicking outside
  useEffect(() => {
    const handleOutsideClick = (e: MouseEvent) => {
      if (
        attachMenuRef.current &&
        !attachMenuRef.current.contains(e.target as Node)
      ) {
        setShowAttachMenu(false);
      }
    };
    if (showAttachMenu) {
      document.addEventListener("mousedown", handleOutsideClick);
    }
    return () => {
      document.removeEventListener("mousedown", handleOutsideClick);
    };
  }, [showAttachMenu]);

  // Auto-scroll on new messages or streamed tokens
  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTo({
        top: scrollRef.current.scrollHeight,
        behavior: "smooth",
      });
    }
  }, [messages, isStreaming]);

  // Adjust textarea height dynamically
  const adjustTextareaHeight = () => {
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
      textareaRef.current.style.height = `${Math.min(
        textareaRef.current.scrollHeight,
        120
      )}px`;
    }
  };

  const handleInputChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setInput(e.target.value);
    adjustTextareaHeight();
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  };

  const handleSubmit = (e?: React.FormEvent) => {
    if (e) e.preventDefault();
    const text = input.trim();
    if ((!text && attachments.length === 0) || isStreaming) return;

    onSend(text, attachments.length > 0 ? attachments : undefined);
    setInput("");
    setAttachments([]);
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
    }
  };

  // ── Attachment Pickers ──────────────────────────────────────────
  const handleAttachImage = async () => {
    setShowAttachMenu(false);
    try {
      const selected = await openDialog({
        multiple: true,
        filters: [
          {
            name: "Images",
            extensions: ["png", "jpg", "jpeg", "webp", "gif", "svg", "bmp"],
          },
        ],
      });

      if (!selected) return;
      const paths = Array.isArray(selected) ? selected : [selected];
      const newItems: ChatAttachment[] = paths.map((p) => {
        const name = p.split(/[/\\]/).pop() || p;
        return {
          type: "image",
          name,
          path: p,
          previewUrl: convertFileSrc(p),
        };
      });

      setAttachments((prev) => [...prev, ...newItems]);
      textareaRef.current?.focus();
    } catch (err) {
      console.error("Failed to select image:", err);
    }
  };

  const handleAttachFile = async () => {
    setShowAttachMenu(false);
    try {
      const selected = await openDialog({
        multiple: true,
        filters: [{ name: "All Files", extensions: ["*"] }],
      });

      if (!selected) return;
      const paths = Array.isArray(selected) ? selected : [selected];
      const newItems: ChatAttachment[] = paths.map((p) => {
        const name = p.split(/[/\\]/).pop() || p;
        return {
          type: "file",
          name,
          path: p,
        };
      });

      setAttachments((prev) => [...prev, ...newItems]);
      textareaRef.current?.focus();
    } catch (err) {
      console.error("Failed to select file:", err);
    }
  };

  const handleAttachFolder = async () => {
    setShowAttachMenu(false);
    try {
      const selected = await openDialog({
        directory: true,
        multiple: false,
      });

      if (!selected || typeof selected !== "string") return;
      const name = selected.split(/[/\\]/).pop() || selected;
      const folderItem: ChatAttachment = {
        type: "folder",
        name,
        path: selected,
      };

      setAttachments((prev) => [...prev, folderItem]);
      textareaRef.current?.focus();
    } catch (err) {
      console.error("Failed to select folder:", err);
    }
  };

  const removeAttachment = (index: number) => {
    setAttachments((prev) => prev.filter((_, i) => i !== index));
  };

  return (
    <div className="flex flex-col flex-1 min-h-0 bg-[#0a0a0c] border-t border-red-950/80">
      {/* Messages Viewport */}
      <div
        ref={scrollRef}
        className="flex-1 overflow-y-auto chat-scroll px-4 py-3 space-y-3"
      >
        {messages.map((msg) => (
          <div
            key={msg.id}
            className={`
              flex
              ${msg.role === "user" ? "justify-end" : "justify-start"}
            `}
          >
            <div
              className={`
                chat-selectable max-w-[85%] px-3.5 py-2.5 rounded-2xl text-[13px] leading-relaxed shadow-sm
                select-text cursor-text
                ${
                  msg.role === "user"
                    ? "bg-red-600 text-white rounded-br-sm shadow-md shadow-red-950/60 font-medium"
                    : "bg-[#141418] text-neutral-100 rounded-bl-sm border border-neutral-750 shadow-md shadow-black/80 font-normal"
                }
              `}
            >
              {/* Attached file/image cards in message history */}
              {msg.attachments && msg.attachments.length > 0 && (
                <div className="mb-2 flex flex-wrap gap-1.5 select-none">
                  {msg.attachments.map((att, idx) => (
                    <div
                      key={idx}
                      className="flex items-center gap-1.5 px-2 py-1 rounded-lg bg-black/40 border border-white/10 text-xs"
                      title={att.path}
                    >
                      {att.type === "image" ? (
                        <>
                          {att.previewUrl ? (
                            <img
                              src={att.previewUrl}
                              alt={att.name}
                              className="w-5 h-5 rounded object-cover"
                            />
                          ) : (
                            <span>🖼️</span>
                          )}
                          <span className="truncate max-w-[130px]">{att.name}</span>
                        </>
                      ) : att.type === "folder" ? (
                        <>
                          <span>📁</span>
                          <span className="truncate max-w-[130px]">{att.name}</span>
                        </>
                      ) : (
                        <>
                          <span>📄</span>
                          <span className="truncate max-w-[130px]">{att.name}</span>
                        </>
                      )}
                    </div>
                  ))}
                </div>
              )}

              <div className="whitespace-pre-wrap select-text">{msg.content}</div>

              {msg.isStreaming && (
                <span className="inline-block w-1.5 h-3.5 ml-1 bg-red-500 animate-pulse align-middle" />
              )}
            </div>
          </div>
        ))}
      </div>

      {/* Live Speech Recognition & Audio Waveform Banner */}
      {isListening && (
        <div className="mx-3 mb-2 px-3 py-2 rounded-xl bg-[#12080a] border border-red-600/50 shadow-lg animate-fade-in">
          <div className="flex items-center justify-between pb-1">
            <span className="flex items-center gap-1.5 text-red-400 font-bold text-[11px]">
              <span className="w-2 h-2 rounded-full bg-red-500 animate-ping" />
              Listening to voice...
            </span>
            {/* Real-time audio waveform equalizer */}
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
          <div className="text-white text-xs italic font-medium">
            {liveTranscript ? `“${liveTranscript}”` : "Speak now — words transcribe live..."}
          </div>
        </div>
      )}

      {/* Active Attachment Previews Strip */}
      {attachments.length > 0 && (
        <div className="flex items-center gap-2 px-3 py-1.5 bg-[#0f0f13] border-t border-red-950/60 overflow-x-auto chat-scroll">
          {attachments.map((att, idx) => (
            <div
              key={idx}
              className="flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-[#181820] border border-red-900/60 text-white text-[11px] shrink-0"
            >
              {att.type === "image" && att.previewUrl ? (
                <img
                  src={att.previewUrl}
                  alt={att.name}
                  className="w-5 h-5 rounded object-cover border border-white/20"
                />
              ) : att.type === "folder" ? (
                <span>📁</span>
              ) : (
                <span>📄</span>
              )}
              <span className="truncate max-w-[120px]" title={att.path}>
                {att.name}
              </span>
              <button
                type="button"
                onClick={() => removeAttachment(idx)}
                className="ml-1 text-neutral-400 hover:text-red-400 transition-colors"
                title="Remove attachment"
              >
                ✕
              </button>
            </div>
          ))}
        </div>
      )}

      {/* Input Form with Word Processing & Attachment Dropdown */}
      <form
        onSubmit={handleSubmit}
        className="relative flex items-end gap-2 px-3.5 py-2.5 border-t border-red-950/80 bg-[#060608]"
      >
        <VoiceControls
          isListening={isListening}
          isSpeaking={isSpeaking}
          onStartListening={onStartListening}
          onStopListening={onStopListening}
          onStopSpeaking={onStopSpeaking}
        />

        {/* "+" Attachment Button & Dropdown */}
        <div className="relative" ref={attachMenuRef}>
          <button
            type="button"
            onClick={() => setShowAttachMenu((prev) => !prev)}
            className={`
              p-2 rounded-xl text-xs font-bold transition-all flex items-center justify-center
              ${
                showAttachMenu
                  ? "bg-red-600 text-white shadow-md shadow-red-600/40"
                  : "bg-[#141418] hover:bg-[#202028] text-neutral-300 hover:text-white border border-neutral-700"
              }
            `}
            title="Attach file, image, or folder"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              viewBox="0 0 20 20"
              fill="currentColor"
              className="w-4 h-4"
            >
              <path d="M10.75 4.75a.75.75 0 00-1.5 0v4.5h-4.5a.75.75 0 000 1.5h4.5v4.5a.75.75 0 001.5 0v-4.5h4.5a.75.75 0 000-1.5h-4.5v-4.5z" />
            </svg>
          </button>

          {/* Dropdown Menu */}
          {showAttachMenu && (
            <div className="absolute bottom-12 left-0 w-44 py-1.5 rounded-xl bg-[#121217] border border-red-900/80 shadow-2xl shadow-black z-50 animate-fade-in text-xs select-none">
              <button
                type="button"
                onClick={handleAttachImage}
                className="w-full flex items-center gap-2.5 px-3 py-2 text-neutral-200 hover:text-white hover:bg-red-950/60 transition-colors text-left font-medium"
              >
                <span>🖼️</span>
                <span>Upload Image</span>
              </button>
              <button
                type="button"
                onClick={handleAttachFile}
                className="w-full flex items-center gap-2.5 px-3 py-2 text-neutral-200 hover:text-white hover:bg-red-950/60 transition-colors text-left font-medium"
              >
                <span>📄</span>
                <span>Upload File</span>
              </button>
              <button
                type="button"
                onClick={handleAttachFolder}
                className="w-full flex items-center gap-2.5 px-3 py-2 text-neutral-200 hover:text-white hover:bg-red-950/60 transition-colors text-left font-medium"
              >
                <span>📁</span>
                <span>Attach Folder</span>
              </button>
            </div>
          )}
        </div>

        {/* Multiline Textarea: NOT disabled so clicking always focuses caret */}
        <textarea
          ref={textareaRef}
          rows={1}
          value={input}
          onChange={handleInputChange}
          onKeyDown={handleKeyDown}
          placeholder={
            isListening
              ? "Transcribing voice..."
              : isSpeaking
              ? "ALIEN is speaking (tap to mute)..."
              : isStreaming
              ? "ALIEN is thinking... (you can prepare your next reply)"
              : "Ask ALIEN anything… (Shift+Enter for new line)"
          }
          className="
            flex-1 px-3 py-2 rounded-xl text-xs
            bg-[#141418] text-white
            placeholder-neutral-400
            border border-neutral-700
            focus:outline-none focus:ring-1 focus:ring-red-500 focus:border-red-500
            transition-all resize-none max-h-[120px] chat-scroll
            leading-normal cursor-text
          "
        />

        {/* Send Button */}
        <button
          type="submit"
          disabled={(!input.trim() && attachments.length === 0) || isStreaming}
          className="
            p-2 rounded-xl
            bg-red-600 hover:bg-red-700
            text-white text-xs font-medium
            disabled:opacity-30 disabled:cursor-not-allowed
            transition-all shadow-md shadow-red-600/30
          "
          aria-label="Send message"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 20 20"
            fill="currentColor"
            className="w-4 h-4"
          >
            <path d="M3.105 2.289a.75.75 0 00-.826.95l1.414 4.925A1.5 1.5 0 005.135 9.25h6.115a.75.75 0 010 1.5H5.135a1.5 1.5 0 00-1.442 1.086l-1.414 4.926a.75.75 0 00.826.95 28.896 28.896 0 0015.293-7.154.75.75 0 000-1.115A28.897 28.897 0 003.105 2.289z" />
          </svg>
        </button>
      </form>
    </div>
  );
}
