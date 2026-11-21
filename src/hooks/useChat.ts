import { useState, useCallback, useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { modelStreamChat, runAgentTask, stopGeneration as stopGenerationApi, fsRead, fsList } from "../lib/tauri";
import type { AgentStepEvent, BuddyState, ChatMessage, ChatAttachment, ModelProvider } from "../types";

export interface ActiveModelConfig {
  provider: ModelProvider;
  model: string;          // Fast model for snappy conversational dialog (e.g. gemini-2.5-flash or local-model)
  heavyModel?: string;    // Heavy reasoning model for complex autonomous tasks (e.g. gemini-2.5-pro or gpt-4o)
  customEndpoint?: string;
  mode: "auto" | "chat" | "agent";
}

/**
 * Intelligent classifier: detects whether a user prompt requires multi-step autonomous PC actions
 * and tool calls, or if it is conversational/informational chat that should stream instantly.
 */
function isAutonomousTask(text: string): boolean {
  const t = text.toLowerCase().trim();

  // Greetings, chit-chat, and pure inquiries are fast conversational chat
  if (/^(hi|hello|hey|yo|greetings|sup|what'?s up|good (morning|afternoon|evening)|how are you|who are you)\b/i.test(t)) {
    return false;
  }
  if (/^(what|why|who|when|where|how|can you explain|explain|tell me about|what is|what are)\b/i.test(t) && !/\b(create|delete|run|save|execute|automate|build|install)\b/i.test(t)) {
    return false;
  }

  // Action commands that require autonomous PC tools & execution
  const actionTriggers = [
    /\b(look at|see|capture|take a look at|check|inspect|view)\s+(my\s+)?(screen|display|monitor|window)\b/i,
    /\b(what('?s| is)\s+(on|showing on)\s+(my\s+)?(screen|display|monitor))\b/i,
    /\b(screenshot|screen capture)\b/i,
    /\b(create|make|write|generate)\s+(a\s+)?(file|script|note|folder|document|app|program)\b/i,
    /\b(delete|remove|erase|clear)\s+(the\s+)?(file|folder|cache)\b/i,
    /\b(run|execute|launch|start|open)\s+(the\s+)?(terminal|powershell|cmd|app|script|process|browser)\b/i,
    /\b(search|find|list|inspect)\s+(my\s+)?(files|folders|directory|system|pc|desktop)\b/i,
    /\b(save|write)\s+to\b/i,
    /\b(organize|clean up|sort)\s+(my\s+)?(desktop|downloads|folder)\b/i,
    /\b(automate|autonomous|schedule|task|mcp)\b/i,
    /\b(powershell|bash|cmd|terminal|cli)\b/i,
    /\b(install|download|git clone|npm|pip|cargo)\b/i,
  ];

  return actionTriggers.some((rgx) => rgx.test(t));
}

export function useChat(onStateChange?: (state: BuddyState) => void) {
  const [messages, setMessages] = useState<ChatMessage[]>([
    {
      id: "welcome",
      role: "assistant",
      content: "Greetings! I am ALIEN. Ask me anything, speak with your voice, or summon me to execute tasks on your PC!",
      timestamp: Date.now(),
    },
  ]);

  const [isStreaming, setIsStreaming] = useState(false);
  const [agentSteps, setAgentSteps] = useState<AgentStepEvent[]>([]);
  const activeReqIdRef = useRef<string | null>(null);

  // Model settings: defaults to Gemini 3.8 Flash
  const [modelConfig, setModelConfig] = useState<ActiveModelConfig>(() => {
    const saved = localStorage.getItem("alien_model_config") || localStorage.getItem("buddy_model_config");
    if (saved) {
      try {
        const parsed = JSON.parse(saved);
        const prov = parsed.provider || "gemini";
        return {
          provider: prov,
          model: prov === "gemini" ? "gemini-3.8-flash" : (parsed.model || "gemini-3.8-flash"),
          heavyModel: prov === "gemini" ? "gemini-3.8-flash" : (parsed.heavyModel || "gemini-3.8-flash"),
          customEndpoint: parsed.customEndpoint,
          mode: parsed.mode || "chat",
        };
      } catch (_) { }
    }
    return {
      provider: "gemini",
      model: "gemini-3.8-flash",
      heavyModel: "gemini-3.8-flash",
      mode: "chat",
    };
  });

  const updateModelConfig = (cfg: ActiveModelConfig) => {
    setModelConfig(cfg);
    localStorage.setItem("alien_model_config", JSON.stringify(cfg));
  };

  // Listen to streaming chunks & agent events
  useEffect(() => {
    const unlistenChunk = listen<{ requestId: string; chunk: string }>(
      "buddy://chat-chunk",
      (event) => {
        if (event.payload.requestId === activeReqIdRef.current) {
          onStateChange?.("speaking");
          setMessages((prev) => {
            const last = prev[prev.length - 1];
            if (last && last.role === "assistant" && last.isStreaming) {
              return [
                ...prev.slice(0, -1),
                { ...last, content: last.content + event.payload.chunk },
              ];
            }
            return prev;
          });
        }
      }
    );

    const unlistenEnd = listen<{ requestId: string; fullText: string }>(
      "buddy://chat-end",
      (event) => {
        if (event.payload.requestId === activeReqIdRef.current) {
          setIsStreaming(false);
          activeReqIdRef.current = null;
          onStateChange?.("idle");
          setMessages((prev) => {
            const last = prev[prev.length - 1];
            if (last && last.role === "assistant") {
              return [
                ...prev.slice(0, -1),
                { ...last, content: event.payload.fullText, isStreaming: false },
              ];
            }
            return prev;
          });
        }
      }
    );

    const unlistenError = listen<{ requestId: string; error: string }>(
      "buddy://chat-error",
      (event) => {
        if (event.payload.requestId === activeReqIdRef.current) {
          setIsStreaming(false);
          activeReqIdRef.current = null;
          onStateChange?.("idle");
          setMessages((prev) => {
            const last = prev[prev.length - 1];
            if (last && last.role === "assistant" && last.isStreaming) {
              return [
                ...prev.slice(0, -1),
                { ...last, content: `⚠️ ${event.payload.error}`, isStreaming: false },
              ];
            }
            return [
              ...prev,
              {
                id: crypto.randomUUID(),
                role: "assistant",
                content: `⚠️ ${event.payload.error}`,
                timestamp: Date.now(),
              },
            ];
          });
        }
      }
    );

    const unlistenAgent = listen<AgentStepEvent>(
      "buddy://agent-step",
      (event) => {
        setAgentSteps((prev) => [...prev, event.payload]);
        if (event.payload.status === "executing") {
          onStateChange?.("working");
        } else if (event.payload.status === "thinking") {
          onStateChange?.("thinking");
        } else if (event.payload.status === "done") {
          onStateChange?.("idle");
          setTimeout(() => setAgentSteps([]), 5000);
        }
      }
    );

    return () => {
      unlistenChunk.then((f) => f());
      unlistenEnd.then((f) => f());
      unlistenError.then((f) => f());
      unlistenAgent.then((f) => f());
    };
  }, [onStateChange]);

  const sendMessage = useCallback(
    async (text: string, attachments?: ChatAttachment[]) => {
      if ((!text.trim() && (!attachments || attachments.length === 0)) || isStreaming) return;

      const userMsg: ChatMessage = {
        id: crypto.randomUUID(),
        role: "user",
        content: text,
        timestamp: Date.now(),
        attachments: attachments && attachments.length > 0 ? attachments : undefined,
      };

      const nextMessages = [...messages, userMsg];
      setMessages(nextMessages);

      const requestId = crypto.randomUUID();
      activeReqIdRef.current = requestId;
      setIsStreaming(true);

      // Construct enriched prompt incorporating file contents, images, and folder trees
      let promptWithAttachments = text;
      if (attachments && attachments.length > 0) {
        let attachmentContext = "";
        for (const att of attachments) {
          if (att.type === "image") {
            attachmentContext += `\n[User Attached Image: ${att.path} (${att.name})]\n`;
          } else if (att.type === "folder") {
            try {
              const list = await fsList(att.path);
              const preview = list
                .slice(0, 35)
                .map((f) => `- ${f.name} (${f.isDir ? "directory" : `${f.size} bytes`})`)
                .join("\n");
              attachmentContext += `\n[User Attached Directory: ${att.path}]\nDirectory items (${list.length} total):\n${preview}\n`;
            } catch (_) {
              attachmentContext += `\n[User Attached Directory: ${att.path}]\n`;
            }
          } else {
            try {
              const content = await fsRead(att.path);
              const snippet =
                content.length > 8000
                  ? `${content.slice(0, 8000)}\n... [truncated ${content.length - 8000} bytes]`
                  : content;
              attachmentContext += `\n[User Attached File: ${att.path} (${att.name})]\n\`\`\`\n${snippet}\n\`\`\`\n`;
            } catch (_) {
              attachmentContext += `\n[User Attached File: ${att.path} (${att.name})]\n`;
            }
          }
        }
        promptWithAttachments = `${attachmentContext}\nUser Request: ${text.trim() || "Please inspect and act upon the attached files."}`;
      }

      // Determine execution mode: Auto-switch vs explicit
      const shouldRunAgent =
        modelConfig.mode === "agent" ||
        (modelConfig.mode === "auto" && isAutonomousTask(text));

      if (shouldRunAgent) {
        // High-reasoning autonomous agent with tool execution
        onStateChange?.("working");
        setAgentSteps([]);

        const agentModel = modelConfig.heavyModel || modelConfig.model;

        try {
          const finalResult = await runAgentTask(
            requestId,
            modelConfig.provider,
            agentModel,
            promptWithAttachments,
            modelConfig.customEndpoint
          );

          setMessages((prev) => [
            ...prev,
            {
              id: requestId,
              role: "assistant",
              content: finalResult,
              timestamp: Date.now(),
            },
          ]);
        } catch (err: unknown) {
          const errMsg = typeof err === "string" ? err : JSON.stringify(err);
          setMessages((prev) => [
            ...prev,
            {
              id: crypto.randomUUID(),
              role: "assistant",
              content: `⚠️ Agent error: ${errMsg}`,
              timestamp: Date.now(),
            },
          ]);
        } finally {
          setIsStreaming(false);
          activeReqIdRef.current = null;
          onStateChange?.("idle");
        }
      } else {
        // Fast streaming conversational turn (Human-speed response)
        onStateChange?.("thinking");

        setMessages((prev) => [
          ...prev,
          {
            id: requestId,
            role: "assistant",
            content: "",
            timestamp: Date.now(),
            isStreaming: true,
          },
        ]);

        const formatted = nextMessages
          .filter((m) => m.id !== "welcome")
          .map((m) =>
            m.id === userMsg.id
              ? { role: m.role, content: promptWithAttachments }
              : { role: m.role, content: m.content }
          );

        try {
          const fullResponse = await modelStreamChat(
            requestId,
            modelConfig.provider,
            modelConfig.model,
            formatted,
            modelConfig.customEndpoint
          );
          // Ensure thinking state stops immediately once response is complete
          setIsStreaming(false);
          activeReqIdRef.current = null;
          onStateChange?.("idle");
          if (fullResponse) {
            setMessages((prev) => {
              const last = prev[prev.length - 1];
              if (last && last.role === "assistant") {
                return [
                  ...prev.slice(0, -1),
                  { ...last, content: fullResponse, isStreaming: false },
                ];
              }
              return prev;
            });
          }
        } catch (err: unknown) {
          setIsStreaming(false);
          activeReqIdRef.current = null;
          onStateChange?.("idle");
          const errMsg = typeof err === "string" ? err : JSON.stringify(err);
          setMessages((prev) => {
            const last = prev[prev.length - 1];
            // If unlistenError already displayed this error, keep it
            if (last && last.role === "assistant" && last.content.includes(errMsg)) {
              return prev;
            }
            return [
              ...prev.filter((m) => m.id !== requestId),
              {
                id: crypto.randomUUID(),
                role: "assistant",
                content: `⚠️ ${errMsg}`,
                timestamp: Date.now(),
              },
            ];
          });
        } finally {
          setIsStreaming(false);
          activeReqIdRef.current = null;
          onStateChange?.("idle");
        }
      }
    },
    [isStreaming, messages, modelConfig, onStateChange]
  );

  const stopGeneration = useCallback(async () => {
    const reqId = activeReqIdRef.current;
    setIsStreaming(false);
    activeReqIdRef.current = null;
    onStateChange?.("idle");

    try {
      await stopGenerationApi(reqId || undefined);
    } catch (_) { }

    setMessages((prev) => {
      const last = prev[prev.length - 1];
      if (last && last.role === "assistant" && last.isStreaming) {
        return [
          ...prev.slice(0, -1),
          {
            ...last,
            content: last.content
              ? `${last.content}\n\n*[Generation stopped by user]*`
              : "⚠️ Generation stopped by user.",
            isStreaming: false,
          },
        ];
      }
      return prev;
    });

    setAgentSteps([]);
  }, [onStateChange]);

  const clearMessages = useCallback(() => {
    setMessages([]);
    setAgentSteps([]);
  }, []);

  return {
    messages,
    agentSteps,
    isStreaming,
    modelConfig,
    updateModelConfig,
    sendMessage,
    stopGeneration,
    clearMessages,
  };
}
