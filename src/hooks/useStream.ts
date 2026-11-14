import { useState, useCallback, useRef } from "react";

export interface UseStreamOptions {
  onChunk?: (chunk: string) => void;
  onFinish?: (fullText: string) => void;
  speedMs?: number;
}

export function useStream({ onChunk, onFinish, speedMs = 25 }: UseStreamOptions = {}) {
  const [isStreaming, setIsStreaming] = useState(false);
  const cancelRef = useRef(false);

  /**
   * Simulates a streaming text response (chunk by chunk).
   * In Phase 4, this is replaced/integrated with real SSE / chunk readers from LLM providers.
   */
  const streamText = useCallback(
    async (fullText: string) => {
      setIsStreaming(true);
      cancelRef.current = false;
      let accumulated = "";

      const tokens = fullText.split(/(\s+)/); // split by whitespace retaining tokens

      for (const token of tokens) {
        if (cancelRef.current) break;
        accumulated += token;
        onChunk?.(accumulated);
        await new Promise((r) => setTimeout(r, speedMs));
      }

      setIsStreaming(false);
      onFinish?.(accumulated);
      return accumulated;
    },
    [onChunk, onFinish, speedMs]
  );

  const stopStream = useCallback(() => {
    cancelRef.current = true;
    setIsStreaming(false);
  }, []);

  return {
    isStreaming,
    streamText,
    stopStream,
  };
}

