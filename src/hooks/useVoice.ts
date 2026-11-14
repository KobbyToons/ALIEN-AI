import { useState, useEffect, useCallback, useRef } from "react";
import type { VoiceConfig } from "../types";

export interface UseVoiceOptions {
  onTranscript?: (text: string) => void;
  onListeningChange?: (isListening: boolean) => void;
  onSpeakingChange?: (isSpeaking: boolean) => void;
  onLiveTranscript?: (text: string) => void;
}

export function useVoice({
  onTranscript,
  onListeningChange,
  onSpeakingChange,
  onLiveTranscript,
}: UseVoiceOptions = {}) {
  const [isListening, setIsListening] = useState(false);
  const [isSpeaking, setIsSpeaking] = useState(false);
  const [availableVoices, setAvailableVoices] = useState<SpeechSynthesisVoice[]>([]);

  // Real-time audio waveform metrics
  const [audioLevel, setAudioLevel] = useState<number>(0);
  const [waveformBars, setWaveformBars] = useState<number[]>([0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1]);
  const [liveTranscript, setLiveTranscript] = useState<string>("");

  const [voiceConfig, setVoiceConfig] = useState<VoiceConfig>(() => {
    const saved = localStorage.getItem("buddy_voice_config");
    if (saved) {
      try {
        return JSON.parse(saved);
      } catch (_) { }
    }
    return {
      autoSpeak: true,
      voiceName: "",
      rate: 1.05,
      pitch: 1.0,
    };
  });

  const recognitionRef = useRef<any>(null);
  const synthRef = useRef<SpeechSynthesis | null>(null);
  const audioContextRef = useRef<AudioContext | null>(null);
  const mediaStreamRef = useRef<MediaStream | null>(null);
  const animFrameRef = useRef<number | null>(null);
  const liveTranscriptRef = useRef<string>("");
  const finalTranscriptRef = useRef<string>("");
  const isListeningRef = useRef<boolean>(false);
  const hasDeliveredRef = useRef<boolean>(false);
  const restartTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const fallbackDeliveryTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Sync callbacks to refs so event listeners always call the latest props without re-binding
  const onTranscriptRef = useRef(onTranscript);
  const onListeningChangeRef = useRef(onListeningChange);
  const onSpeakingChangeRef = useRef(onSpeakingChange);
  const onLiveTranscriptRef = useRef(onLiveTranscript);

  useEffect(() => {
    onTranscriptRef.current = onTranscript;
    onListeningChangeRef.current = onListeningChange;
    onSpeakingChangeRef.current = onSpeakingChange;
    onLiveTranscriptRef.current = onLiveTranscript;
  }, [onTranscript, onListeningChange, onSpeakingChange, onLiveTranscript]);

  // Initialize SpeechSynthesis and load system voices
  useEffect(() => {
    if (typeof window !== "undefined" && "speechSynthesis" in window) {
      synthRef.current = window.speechSynthesis;

      const loadVoices = () => {
        const voices = synthRef.current?.getVoices() || [];
        setAvailableVoices(voices);
      };

      loadVoices();
      if (synthRef.current.onvoiceschanged !== undefined) {
        synthRef.current.onvoiceschanged = loadVoices;
      }
    }
  }, []);

  // Update voice configuration and persist
  const updateVoiceConfig = useCallback((cfg: VoiceConfig) => {
    setVoiceConfig(cfg);
    localStorage.setItem("buddy_voice_config", JSON.stringify(cfg));
  }, []);

  // Clean up audio analyzer resources
  const cleanupAudioAnalyser = useCallback(() => {
    if (animFrameRef.current) {
      cancelAnimationFrame(animFrameRef.current);
      animFrameRef.current = null;
    }
    if (mediaStreamRef.current) {
      mediaStreamRef.current.getTracks().forEach((track) => track.stop());
      mediaStreamRef.current = null;
    }
    if (audioContextRef.current) {
      try {
        audioContextRef.current.close();
      } catch (_) { }
      audioContextRef.current = null;
    }
    setAudioLevel(0);
    setWaveformBars([0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1]);
  }, []);

  // Stop active speech playback
  const stopSpeaking = useCallback(() => {
    if (synthRef.current) {
      synthRef.current.cancel();
    }
    setIsSpeaking(false);
    onSpeakingChangeRef.current?.(false);
  }, []);

  // Start audio waveform analyzer in parallel without blocking speech recognition
  const startAudioWaveform = useCallback(async () => {
    try {
      if (!navigator.mediaDevices?.getUserMedia) return;
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      if (!isListeningRef.current) {
        // Stop stream if listening ended before stream resolved
        stream.getTracks().forEach((t) => t.stop());
        return;
      }
      mediaStreamRef.current = stream;

      const AudioCtx = window.AudioContext || (window as any).webkitAudioContext;
      if (AudioCtx) {
        const audioCtx = new AudioCtx();
        audioContextRef.current = audioCtx;

        const source = audioCtx.createMediaStreamSource(stream);
        const analyser = audioCtx.createAnalyser();
        analyser.fftSize = 64;
        analyser.smoothingTimeConstant = 0.35;
        source.connect(analyser);

        const dataArray = new Uint8Array(analyser.frequencyBinCount);

        const renderAudioLoop = () => {
          if (!isListeningRef.current || !analyser) return;
          analyser.getByteFrequencyData(dataArray);

          let sum = 0;
          for (let i = 0; i < dataArray.length; i++) {
            sum += dataArray[i] ?? 0;
          }
          const avg = sum / dataArray.length;
          const norm = Math.min(1.0, Math.max(0, (avg - 6) / 85));
          setAudioLevel(norm);

          const step = Math.max(1, Math.floor(dataArray.length / 8));
          const bars: number[] = [];
          for (let i = 0; i < 8; i++) {
            const val = dataArray[i * step] || 0;
            bars.push(Math.max(0.12, Math.min(1.0, val / 255)));
          }
          setWaveformBars(bars);

          animFrameRef.current = requestAnimationFrame(renderAudioLoop);
        };

        renderAudioLoop();
      }
    } catch (err) {
      console.warn("Waveform audio capture notice:", err);
    }
  }, []);

  // ── Speech to Text (STT) ──────────────────────────────────────────

  const createAndStartRecognition = useCallback(() => {
    const SpeechRecognition =
      (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;

    if (!SpeechRecognition) {
      console.warn("SpeechRecognition API not available in this environment.");
      setLiveTranscript("⚠️ SpeechRecognition not available");
      return;
    }

    try {
      const recognition = new SpeechRecognition();
      recognition.continuous = true;
      recognition.interimResults = true;
      recognition.lang = "en-US";
      recognition.maxAlternatives = 1;

      recognition.onstart = () => {
        setIsListening(true);
        onListeningChangeRef.current?.(true);
      };

      recognition.onresult = (event: any) => {
        let interimTranscript = "";

        for (let i = event.resultIndex; i < event.results.length; ++i) {
          const res = event.results[i];
          if (!res || !res[0]) continue;
          if (res.isFinal) {
            finalTranscriptRef.current += res[0].transcript + " ";
          } else {
            interimTranscript += res[0].transcript;
          }
        }

        const combined = (finalTranscriptRef.current + interimTranscript).trim();
        if (combined) {
          setLiveTranscript(combined);
          liveTranscriptRef.current = combined;
          onLiveTranscriptRef.current?.(combined);
        }
      };

      recognition.onerror = (e: any) => {
        if (e.error === "no-speech" || e.error === "aborted") {
          return;
        }
        console.warn("Speech recognition notice:", e.error);
        if (e.error === "not-allowed" || e.error === "service-not-allowed") {
          const msg = "⚠️ Microphone access required";
          setLiveTranscript(msg);
          liveTranscriptRef.current = msg;
        }
      };

      recognition.onend = () => {
        if (isListeningRef.current) {
          // Restart with a fresh recognition instance so Chromium never throws InvalidStateError
          restartTimerRef.current = setTimeout(() => {
            if (isListeningRef.current) {
              createAndStartRecognition();
            }
          }, 80);
        } else {
          // Finalize transcript delivery once speech recognition has settled
          if (!hasDeliveredRef.current) {
            const text = (finalTranscriptRef.current + " " + liveTranscriptRef.current).trim();
            if (text && !text.startsWith("⚠️")) {
              hasDeliveredRef.current = true;
              onTranscriptRef.current?.(text);
            }
          }
        }
      };

      recognitionRef.current = recognition;
      recognition.start();
    } catch (recErr) {
      console.warn("SpeechRecognition start exception:", recErr);
    }
  }, []);

  const startListening = useCallback(() => {
    if (typeof window === "undefined") return;

    // Interrupt any active speech first
    stopSpeaking();

    // Stop any existing recognition instance cleanly
    if (recognitionRef.current) {
      try {
        recognitionRef.current.abort();
      } catch (_) { }
      recognitionRef.current = null;
    }

    isListeningRef.current = true;
    hasDeliveredRef.current = false;
    finalTranscriptRef.current = "";
    liveTranscriptRef.current = "";
    setLiveTranscript("");
    setIsListening(true);
    onListeningChangeRef.current?.(true);

    // Initialize fresh recognition instance
    createAndStartRecognition();

    // Launch waveform audio analyser in parallel
    startAudioWaveform();
  }, [createAndStartRecognition, startAudioWaveform, stopSpeaking]);

  const stopListening = useCallback(() => {
    isListeningRef.current = false;

    // Check if we already have speech captured
    const textToDeliver = liveTranscriptRef.current.trim();
    if (textToDeliver && !hasDeliveredRef.current && !textToDeliver.startsWith("⚠️")) {
      hasDeliveredRef.current = true;
      onTranscriptRef.current?.(textToDeliver);
    }

    if (recognitionRef.current) {
      try {
        recognitionRef.current.stop();
      } catch (_) { }
      setTimeout(() => {
        recognitionRef.current = null;
      }, 300);
    }

    cleanupAudioAnalyser();
    setIsListening(false);
    onListeningChangeRef.current?.(false);

    // Fallback transcript delivery in case onend takes more than 280ms
    fallbackDeliveryTimerRef.current = setTimeout(() => {
      if (!hasDeliveredRef.current) {
        const text = (finalTranscriptRef.current + " " + liveTranscriptRef.current).trim();
        if (text && !text.startsWith("⚠️")) {
          hasDeliveredRef.current = true;
          onTranscriptRef.current?.(text);
        }
      }

      // Clear transcript display after brief delay so user sees what was submitted
      setTimeout(() => {
        setLiveTranscript("");
        liveTranscriptRef.current = "";
        finalTranscriptRef.current = "";
        hasDeliveredRef.current = false;
        recognitionRef.current = null;
      }, 350);
    }, 280);
  }, [cleanupAudioAnalyser]);

  const speak = useCallback(
    (text: string) => {
      if (!voiceConfig.autoSpeak || !synthRef.current || !text.trim()) return;

      synthRef.current.cancel();

      // Clean markdown symbols for natural TTS speech
      const cleanText = text
        .replace(/```[\s\S]*?```/g, "Code snippet omitted.")
        .replace(/`([^`]+)`/g, "$1")
        .replace(/\*\*([^*]+)\*\*/g, "$1")
        .replace(/\*([^*]+)\*/g, "$1")
        .replace(/#+\s/g, "")
        .replace(/\[([^\]]+)\]\([^)]+\)/g, "$1")
        .trim();

      if (!cleanText) return;

      const utterance = new SpeechSynthesisUtterance(cleanText);
      utterance.rate = voiceConfig.rate;
      utterance.pitch = voiceConfig.pitch;

      if (voiceConfig.voiceName) {
        const found = availableVoices.find((v) => v.name === voiceConfig.voiceName);
        if (found) utterance.voice = found;
      }

      utterance.onstart = () => {
        setIsSpeaking(true);
        onSpeakingChangeRef.current?.(true);
      };

      utterance.onend = () => {
        setIsSpeaking(false);
        onSpeakingChangeRef.current?.(false);
      };

      utterance.onerror = () => {
        setIsSpeaking(false);
        onSpeakingChangeRef.current?.(false);
      };

      synthRef.current.speak(utterance);
    },
    [voiceConfig, availableVoices]
  );

  return {
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
  };
}
