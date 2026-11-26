import { useState, useEffect } from "react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
import {
  addSafetyBlocklistKeyword,
  connectGoogleAccount,
  createVaultTestNote,
  deleteApiKey,
  getAgentPresence,
  getAuditLog,
  getComputerUseMetrics,
  getGoogleAccountStatus,
  getKillSwitch,
  getLearningConfig,
  getSafetyBlocklist,
  getSystemIdentity,
  getThinkingProfile,
  getVaultInfo,
  getWatchModeStatus,
  hasApiKey,
  listSkills,
  openVaultInExplorer,
  reloadSkills,
  removeSafetyBlocklistKeyword,
  resetSystemIdentity,
  restoreSystemCursors,
  saveApiKey,
  saveSystemIdentity,
  setVaultPath,
  startWatchMode,
  stopWatchMode,
  toggleAgentPresence,
  toggleKillSwitch,
  toggleSafetyBlocklist,
  updateLearningConfig,
  filmcraftDetectCli,
  filmcraftConnect,
  filmcraftDisconnect,
  filmcraftGetStatus,
  filmcraftSetMode,
  filmcraftGetHistory,
  filmcraftClearHistory,
} from "../lib/tauri";
import type { ActiveModelConfig } from "../hooks/useChat";
import type {
  AuditEntry,
  ComputerUseMetrics,
  FilmCraftEditingMode,
  FilmCraftEditingStep,
  FilmCraftStatus,
  GoogleAccountStatus,
  LearningConfig,
  LoadedSkill,
  ModelProvider,
  ThinkingProfile,
  VaultInfo,
  VoiceConfig,
} from "../types";

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  onClearChat: () => void;
  modelConfig: ActiveModelConfig;
  onUpdateModelConfig: (cfg: ActiveModelConfig) => void;
  voiceConfig: VoiceConfig;
  availableVoices: SpeechSynthesisVoice[];
  onUpdateVoiceConfig: (cfg: VoiceConfig) => void;
}

const PROVIDERS: { id: ModelProvider; name: string; defaultModel: string; models: string[] }[] = [
  {
    id: "gemini",
    name: "Google Gemini (3.8 Flash)",
    defaultModel: "gemini-3.8-flash",
    models: ["gemini-3.8-flash"],
  },
  {
    id: "openai",
    name: "OpenAI",
    defaultModel: "gpt-4o",
    models: ["gpt-4o", "gpt-4o-mini"],
  },
  {
    id: "anthropic",
    name: "Anthropic Claude",
    defaultModel: "claude-3-5-sonnet-20241022",
    models: ["claude-3-5-sonnet-20241022", "claude-3-5-haiku-20241022"],
  },
  {
    id: "deepseek",
    name: "DeepSeek",
    defaultModel: "deepseek-chat",
    models: ["deepseek-chat", "deepseek-reasoner"],
  },
  {
    id: "lmstudio",
    name: "LM Studio (Local)",
    defaultModel: "local-model",
    models: ["local-model"],
  },
];

export default function SettingsModal({
  isOpen,
  onClose,
  onClearChat,
  modelConfig,
  onUpdateModelConfig,
  voiceConfig,
  availableVoices,
  onUpdateVoiceConfig,
}: SettingsModalProps) {
  const [killSwitch, setKillSwitch] = useState(false);
  const [logs, setLogs] = useState<AuditEntry[]>([]);
  const [vaultInfo, setVaultInfo] = useState<VaultInfo | null>(null);
  const [learningConfig, setLearningConfig] = useState<LearningConfig>({
    passive_learning: true,
    scenario_questions: true,
  });
  const [thinkingProfile, setThinkingProfile] = useState<ThinkingProfile | null>(null);
  const [skills, setSkills] = useState<LoadedSkill[]>([]);
  const [autostartEnabled, setAutostartEnabled] = useState(false);

  const [activeTab, setActiveTab] = useState<
    "ai" | "computer_use" | "video_editor" | "identity" | "skills" | "memory" | "learning" | "voice" | "general" | "audit"
  >("ai");

  // FilmCraft Video Editor state
  const [fcStatus, setFcStatus] = useState<FilmCraftStatus | null>(null);
  const [fcCliPath, setFcCliPath] = useState<string | null>(null);
  const [fcHistory, setFcHistory] = useState<FilmCraftEditingStep[]>([]);
  const [fcLaunchMode, setFcLaunchMode] = useState<"headless" | "demo" | "project" | "bridge">("headless");
  const [fcProjectPath, setFcProjectPath] = useState("");
  const [fcBridgeAddr, setFcBridgeAddr] = useState("127.0.0.1:9886");
  const [fcConnecting, setFcConnecting] = useState(false);
  const [fcMsg, setFcMsg] = useState<{ text: string; isError: boolean } | null>(null);

  // Vision & Computer Use state (Phase H)
  const [blocklist, setBlocklist] = useState<string[]>([]);
  const [blocklistInput, setBlocklistInput] = useState("");
  const [blocklistEnabled, setBlocklistEnabled] = useState(true);
  const [metrics, setMetrics] = useState<ComputerUseMetrics | null>(null);
  const [cursorRestoreMsg, setCursorRestoreMsg] = useState<string | null>(null);
  const [isOverlayActive, setIsOverlayActive] = useState(false);
  const [isWatchActive, setIsWatchActive] = useState(false);

  // AI Identity & Foundation Manual state
  const [systemIdentity, setSystemIdentity] = useState("");
  const [isSavingIdentity, setIsSavingIdentity] = useState(false);
  const [identityStatus, setIdentityStatus] = useState<string | null>(null);

  // API Key & Google Account state
  const [keysConfigured, setKeysConfigured] = useState<Record<string, boolean>>({});
  const [googleStatus, setGoogleStatus] = useState<GoogleAccountStatus | null>(null);
  const [isConnectingGoogle, setIsConnectingGoogle] = useState(false);
  const [keyInput, setKeyInput] = useState("");
  const [saveStatus, setSaveStatus] = useState<string | null>(null);

  // LM Studio local server state
  const [lmStudioModels, setLmStudioModels] = useState<string[]>([]);
  const [isFetchingLmStudio, setIsFetchingLmStudio] = useState(false);
  const [lmStudioStatus, setLmStudioStatus] = useState<string | null>(null);

  // Obsidian Vault connection state
  const [vaultPathInput, setVaultPathInput] = useState("");
  const [vaultStatusMsg, setVaultStatusMsg] = useState<{ text: string; isError: boolean } | null>(null);
  const [isConnectingVault, setIsConnectingVault] = useState(false);
  const [isTestingVault, setIsTestingVault] = useState(false);

  useEffect(() => {
    if (isOpen) {
      getKillSwitch().then(setKillSwitch).catch(() => {});
      getAuditLog(30).then(setLogs).catch(() => {});
      getVaultInfo().then((v) => {
        setVaultInfo(v);
        if (v?.path) {
          setVaultPathInput(v.path);
        } else {
          setVaultPathInput("C:\\Users\\Kobby\\Documents\\Future Heroes\\ALIEN's Mind");
        }
      }).catch(() => {});
      getLearningConfig().then(setLearningConfig).catch(() => {});
      getThinkingProfile().then(setThinkingProfile).catch(() => {});
      getSystemIdentity().then(setSystemIdentity).catch(() => {});
      listSkills().then(setSkills).catch(() => {});
      isEnabled().then(setAutostartEnabled).catch(() => {});

      // Check Google Account status
      getGoogleAccountStatus().then((st) => {
        setGoogleStatus(st);
        if (st.is_connected) {
          setKeysConfigured((prev) => ({ ...prev, gemini: true }));
        }
      }).catch(() => {});

      setKeysConfigured((prev) => ({ ...prev, lmstudio: true, ollama: true }));

      getSafetyBlocklist().then(setBlocklist).catch(() => {});
      getComputerUseMetrics().then(setMetrics).catch(() => {});
      getAgentPresence().then(setIsOverlayActive).catch(() => {});
      getWatchModeStatus().then(setIsWatchActive).catch(() => {});

      filmcraftDetectCli().then(setFcCliPath).catch(() => {});
      filmcraftGetStatus().then(setFcStatus).catch(() => {});
      filmcraftGetHistory().then(setFcHistory).catch(() => {});

      PROVIDERS.forEach(async (p) => {
        if (p.id !== "ollama" && p.id !== "lmstudio") {
          const has = await hasApiKey(p.id);
          setKeysConfigured((prev) => ({ ...prev, [p.id]: has }));
        }
      });
    }
  }, [isOpen]);

  const handleFetchLmStudioModels = async () => {
    const rawEndpoint = modelConfig.customEndpoint || "http://localhost:1234/v1";
    const endpoint = rawEndpoint.trim().replace(/\/$/, "");
    setIsFetchingLmStudio(true);
    setLmStudioStatus(null);
    try {
      const res = await fetch(`${endpoint}/models`);
      if (!res.ok) throw new Error(`HTTP ${res.status}: ${res.statusText}`);
      const data = await res.json();
      const models = (data.data || []).map((m: any) => m.id);
      if (models.length > 0) {
        setLmStudioModels(models);
        onUpdateModelConfig({ ...modelConfig, model: models[0] });
        setLmStudioStatus(`✓ Connected! Found ${models.length} model(s): ${models.join(", ")}`);
      } else {
        setLmStudioStatus("Connected to LM Studio, but no models are loaded yet. Please load a model in LM Studio.");
      }
    } catch (e: any) {
      setLmStudioStatus(`Failed to connect to LM Studio at ${endpoint}: ${e.message || e}. Ensure the Local Server is running on port 1234.`);
    } finally {
      setIsFetchingLmStudio(false);
    }
  };

  const handleToggleAutostart = async () => {
    try {
      if (autostartEnabled) {
        await disable();
        setAutostartEnabled(false);
      } else {
        await enable();
        setAutostartEnabled(true);
      }
    } catch (e) {
      console.error(e);
    }
  };

  const handleConnectVault = async (targetPath?: string) => {
    const p = (targetPath || vaultPathInput).trim();
    if (!p) {
      setVaultStatusMsg({ text: "Please provide a valid folder path.", isError: true });
      return;
    }
    setIsConnectingVault(true);
    setVaultStatusMsg(null);
    try {
      const info = await setVaultPath(p);
      setVaultInfo(info);
      setVaultPathInput(info.path);
      setVaultStatusMsg({
        text: `Connected to vault! ${info.noteCount} notes indexed.`,
        isError: false,
      });
    } catch (e: any) {
      setVaultStatusMsg({
        text: `Connection failed: ${e?.message || e}`,
        isError: true,
      });
    } finally {
      setIsConnectingVault(false);
    }
  };

  const handleSelectVaultFolder = async () => {
    setVaultStatusMsg(null);
    try {
      const selected = await openDialog({
        directory: true,
        multiple: false,
        title: "Select your Obsidian Vault folder",
      });

      if (selected && typeof selected === "string") {
        setVaultPathInput(selected);
        await handleConnectVault(selected);
      }
    } catch (e: any) {
      console.error("Failed to open dialog:", e);
      setVaultStatusMsg({
        text: `Folder picker notice: ${e?.message || e}. You can paste or type the path directly.`,
        isError: true,
      });
    }
  };

  const handleCreateTestNote = async () => {
    setIsTestingVault(true);
    setVaultStatusMsg(null);
    try {
      const res = await createVaultTestNote();
      const info = await getVaultInfo();
      setVaultInfo(info);
      setVaultStatusMsg({
        text: `Verified! Created: ${res}`,
        isError: false,
      });
    } catch (e: any) {
      setVaultStatusMsg({
        text: `Failed to write test note: ${e?.message || e}`,
        isError: true,
      });
    } finally {
      setIsTestingVault(false);
    }
  };

  const handleOpenExplorer = async () => {
    try {
      await openVaultInExplorer();
    } catch (e: any) {
      setVaultStatusMsg({
        text: `Explorer error: ${e?.message || e}`,
        isError: true,
      });
    }
  };

  const handleToggleKillSwitch = async () => {
    try {
      const next = !killSwitch;
      const res = await toggleKillSwitch(next);
      setKillSwitch(res);
    } catch (err) {
      console.error(err);
    }
  };

  const handleConnectGoogle = async () => {
    setIsConnectingGoogle(true);
    setSaveStatus("Connecting to Google Account...");
    try {
      const res = await connectGoogleAccount();
      setGoogleStatus(res);
      setKeysConfigured((prev) => ({ ...prev, gemini: true }));
      setSaveStatus("Connected to Google Account successfully! 🚀");
      setTimeout(() => setSaveStatus(null), 3000);
    } catch (e) {
      setSaveStatus(`Failed to connect Google Account: ${e}`);
    } finally {
      setIsConnectingGoogle(false);
    }
  };

  const handleSaveKey = async (provider: ModelProvider) => {
    if (!keyInput.trim()) return;
    try {
      await saveApiKey(provider, keyInput.trim());
      setKeysConfigured((prev) => ({ ...prev, [provider]: true }));
      setKeyInput("");
      setSaveStatus("Saved to Windows Credential Manager! 🔒");
      setTimeout(() => setSaveStatus(null), 3000);
    } catch (e) {
      setSaveStatus(`Failed to save: ${e}`);
    }
  };

  const handleDeleteKey = async (provider: ModelProvider) => {
    try {
      await deleteApiKey(provider);
      setKeysConfigured((prev) => ({ ...prev, [provider]: false }));
      if (provider === "gemini") {
        setGoogleStatus(null);
      }
      setSaveStatus("Credentials removed from Credential Manager.");
      setTimeout(() => setSaveStatus(null), 3000);
    } catch (e) {
      setSaveStatus(`Failed: ${e}`);
    }
  };

  const handleUpdateLearning = async (patch: Partial<LearningConfig>) => {
    const next = { ...learningConfig, ...patch };
    setLearningConfig(next);
    await updateLearningConfig(next);
  };

  const handleReloadSkills = async () => {
    try {
      const reloaded = await reloadSkills();
      setSkills(reloaded);
    } catch (e) {
      console.error(e);
    }
  };

  const handleSaveIdentity = async () => {
    setIsSavingIdentity(true);
    setIdentityStatus("Saving Identity Manual...");
    try {
      await saveSystemIdentity(systemIdentity);
      setIdentityStatus("Identity & Directives Manual saved! Synced to Obsidian & Brain. 🧠");
      setTimeout(() => setIdentityStatus(null), 3500);
    } catch (e) {
      setIdentityStatus(`Failed to save manual: ${e}`);
    } finally {
      setIsSavingIdentity(false);
    }
  };

  const handleResetIdentity = async () => {
    try {
      const def = await resetSystemIdentity();
      setSystemIdentity(def);
      setIdentityStatus("Restored default ALIEN Foundation Manual & Directives. ✨");
      setTimeout(() => setIdentityStatus(null), 3000);
    } catch (e) {
      setIdentityStatus(`Error resetting manual: ${e}`);
    }
  };

  if (!isOpen) return null;

  const currentProviderInfo = PROVIDERS.find((p) => p.id === modelConfig.provider)!;

  return (
    <div className="absolute inset-0 z-50 flex flex-col bg-[#08080b] rounded-3xl overflow-hidden p-4 animate-fade-in text-neutral-100 border border-red-900/60 shadow-2xl shadow-black">
      {/* Header */}
      <div className="flex items-center justify-between pb-2 border-b border-red-950/80">
        <div className="flex items-center gap-2">
          <span className="text-red-500 font-extrabold text-sm tracking-wider">ALIEN</span>
          <span className="text-neutral-300 text-xs">Settings & Brain</span>
        </div>
        <button
          onClick={onClose}
          className="p-1 rounded-lg hover:bg-neutral-800 transition-colors text-xs text-neutral-400 hover:text-red-400 font-bold"
        >
          ✕
        </button>
      </div>

      {/* Tabs */}
      <div className="flex gap-1.5 my-2.5 text-[11px] overflow-x-auto pb-1 chat-scroll">
        {(
          [
            ["ai", "Brain"],
            ["computer_use", "Vision & PC Control"],
            ["video_editor", "🎬 Video Editor"],
            ["identity", "Identity Manual"],
            ["skills", `Skills (${skills.length})`],
            ["memory", "Memory"],
            ["learning", "Learning"],
            ["voice", "Voice"],
            ["general", "Controls"],
            ["audit", `Audit (${logs.length})`],
          ] as const
        ).map(([tab, label]) => (
          <button
            key={tab}
            onClick={() => {
              setActiveTab(tab);
              if (tab === "audit") {
                getAuditLog(30).then(setLogs).catch(() => {});
              }
            }}
            className={`px-2.5 py-1 rounded-xl font-medium transition-colors whitespace-nowrap ${
              activeTab === tab
                ? "bg-red-600 text-white shadow-sm shadow-red-600/30 font-semibold"
                : "bg-neutral-900 text-neutral-400 hover:text-neutral-200 border border-neutral-800"
            }`}
          >
            {label}
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto chat-scroll text-xs space-y-3 pr-1">
        {activeTab === "ai" && (
          <div className="space-y-3">
            {/* Operating Mode Toggle */}
            <div className="space-y-1.5">
              <div className="flex items-center justify-between">
                <label className="text-[10px] uppercase font-bold text-neutral-400">
                  Operating Mode
                </label>
                <span className="text-[10px] text-red-400 font-medium">
                  {modelConfig.mode === "agent" ? "🤖 Autonomous Agent" : "💬 Direct Fast Chat"}
                </span>
              </div>
              <div className="grid grid-cols-2 gap-2">
                <button
                  onClick={() => onUpdateModelConfig({ ...modelConfig, mode: "chat" })}
                  className={`py-2 px-3 rounded-xl text-xs font-semibold transition-all ${
                    modelConfig.mode !== "agent"
                      ? "bg-red-600 text-white shadow-md shadow-red-600/30"
                      : "bg-[#0e0e12] text-neutral-400 border border-neutral-800 hover:text-white"
                  }`}
                >
                  💬 Fast Chat (Instant Response)
                </button>
                <button
                  onClick={() => onUpdateModelConfig({ ...modelConfig, mode: "agent" })}
                  className={`py-2 px-3 rounded-xl text-xs font-semibold transition-all ${
                    modelConfig.mode === "agent"
                      ? "bg-red-600 text-white shadow-md shadow-red-600/30"
                      : "bg-[#0e0e12] text-neutral-400 border border-neutral-800 hover:text-white"
                  }`}
                >
                  🤖 Autonomous Agent (MCP Tools)
                </button>
              </div>
            </div>

            {/* Provider Selector (Only the 5 focused options) */}
            <div className="space-y-1.5">
              <label className="text-[10px] uppercase font-bold text-neutral-400">
                AI Model Provider
              </label>
              <select
                value={modelConfig.provider}
                onChange={(e) => {
                  const p = e.target.value as ModelProvider;
                  const def = PROVIDERS.find((x) => x.id === p)?.defaultModel || "";
                  onUpdateModelConfig({ ...modelConfig, provider: p, model: def, heavyModel: def });
                }}
                className="w-full px-3 py-2 rounded-xl bg-[#0c0c0e] border border-neutral-800 focus:outline-none focus:border-red-500 text-xs font-medium text-neutral-100"
              >
                {PROVIDERS.map((p) => (
                  <option key={p.id} value={p.id}>
                    {p.name} {p.id === "lmstudio" ? "⚡ (Local Server)" : keysConfigured[p.id] ? "✓" : "(Requires Setup)"}
                  </option>
                ))}
              </select>
            </div>

            {/* Model Name Selector */}
            <div className="space-y-1.5">
              <label className="text-[10px] uppercase font-bold text-neutral-400">
                Active Model
              </label>
              <input
                type="text"
                value={modelConfig.model}
                onChange={(e) =>
                  onUpdateModelConfig({ ...modelConfig, model: e.target.value, heavyModel: e.target.value })
                }
                list="provider-models"
                className="w-full px-3 py-2 rounded-xl bg-[#0c0c0e] border border-neutral-800 focus:outline-none focus:border-red-500 text-xs font-mono text-neutral-100"
              />
              <datalist id="provider-models">
                {currentProviderInfo.models.map((m) => (
                  <option key={m} value={m} />
                ))}
                {lmStudioModels.map((m) => (
                  <option key={m} value={m} />
                ))}
              </datalist>
            </div>

            {/* Provider-Specific Configuration Card */}
            <button
              onClick={() => setActiveTab("identity")}
              className="w-full p-2.5 rounded-xl bg-neutral-950 hover:bg-neutral-900 border border-red-950/70 text-neutral-300 hover:text-white text-[11px] font-medium flex items-center justify-between transition-colors shadow-sm"
            >
              <span className="flex items-center gap-2">
                <span className="text-red-500 text-xs">📜</span>
                <span>Customize AI Identity & Foundation Manual</span>
              </span>
              <span className="text-red-400 text-[10px] font-semibold">Edit Manual →</span>
            </button>

            {modelConfig.provider === "gemini" ? (
              /* Google Gemini 3.8 Flash Panel */
              <div className="p-3.5 rounded-2xl bg-[#0c0c0e] border border-red-900/60 space-y-3">
                <div className="flex justify-between items-center text-[11px]">
                  <span className="font-semibold text-neutral-200 flex items-center gap-1.5">
                    <span className="text-red-500 font-bold">G</span> Google Gemini (3.8 Flash)
                  </span>
                  <span
                    className={`px-2 py-0.5 rounded-full text-[9px] font-bold ${
                      keysConfigured.gemini
                        ? "bg-emerald-950 text-emerald-300 border border-emerald-800/80"
                        : "bg-neutral-800 text-neutral-300 border border-neutral-700"
                    }`}
                  >
                    {keysConfigured.gemini ? "Connected 🔒" : "Ready to Connect"}
                  </span>
                </div>

                {/* Direct Google Account Login via Browser */}
                <div className="p-3 rounded-xl bg-neutral-950 border border-neutral-800 flex items-center justify-between gap-2">
                  <div>
                    <div className="font-semibold text-neutral-200 text-xs flex items-center gap-1.5">
                      <span>Google Account Connection</span>
                      <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse inline-block" />
                    </div>
                    <div className="text-[10px] text-emerald-400 font-medium">
                      {googleStatus?.email || "kobbyagyemang7@gmail.com"} (Permanently Connected)
                    </div>
                  </div>
                  <button
                    onClick={handleConnectGoogle}
                    disabled={isConnectingGoogle}
                    className="px-3 py-1.5 rounded-xl bg-neutral-900 hover:bg-neutral-800 border border-neutral-800 disabled:opacity-50 text-neutral-300 font-semibold text-[11px] transition-all whitespace-nowrap"
                  >
                    {isConnectingGoogle ? "Opening..." : "🌐 Re-authenticate"}
                  </button>
                </div>

                {/* API Key Input */}
                <div className="space-y-1.5 pt-1">
                  <label className="text-[10px] uppercase font-bold text-neutral-400">
                    Google Gemini API Key (For sub-300ms direct streaming)
                  </label>
                  <div className="flex gap-1.5">
                    <input
                      type="password"
                      placeholder="Paste AIza... key here"
                      value={keyInput}
                      onChange={(e) => setKeyInput(e.target.value)}
                      className="flex-1 px-3 py-1.5 rounded-xl bg-neutral-950 border border-neutral-800 text-xs font-mono text-neutral-100 focus:outline-none focus:border-red-500"
                    />
                    <button
                      onClick={() => handleSaveKey("gemini")}
                      disabled={!keyInput.trim()}
                      className="px-3 py-1.5 bg-red-600 hover:bg-red-500 disabled:opacity-40 text-white rounded-xl font-semibold text-[11px] transition-colors shadow-md shadow-red-950/40"
                    >
                      Save
                    </button>
                    {keysConfigured.gemini && (
                      <button
                        onClick={() => handleDeleteKey("gemini")}
                        className="px-2 py-1.5 bg-neutral-900 hover:bg-neutral-800 text-red-400 rounded-xl text-[11px] border border-neutral-800"
                        title="Clear key from Windows Credential Manager"
                      >
                        Clear
                      </button>
                    )}
                  </div>
                </div>

                {saveStatus && (
                  <div className="text-[11px] text-red-400 font-medium">
                    {saveStatus}
                  </div>
                )}
              </div>
            ) : modelConfig.provider === "lmstudio" ? (
              /* LM Studio Local Server Panel */
              <div className="p-3.5 rounded-2xl bg-[#0c0c0e] border border-red-900/60 space-y-3">
                <div className="flex justify-between items-center text-[11px]">
                  <span className="font-semibold text-neutral-200 flex items-center gap-1.5">
                    <span className="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
                    LM Studio (Local Server)
                  </span>
                  <span className="px-2 py-0.5 rounded-full text-[9px] font-bold bg-emerald-950 text-emerald-300 border border-emerald-800/80">
                    Local • Zero Latency ⚡
                  </span>
                </div>

                <div className="space-y-1.5">
                  <label className="text-[10px] uppercase font-bold text-neutral-400">
                    Server Endpoint URL
                  </label>
                  <div className="flex gap-1.5">
                    <input
                      type="text"
                      placeholder="http://localhost:1234/v1"
                      value={modelConfig.customEndpoint || "http://localhost:1234/v1"}
                      onChange={(e) =>
                        onUpdateModelConfig({
                          ...modelConfig,
                          customEndpoint: e.target.value || undefined,
                        })
                      }
                      className="flex-1 px-3 py-1.5 rounded-xl bg-neutral-950 border border-neutral-800 text-xs font-mono text-neutral-100 focus:outline-none focus:border-red-500"
                    />
                    <button
                      onClick={handleFetchLmStudioModels}
                      disabled={isFetchingLmStudio}
                      className="px-3.5 py-1.5 rounded-xl bg-red-600 hover:bg-red-500 disabled:opacity-40 text-white font-semibold text-[11px] transition-colors whitespace-nowrap shadow-md shadow-red-950/40"
                    >
                      {isFetchingLmStudio ? "Connecting..." : "Fetch Models"}
                    </button>
                  </div>
                </div>

                {lmStudioStatus && (
                  <div className="p-2.5 rounded-xl bg-neutral-950 border border-neutral-800 text-[11px] text-neutral-300 leading-tight">
                    {lmStudioStatus}
                  </div>
                )}
              </div>
            ) : (
              /* Cloud Providers (OpenAI, Anthropic, DeepSeek) */
              <div className="p-3.5 rounded-2xl bg-[#0c0c0e] border border-red-900/60 space-y-3">
                <div className="flex justify-between items-center text-[11px]">
                  <span className="font-semibold text-neutral-200">{currentProviderInfo.name} API Key</span>
                  <span
                    className={`px-2 py-0.5 rounded-full text-[9px] font-bold ${
                      keysConfigured[modelConfig.provider]
                        ? "bg-emerald-950 text-emerald-300 border border-emerald-800/80"
                        : "bg-red-950 text-red-300 border border-red-800/80"
                    }`}
                  >
                    {keysConfigured[modelConfig.provider] ? "Configured 🔒" : "Key Required"}
                  </span>
                </div>

                <div className="flex gap-1.5">
                  <input
                    type="password"
                    placeholder={`Paste ${currentProviderInfo.name} API Key here…`}
                    value={keyInput}
                    onChange={(e) => setKeyInput(e.target.value)}
                    className="flex-1 px-3 py-1.5 rounded-xl bg-neutral-950 border border-neutral-800 text-xs font-mono text-neutral-100 focus:outline-none focus:border-red-500"
                  />
                  <button
                    onClick={() => handleSaveKey(modelConfig.provider)}
                    disabled={!keyInput.trim()}
                    className="px-3.5 py-1.5 bg-red-600 hover:bg-red-500 disabled:opacity-40 text-white rounded-xl font-semibold text-[11px] transition-colors shadow-md shadow-red-950/40"
                  >
                    Save
                  </button>
                  {keysConfigured[modelConfig.provider] && (
                    <button
                      onClick={() => handleDeleteKey(modelConfig.provider)}
                      className="px-2 py-1.5 bg-neutral-900 hover:bg-neutral-800 text-red-400 rounded-xl text-[11px] border border-neutral-800"
                      title="Clear key from Windows Credential Manager"
                    >
                      Clear
                    </button>
                  )}
                </div>

                {saveStatus && (
                  <div className="text-[11px] text-red-400 font-medium">
                    {saveStatus}
                  </div>
                )}
              </div>
            )}
          </div>
        )}

        {activeTab === "computer_use" && (
          <div className="space-y-3">
            {/* 1. Operating Modes Overview */}
            <div className="p-3.5 rounded-2xl bg-[#0c0c0e] border border-red-950/80 space-y-2.5">
              <div className="flex items-center justify-between text-[11px]">
                <span className="font-bold text-neutral-100 flex items-center gap-1.5">
                  <span className="text-red-500 text-sm">👁️</span>
                  Vision & Operating Modes
                </span>
                <span className="px-2 py-0.5 rounded-full text-[9px] font-bold bg-neutral-900 border border-red-900/40 text-red-400">
                  Per-Monitor DPI V2
                </span>
              </div>
              <p className="text-[10px] text-neutral-400 leading-relaxed">
                ALIEN can visually observe your screen on demand and visibly operate desktop software with real mouse & keyboard input.
              </p>

              <div className="grid grid-cols-2 gap-2 pt-1">
                <div className="p-2.5 rounded-xl bg-neutral-900/60 border border-neutral-800 space-y-1">
                  <div className="flex items-center justify-between">
                    <span className="text-[10px] font-bold text-cyan-400 flex items-center gap-1">
                      👁️ WATCH Mode
                    </span>
                    <span className={`text-[9px] px-1.5 py-0.2 rounded font-semibold ${isWatchActive ? "bg-cyan-900/60 text-cyan-200" : "bg-neutral-800 text-neutral-400"}`}>
                      {isWatchActive ? "ACTIVE" : "OFF"}
                    </span>
                  </div>
                  <p className="text-[9px] text-neutral-400">
                    Continuous ~1 fps visual observation stored in a 10-frame RAM ring buffer.
                  </p>
                  <button
                    onClick={async () => {
                      if (isWatchActive) {
                        await stopWatchMode();
                        setIsWatchActive(false);
                      } else {
                        await startWatchMode();
                        setIsWatchActive(true);
                      }
                    }}
                    className={`w-full mt-1.5 py-1 px-2 rounded-lg text-[10px] font-bold transition-all ${
                      isWatchActive
                        ? "bg-neutral-800 hover:bg-neutral-700 text-neutral-200"
                        : "bg-cyan-700 hover:bg-cyan-600 text-white"
                    }`}
                  >
                    {isWatchActive ? "Stop Watch Mode" : "Start Watch Mode"}
                  </button>
                </div>

                <div className="p-2.5 rounded-xl bg-neutral-900/60 border border-neutral-800 space-y-1">
                  <div className="flex items-center justify-between">
                    <span className="text-[10px] font-bold text-red-400 flex items-center gap-1">
                      🔴 AGENT Presence
                    </span>
                    <span className={`text-[9px] px-1.5 py-0.2 rounded font-semibold ${isOverlayActive ? "bg-red-900/60 text-red-200" : "bg-neutral-800 text-neutral-400"}`}>
                      {isOverlayActive ? "VISIBLE" : "OFF"}
                    </span>
                  </div>
                  <p className="text-[9px] text-neutral-400">
                    Subtle red edge glow on all borders & red cursor. Excluded from screenshots.
                  </p>
                  <button
                    onClick={async () => {
                      const next = !isOverlayActive;
                      await toggleAgentPresence(next);
                      setIsOverlayActive(next);
                    }}
                    className={`w-full mt-1.5 py-1 px-2 rounded-lg text-[10px] font-bold transition-all ${
                      isOverlayActive
                        ? "bg-neutral-800 hover:bg-neutral-700 text-neutral-200"
                        : "bg-red-700 hover:bg-red-600 text-white"
                    }`}
                  >
                    {isOverlayActive ? "Hide Overlay & Cursor" : "Test Red Glow & Cursor"}
                  </button>
                </div>
              </div>
            </div>

            {/* 2. Safety & Privacy Blocklist */}
            <div className="p-3.5 rounded-2xl bg-[#0c0c0e] border border-red-950/80 space-y-2.5">
              <div className="flex items-center justify-between text-[11px]">
                <span className="font-bold text-neutral-100 flex items-center gap-1.5">
                  <span className="text-amber-500 text-sm">🛡️</span>
                  Privacy & Application Blocklist
                </span>
                <button
                  onClick={async () => {
                    const next = !blocklistEnabled;
                    await toggleSafetyBlocklist(next);
                    setBlocklistEnabled(next);
                  }}
                  className={`px-2 py-0.5 rounded-full text-[9px] font-bold transition-colors ${
                    blocklistEnabled
                      ? "bg-amber-950/60 border border-amber-600/40 text-amber-300"
                      : "bg-neutral-800 text-neutral-400"
                  }`}
                >
                  {blocklistEnabled ? "Active & Enforced" : "Disabled"}
                </button>
              </div>
              <p className="text-[10px] text-neutral-400 leading-relaxed">
                ALIEN automatically pauses and excludes capture whenever windows or tabs matching these keywords gain focus (password managers, banking sites, secure dialogs).
              </p>

              <div className="flex flex-wrap gap-1.5 max-h-28 overflow-y-auto chat-scroll p-1.5 rounded-xl bg-neutral-950/60 border border-neutral-800">
                {blocklist.map((kw) => (
                  <span
                    key={kw}
                    className="inline-flex items-center gap-1 px-2 py-0.5 rounded-lg text-[10px] font-medium bg-neutral-900 border border-neutral-700/60 text-neutral-300"
                  >
                    {kw}
                    <button
                      onClick={async () => {
                        const updated = await removeSafetyBlocklistKeyword(kw);
                        setBlocklist(updated);
                      }}
                      className="text-neutral-500 hover:text-red-400 font-bold text-[9px] ml-0.5"
                    >
                      ✕
                    </button>
                  </span>
                ))}
              </div>

              <div className="flex gap-2 pt-1">
                <input
                  type="text"
                  placeholder="Add keyword (e.g. 1password, bank)..."
                  value={blocklistInput}
                  onChange={(e) => setBlocklistInput(e.target.value)}
                  onKeyDown={async (e) => {
                    if (e.key === "Enter" && blocklistInput.trim()) {
                      const updated = await addSafetyBlocklistKeyword(blocklistInput.trim());
                      setBlocklist(updated);
                      setBlocklistInput("");
                    }
                  }}
                  className="flex-1 px-2.5 py-1.5 rounded-xl bg-neutral-900 border border-neutral-800 text-[10px] text-neutral-200 placeholder:text-neutral-500 focus:outline-none focus:border-red-600"
                />
                <button
                  onClick={async () => {
                    if (blocklistInput.trim()) {
                      const updated = await addSafetyBlocklistKeyword(blocklistInput.trim());
                      setBlocklist(updated);
                      setBlocklistInput("");
                    }
                  }}
                  className="px-3 py-1.5 rounded-xl bg-red-700 hover:bg-red-600 text-white font-bold text-[10px] transition-colors"
                >
                  Add
                </button>
              </div>
            </div>

            {/* 3. Crash-Safe Cursor Recovery & Hardware Kill Switch */}
            <div className="p-3.5 rounded-2xl bg-[#0c0c0e] border border-red-950/80 space-y-2.5">
              <div className="flex items-center justify-between text-[11px]">
                <span className="font-bold text-neutral-100 flex items-center gap-1.5">
                  <span className="text-red-500 text-sm">🛑</span>
                  Safety Controls & Cursor Rescue
                </span>
                <span className="px-2 py-0.5 rounded-full text-[9px] font-bold bg-neutral-900 border border-neutral-700 text-neutral-300">
                  Global Hook Active
                </span>
              </div>

              <div className="grid grid-cols-2 gap-2 text-[10px]">
                <div className="p-2 rounded-xl bg-neutral-900/60 border border-neutral-800 space-y-1">
                  <span className="text-neutral-400 font-semibold block">Kill Switch Hotkey</span>
                  <span className="text-red-400 font-mono font-bold text-[11px]">Ctrl + Shift + F12</span>
                  <p className="text-[9px] text-neutral-500">Hardware abort hook: instantly terminates PC takeover and resets cursor.</p>
                </div>
                <div className="p-2 rounded-xl bg-neutral-900/60 border border-neutral-800 space-y-1">
                  <span className="text-neutral-400 font-semibold block">Hardware Takeover</span>
                  <span className="text-emerald-400 font-mono font-bold text-[11px]">Physical Mouse / Key</span>
                  <p className="text-[9px] text-neutral-500">Touching your physical mouse instantly yields control back to you.</p>
                </div>
              </div>

              <div className="pt-1">
                <button
                  onClick={async () => {
                    try {
                      const msg = await restoreSystemCursors();
                      setCursorRestoreMsg(msg);
                      setIsOverlayActive(false);
                      setTimeout(() => setCursorRestoreMsg(null), 3000);
                    } catch (e: any) {
                      setCursorRestoreMsg(`Error: ${e?.message || e}`);
                    }
                  }}
                  className="w-full py-2 px-3 rounded-xl bg-neutral-900 hover:bg-neutral-800 border border-neutral-700 text-neutral-200 font-semibold text-[11px] transition-colors flex items-center justify-center gap-2"
                >
                  <span>🔄</span>
                  Restore Default System Cursors
                </button>
                {cursorRestoreMsg && (
                  <p className="text-[10px] text-center text-emerald-400 font-medium pt-1">
                    {cursorRestoreMsg}
                  </p>
                )}
              </div>
            </div>

            {/* 4. Live Telemetry & Cost Display */}
            <div className="p-3.5 rounded-2xl bg-[#0c0c0e] border border-red-950/80 space-y-2">
              <div className="flex items-center justify-between text-[11px]">
                <span className="font-bold text-neutral-100 flex items-center gap-1.5">
                  <span className="text-emerald-500 text-sm">📊</span>
                  Session Telemetry & Cost
                </span>
                <span className="px-2 py-0.5 rounded-full text-[9px] font-bold bg-neutral-900 border border-emerald-900/40 text-emerald-400">
                  Zero Telemetry
                </span>
              </div>

              <div className="grid grid-cols-4 gap-1.5 pt-1 text-center">
                <div className="p-2 rounded-xl bg-neutral-900/60 border border-neutral-800">
                  <span className="text-[9px] text-neutral-400 block font-medium">Steps</span>
                  <span className="text-xs font-bold text-neutral-100">{metrics?.total_steps ?? 0}</span>
                </div>
                <div className="p-2 rounded-xl bg-neutral-900/60 border border-neutral-800">
                  <span className="text-[9px] text-neutral-400 block font-medium">Frames</span>
                  <span className="text-xs font-bold text-neutral-100">{metrics?.total_frames ?? 0}</span>
                </div>
                <div className="p-2 rounded-xl bg-neutral-900/60 border border-neutral-800">
                  <span className="text-[9px] text-neutral-400 block font-medium">Tokens</span>
                  <span className="text-xs font-bold text-cyan-400">~{metrics?.estimated_tokens.toLocaleString() ?? 0}</span>
                </div>
                <div className="p-2 rounded-xl bg-neutral-900/60 border border-neutral-800">
                  <span className="text-[9px] text-neutral-400 block font-medium">Est. Cost</span>
                  <span className="text-xs font-bold text-emerald-400">${metrics?.estimated_cost_usd?.toFixed(4) ?? "0.0000"}</span>
                </div>
              </div>

              <div className="flex items-center justify-between text-[9px] text-neutral-400 pt-1 border-t border-neutral-900">
                <span>Avg Latency: <strong className="text-neutral-300">{metrics?.average_step_ms ?? 0} ms/step</strong></span>
                <span>Provider: <strong className="text-red-400">{metrics?.provider || "Google Gemini"}</strong></span>
              </div>
            </div>
          </div>
        )}

        {activeTab === "identity" && (
          <div className="space-y-3">
            <div className="p-3.5 rounded-2xl bg-[#0c0c0e] border border-red-950/80 space-y-2">
              <div className="flex justify-between items-center text-[11px]">
                <span className="font-bold text-neutral-100 flex items-center gap-1.5">
                  <span className="text-red-500 text-sm">📜</span>
                  AI Identity & Foundation Manual
                </span>
                <span className="px-2 py-0.5 rounded-full text-[9px] font-bold bg-neutral-900 border border-red-900/40 text-red-400">
                  Foundation Directives • Active
                </span>
              </div>
              <p className="text-[10px] text-neutral-400 leading-relaxed">
                Define ALIEN's core persona, operating directives, WebToon priorities, and behavioral foundation. This manual is injected into every thought, conversational reply, and autonomous tool execution.
              </p>
            </div>

            <div className="space-y-1.5">
              <div className="flex items-center justify-between">
                <label className="text-[10px] uppercase font-bold text-neutral-400">
                  Foundation Manual & System Directives
                </label>
                <span className="text-[10px] text-neutral-500 font-mono">
                  {systemIdentity.length} chars • {systemIdentity.split(/\s+/).filter(Boolean).length} words
                </span>
              </div>
              <textarea
                value={systemIdentity}
                onChange={(e) => setSystemIdentity(e.target.value)}
                placeholder="Write your custom identity and instruction foundation manual here..."
                rows={15}
                className="w-full p-3 rounded-2xl bg-[#0c0c0e] border border-neutral-800 text-neutral-100 font-mono text-[11px] leading-relaxed placeholder-neutral-600 focus:outline-none focus:border-red-500 transition-colors resize-y chat-scroll"
              />
            </div>

            {identityStatus && (
              <div className="p-2.5 rounded-xl bg-neutral-900 border border-red-900/40 text-red-400 text-[11px] font-medium flex items-center gap-2">
                <span>⚡</span>
                <span>{identityStatus}</span>
              </div>
            )}

            <div className="flex items-center justify-between pt-1">
              <button
                onClick={handleResetIdentity}
                className="px-3 py-1.5 bg-neutral-900 hover:bg-neutral-800 text-neutral-300 rounded-xl text-[11px] font-medium border border-neutral-800 transition-colors"
                title="Restore default recommended ALIEN identity"
              >
                🔄 Reset to Default
              </button>

              <button
                onClick={handleSaveIdentity}
                disabled={isSavingIdentity}
                className="px-4 py-2 bg-red-600 hover:bg-red-500 disabled:opacity-50 text-white rounded-xl text-[11px] font-semibold transition-all shadow-md shadow-red-950/40 flex items-center gap-1.5"
              >
                {isSavingIdentity ? "Saving..." : "💾 Save Foundation Manual"}
              </button>
            </div>

            <div className="p-3 rounded-xl bg-neutral-950 border border-neutral-800 text-[10px] text-neutral-400 space-y-1">
              <div className="font-semibold text-neutral-300">Obsidian & Local Sync</div>
              <div>
                Saved copies are automatically written to your Obsidian Vault under <code className="text-red-400 bg-neutral-900 px-1 py-0.5 rounded">Instructions/Identity & System Manual.md</code> and app memory.
              </div>
            </div>
          </div>
        )}

        {activeTab === "skills" && (
          <div className="space-y-3">
            <div className="flex justify-between items-center">
              <div>
                <div className="font-semibold text-neutral-200 text-xs">
                  Installed Plug-in Skills
                </div>
                <div className="text-[10px] text-neutral-400">
                  Drop folders into ./skills with manifest.json
                </div>
              </div>
              <button
                onClick={handleReloadSkills}
                className="px-2.5 py-1 rounded-xl bg-red-600 hover:bg-red-500 text-white font-medium text-[11px] transition-colors"
              >
                Reload
              </button>
            </div>

            <div className="space-y-2">
              {skills.length === 0 ? (
                <div className="text-center py-6 text-neutral-400">No custom skills found.</div>
              ) : (
                skills.map((s) => (
                  <div
                    key={s.manifest.id}
                    className="p-3 rounded-2xl bg-neutral-900 border border-neutral-800 space-y-1"
                  >
                    <div className="flex justify-between items-center text-[11px]">
                      <span className="font-semibold text-red-400">{s.manifest.name}</span>
                      <span className="text-[9px] font-mono text-neutral-400">v{s.manifest.version}</span>
                    </div>
                    <p className="text-[10px] text-neutral-400">
                      {s.manifest.description}
                    </p>
                    <div className="flex gap-1 pt-1">
                      {s.manifest.tools.map((t) => (
                        <span
                          key={t.name}
                          className="px-1.5 py-0.5 rounded bg-red-950/60 border border-red-900/40 text-red-300 font-mono text-[9px]"
                        >
                          {t.name}
                        </span>
                      ))}
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>
        )}

        {activeTab === "memory" && (
          <div className="space-y-3">
            {/* Status & Connection Card */}
            <div className="p-3.5 rounded-2xl bg-[#08080b] border border-red-950/80 space-y-3">
              <div className="flex justify-between items-center text-[11px]">
                <span className="font-bold text-neutral-100 flex items-center gap-1.5">
                  <span className={`w-2 h-2 rounded-full ${vaultInfo?.isConnected ? "bg-emerald-500 animate-pulse" : "bg-red-500"}`} />
                  Obsidian Vault Memory
                </span>
                <span
                  className={`px-2 py-0.5 rounded-full text-[9px] font-bold ${
                    vaultInfo?.isConnected
                      ? "bg-emerald-950 text-emerald-300 border border-emerald-800/80"
                      : "bg-red-950 text-red-300 border border-red-800/80"
                  }`}
                >
                  {vaultInfo?.isConnected ? "Connected 🧠" : "Not Linked"}
                </span>
              </div>

              {vaultInfo?.isConnected ? (
                <div className="p-2.5 rounded-xl bg-[#0c0c0e] border border-neutral-800/80 space-y-1.5 font-mono text-[10px] text-neutral-300 break-all">
                  <div className="flex items-center gap-1 text-emerald-400 font-semibold text-[11px]">
                    <span>✓ Active Vault</span>
                  </div>
                  <div className="text-neutral-200">📁 {vaultInfo.path}</div>
                  <div className="text-neutral-400">📄 {vaultInfo.noteCount} total notes indexed</div>
                </div>
              ) : (
                <p className="text-[11px] text-neutral-300 leading-relaxed">
                  Link ALIEN directly to your local Obsidian vault. ALIEN will continuously record markdown notes, update your daily logs, track your preferences, and build knowledge with [[wikilinks]].
                </p>
              )}

              {/* Folder Pointing Section */}
              <div className="space-y-2 pt-1">
                <label className="block text-[10px] font-bold text-neutral-400 tracking-wider uppercase">
                  Vault Folder Path
                </label>
                <div className="flex gap-1.5">
                  <input
                    type="text"
                    value={vaultPathInput}
                    onChange={(e) => setVaultPathInput(e.target.value)}
                    placeholder="Enter or paste full folder path..."
                    className="flex-1 px-3 py-2 rounded-xl bg-[#0c0c0e] border border-neutral-800 text-xs font-mono text-neutral-100 placeholder:text-neutral-600 focus:outline-none focus:border-red-500"
                  />
                  <button
                    onClick={() => handleConnectVault()}
                    disabled={isConnectingVault || !vaultPathInput.trim()}
                    className="px-3.5 py-2 rounded-xl bg-red-600 hover:bg-red-500 disabled:opacity-40 text-white font-semibold text-[11px] transition-colors shadow-md shadow-red-900/30 whitespace-nowrap"
                  >
                    {isConnectingVault ? "Linking..." : "Link Vault"}
                  </button>
                  <button
                    onClick={handleSelectVaultFolder}
                    className="px-3 py-2 rounded-xl bg-neutral-900 hover:bg-neutral-800 text-neutral-200 font-medium text-[11px] transition-colors border border-neutral-800 whitespace-nowrap"
                    title="Browse your computer for folder"
                  >
                    Browse...
                  </button>
                </div>

                {/* Quick Detect Presets */}
                <div className="flex items-center gap-1.5 pt-1 text-[10px]">
                  <span className="text-neutral-400">Quick link:</span>
                  <button
                    onClick={() => {
                      const p = "C:\\Users\\Kobby\\Documents\\Future Heroes\\ALIEN's Mind";
                      setVaultPathInput(p);
                      handleConnectVault(p);
                    }}
                    className="px-2 py-0.5 rounded-lg bg-neutral-900 hover:bg-red-950/60 hover:text-red-300 text-neutral-300 border border-neutral-800 text-[10px] transition-colors"
                  >
                    ⚡ Future Heroes / ALIEN's Mind
                  </button>
                </div>
              </div>

              {/* Status & Feedback Alert */}
              {vaultStatusMsg && (
                <div
                  className={`p-2.5 rounded-xl border text-[11px] font-medium leading-tight ${
                    vaultStatusMsg.isError
                      ? "bg-red-950/60 border-red-800/80 text-red-300"
                      : "bg-emerald-950/60 border-emerald-800/80 text-emerald-300"
                  }`}
                >
                  {vaultStatusMsg.text}
                </div>
              )}

              {/* Action Buttons for Active Vault */}
              {vaultInfo?.isConnected && (
                <div className="flex gap-2 pt-1 border-t border-neutral-900">
                  <button
                    onClick={handleCreateTestNote}
                    disabled={isTestingVault}
                    className="flex-1 py-1.5 px-3 rounded-xl bg-neutral-900 hover:bg-neutral-800 text-red-400 border border-red-900/40 text-[11px] font-medium transition-colors"
                  >
                    {isTestingVault ? "Creating..." : "📝 Write Test Note"}
                  </button>
                  <button
                    onClick={handleOpenExplorer}
                    className="py-1.5 px-3 rounded-xl bg-neutral-900 hover:bg-neutral-800 text-neutral-300 border border-neutral-800 text-[11px] font-medium transition-colors"
                  >
                    📂 Open in Explorer
                  </button>
                </div>
              )}
            </div>

            {/* Vault Categories Breakdown */}
            {vaultInfo?.isConnected && vaultInfo.categories.length > 0 && (
              <div className="space-y-1.5">
                <label className="text-[10px] uppercase font-bold text-neutral-400 tracking-wider">
                  Vault Categories Indexed
                </label>
                <div className="grid grid-cols-2 gap-1.5">
                  {vaultInfo.categories.map((c) => (
                    <div
                      key={c.name}
                      className="p-2 rounded-xl bg-[#08080b] border border-neutral-800/80 flex justify-between items-center text-[11px]"
                    >
                      <span className="text-neutral-300">{c.name}</span>
                      <span className="font-mono text-red-400 font-semibold">{c.count} notes</span>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        )}

        {activeTab === "learning" && (
          <div className="space-y-3">
            {/* Passive preference learning */}
            <div className="flex items-center justify-between p-3 rounded-2xl bg-neutral-900 border border-neutral-800">
              <div>
                <div className="font-semibold text-neutral-200">
                  Passive Preference Learning
                </div>
                <div className="text-[10px] text-neutral-400">
                  Extracts habits & style from chat & actions
                </div>
              </div>
              <button
                onClick={() =>
                  handleUpdateLearning({
                    passive_learning: !learningConfig.passive_learning,
                  })
                }
                className={`px-3 py-1.5 rounded-xl font-semibold text-xs transition-all ${
                  learningConfig.passive_learning
                    ? "bg-red-600 text-white shadow-sm shadow-red-600/30"
                    : "bg-neutral-800 text-neutral-400"
                }`}
              >
                {learningConfig.passive_learning ? "ACTIVE" : "PAUSED"}
              </button>
            </div>

            {/* Scenario questions */}
            <div className="flex items-center justify-between p-3 rounded-2xl bg-neutral-900 border border-neutral-800">
              <div>
                <div className="font-semibold text-neutral-200">
                  Getting to Know You
                </div>
                <div className="text-[10px] text-neutral-400">
                  Occasional friendly questions when idle
                </div>
              </div>
              <button
                onClick={() =>
                  handleUpdateLearning({
                    scenario_questions: !learningConfig.scenario_questions,
                  })
                }
                className={`px-3 py-1.5 rounded-xl font-semibold text-xs transition-all ${
                  learningConfig.scenario_questions
                    ? "bg-red-600 text-white shadow-sm shadow-red-600/30"
                    : "bg-neutral-800 text-neutral-400"
                }`}
              >
                {learningConfig.scenario_questions ? "ENABLED" : "DISABLED"}
              </button>
            </div>

            {/* Active mindset preview */}
            <div className="space-y-1">
              <label className="text-[10px] uppercase font-bold text-neutral-400">
                Active Mindset Profile (How Buddy Thinks You Think)
              </label>
              <div className="p-3 rounded-2xl bg-neutral-900 border border-neutral-800 font-mono text-[11px] text-neutral-300 leading-relaxed whitespace-pre-wrap">
                {thinkingProfile?.summary || "No mindset synthesized yet."}
              </div>
            </div>
          </div>
        )}

        {activeTab === "voice" && (
          <div className="space-y-3">
            {/* Auto speak response */}
            <div className="flex items-center justify-between p-3 rounded-2xl bg-neutral-900 border border-neutral-800">
              <div>
                <div className="font-semibold text-neutral-200">
                  Spoken Replies (TTS)
                </div>
                <div className="text-[10px] text-neutral-400">
                  Read out answers naturally
                </div>
              </div>
              <button
                onClick={() =>
                  onUpdateVoiceConfig({
                    ...voiceConfig,
                    autoSpeak: !voiceConfig.autoSpeak,
                  })
                }
                className={`px-3 py-1.5 rounded-xl font-semibold text-xs transition-all ${
                  voiceConfig.autoSpeak
                    ? "bg-red-600 text-white shadow-sm shadow-red-600/30"
                    : "bg-neutral-800 text-neutral-400"
                }`}
              >
                {voiceConfig.autoSpeak ? "ENABLED" : "MUTED"}
              </button>
            </div>

            {/* Voice selection */}
            <div className="space-y-1">
              <label className="text-[10px] uppercase font-bold text-neutral-400">
                Voice Selection
              </label>
              <select
                value={voiceConfig.voiceName}
                onChange={(e) =>
                  onUpdateVoiceConfig({ ...voiceConfig, voiceName: e.target.value })
                }
                className="w-full px-2.5 py-1.5 rounded-xl bg-neutral-900 border border-neutral-800 text-xs text-neutral-100"
              >
                <option value="">Default Friendly Voice</option>
                {availableVoices.map((v) => (
                  <option key={v.name} value={v.name}>
                    {v.name} ({v.lang})
                  </option>
                ))}
              </select>
            </div>

            {/* Rate / Speed slider */}
            <div className="space-y-1 p-2">
              <div className="flex justify-between text-[11px] text-neutral-400">
                <span>Speed</span>
                <span>{voiceConfig.rate}x</span>
              </div>
              <input
                type="range"
                min="0.8"
                max="1.4"
                step="0.05"
                value={voiceConfig.rate}
                onChange={(e) =>
                  onUpdateVoiceConfig({
                    ...voiceConfig,
                    rate: parseFloat(e.target.value),
                  })
                }
                className="w-full accent-red-600"
              />
            </div>
          </div>
        )}

        {activeTab === "general" && (
          <>
            {/* Autostart on Boot */}
            <div className="flex items-center justify-between p-3 rounded-2xl bg-neutral-900 border border-neutral-800">
              <div>
                <div className="font-semibold text-neutral-200">
                  Launch on Windows Startup
                </div>
                <div className="text-[10px] text-neutral-400">
                  Start Buddy minimized in system tray
                </div>
              </div>
              <button
                onClick={handleToggleAutostart}
                className={`px-3 py-1.5 rounded-xl font-semibold text-xs transition-all ${
                  autostartEnabled
                    ? "bg-red-600 text-white shadow-sm shadow-red-600/30"
                    : "bg-neutral-800 text-neutral-400"
                }`}
              >
                {autostartEnabled ? "ENABLED" : "DISABLED"}
              </button>
            </div>

            {/* Kill Switch Card */}
            <div className="p-3 rounded-2xl border border-red-900/60 bg-red-950/20 space-y-2">
              <div className="flex items-center justify-between">
                <div>
                  <div className="font-semibold text-red-400">
                    Emergency Kill Switch
                  </div>
                  <div className="text-[10px] text-red-400/80">
                    Immediately aborts & blocks all actions
                  </div>
                </div>
                <button
                  onClick={handleToggleKillSwitch}
                  className={`px-3 py-1.5 rounded-xl font-semibold text-[11px] transition-all shadow-sm ${
                    killSwitch
                      ? "bg-red-600 text-white hover:bg-red-700 animate-pulse"
                      : "bg-neutral-800 text-neutral-300 hover:bg-neutral-700"
                  }`}
                >
                  {killSwitch ? "ACTIVE (BLOCKED)" : "ENGAGE"}
                </button>
              </div>
            </div>

            {/* General Info */}
            <div className="p-3 rounded-2xl bg-neutral-900 border border-neutral-800 space-y-2">
              <div className="flex justify-between items-center">
                <span className="text-neutral-400">Summon Shortcut</span>
                <span className="font-mono bg-neutral-800 text-red-400 border border-neutral-700 px-2 py-0.5 rounded text-[11px]">
                  Ctrl+Space
                </span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-neutral-400">Security Vault</span>
                <span className="font-medium text-[11px] text-neutral-200">Windows Credential Manager</span>
              </div>
            </div>

            {/* Actions */}
            <div className="pt-2">
              <button
                onClick={() => {
                  onClearChat();
                  onClose();
                }}
                className="w-full py-2 rounded-xl bg-neutral-900 hover:bg-neutral-800 text-neutral-300 font-medium transition-colors border border-neutral-800 hover:border-red-900"
              >
                Clear Chat History
              </button>
            </div>
          </>
        )}

        {activeTab === "video_editor" && (
          <div className="space-y-3">
            {/* Header & Status */}
            <div className="p-3 rounded-xl bg-neutral-900 border border-neutral-800 space-y-2">
              <div className="flex items-center justify-between">
                <div>
                  <div className="font-bold text-neutral-200 text-xs">🎬 FilmCraft MCP Integration</div>
                  <div className="text-[10px] text-neutral-400">
                    Autonomous video editing powered by FilmCraft
                  </div>
                </div>
                <span
                  className={`px-2 py-0.5 rounded-full text-[10px] font-bold uppercase ${
                    fcStatus?.connection === "Connected"
                      ? "bg-emerald-950/80 text-emerald-300 border border-emerald-700"
                      : fcStatus?.connection === "Connecting"
                      ? "bg-amber-950/80 text-amber-300 border border-amber-700"
                      : typeof fcStatus?.connection === "object"
                      ? "bg-red-950/80 text-red-300 border border-red-700"
                      : "bg-neutral-800 text-neutral-400 border border-neutral-700"
                  }`}
                >
                  {typeof fcStatus?.connection === "object"
                    ? "Error"
                    : fcStatus?.connection || "Disconnected"}
                </span>
              </div>

              <div className="text-[10px] font-mono text-neutral-400 bg-black/40 p-2 rounded-lg border border-neutral-800/80 break-all">
                CLI: {fcStatus?.cli_path || fcCliPath || "Not detected"}
              </div>

              {fcStatus?.last_error && (
                <div className="text-[10px] text-red-400 bg-red-950/40 p-2 rounded-lg border border-red-900/60">
                  {fcStatus.last_error}
                </div>
              )}
              {fcMsg && (
                <div
                  className={`text-[10px] p-2 rounded-lg border ${
                    fcMsg.isError
                      ? "text-red-300 bg-red-950/40 border-red-900/60"
                      : "text-emerald-300 bg-emerald-950/40 border-emerald-900/60"
                  }`}
                >
                  {fcMsg.text}
                </div>
              )}
            </div>

            {/* Operating Mode */}
            <div className="p-3 rounded-xl bg-neutral-900 border border-neutral-800 space-y-2">
              <label className="text-[10px] uppercase font-bold text-neutral-400">
                MCP Launch Mode
              </label>
              <div className="grid grid-cols-2 gap-1.5">
                {(
                  [
                    ["headless", "Headless (Empty)"],
                    ["demo", "Headless (Demo)"],
                    ["project", "Headless (.fcproj)"],
                    ["bridge", "Desktop Bridge"],
                  ] as const
                ).map(([m, label]) => (
                  <button
                    key={m}
                    onClick={() => setFcLaunchMode(m)}
                    className={`py-1.5 px-2 rounded-lg text-[10px] font-medium border transition-colors ${
                      fcLaunchMode === m
                        ? "bg-red-600/90 text-white border-red-500"
                        : "bg-neutral-950 text-neutral-400 border-neutral-800 hover:text-neutral-200"
                    }`}
                  >
                    {label}
                  </button>
                ))}
              </div>

              {fcLaunchMode === "project" && (
                <input
                  type="text"
                  value={fcProjectPath}
                  onChange={(e) => setFcProjectPath(e.target.value)}
                  placeholder="C:\path\to\project.fcproj"
                  className="w-full px-2.5 py-1.5 rounded-lg bg-neutral-950 border border-neutral-800 text-neutral-200 text-[11px] font-mono"
                />
              )}
              {fcLaunchMode === "bridge" && (
                <input
                  type="text"
                  value={fcBridgeAddr}
                  onChange={(e) => setFcBridgeAddr(e.target.value)}
                  placeholder="127.0.0.1:9886"
                  className="w-full px-2.5 py-1.5 rounded-lg bg-neutral-950 border border-neutral-800 text-neutral-200 text-[11px] font-mono"
                />
              )}

              <div className="flex gap-2 pt-1">
                <button
                  disabled={fcConnecting || fcStatus?.connection === "Connected"}
                  onClick={async () => {
                    setFcConnecting(true);
                    setFcMsg(null);
                    try {
                      const st = await filmcraftConnect({
                        mode: fcLaunchMode,
                        projectPath: fcLaunchMode === "project" ? fcProjectPath : undefined,
                        bridgeAddr: fcLaunchMode === "bridge" ? fcBridgeAddr : undefined,
                      });
                      setFcStatus(st);
                      setFcMsg({
                        text: `Connected! ${st.tools_count} FilmCraft MCP tools discovered.`,
                        isError: false,
                      });
                    } catch (e: any) {
                      setFcMsg({ text: String(e), isError: true });
                      filmcraftGetStatus().then(setFcStatus).catch(() => {});
                    } finally {
                      setFcConnecting(false);
                    }
                  }}
                  className="flex-1 py-1.5 rounded-lg bg-red-600 hover:bg-red-500 disabled:opacity-40 text-white font-semibold text-[11px] transition-colors"
                >
                  {fcConnecting ? "Connecting..." : "Connect MCP"}
                </button>
                <button
                  disabled={fcStatus?.connection !== "Connected"}
                  onClick={async () => {
                    const st = await filmcraftDisconnect();
                    setFcStatus(st);
                    setFcMsg({ text: "Disconnected from FilmCraft MCP.", isError: false });
                  }}
                  className="px-3 py-1.5 rounded-lg bg-neutral-800 hover:bg-neutral-700 disabled:opacity-40 text-neutral-200 font-semibold text-[11px] transition-colors"
                >
                  Disconnect
                </button>
              </div>
            </div>

            {/* Safety Mode */}
            <div className="p-3 rounded-xl bg-neutral-900 border border-neutral-800 space-y-2">
              <label className="text-[10px] uppercase font-bold text-neutral-400">
                Editing Safety Mode
              </label>
              <div className="grid grid-cols-3 gap-1.5">
                {(["Planning", "Supervised", "Autonomous"] as FilmCraftEditingMode[]).map((m) => (
                  <button
                    key={m}
                    onClick={async () => {
                      await filmcraftSetMode(m);
                      const st = await filmcraftGetStatus();
                      setFcStatus(st);
                    }}
                    className={`py-1.5 px-2 rounded-lg text-[10px] font-semibold border transition-colors ${
                      fcStatus?.mode === m
                        ? "bg-red-600 text-white border-red-500"
                        : "bg-neutral-950 text-neutral-400 border-neutral-800 hover:text-neutral-200"
                    }`}
                  >
                    {m}
                  </button>
                ))}
              </div>
              <div className="text-[10px] text-neutral-400">
                {fcStatus?.mode === "Planning" && "Analyse footage and plan only. Modifications blocked."}
                {fcStatus?.mode === "Supervised" && "Execute edits with safety checks and user oversight."}
                {fcStatus?.mode === "Autonomous" && "Full autonomous execution with checkpoints."}
              </div>
            </div>

            {/* Project Info & History */}
            <div className="p-3 rounded-xl bg-neutral-900 border border-neutral-800 space-y-1.5 text-[11px]">
              <div className="flex justify-between">
                <span className="text-neutral-400">Tools Discovered</span>
                <span className="text-neutral-200 font-mono">{fcStatus?.tools_count ?? 0}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-neutral-400">Steps Executed</span>
                <span className="text-neutral-200 font-mono">{fcStatus?.steps_taken ?? 0}</span>
              </div>
              {fcStatus?.project_name && (
                <div className="flex justify-between">
                  <span className="text-neutral-400">Project</span>
                  <span className="text-neutral-200 font-mono truncate max-w-[180px]">
                    {fcStatus.project_name}
                  </span>
                </div>
              )}
              {fcStatus?.project_path && (
                <div className="text-[10px] font-mono text-neutral-400 break-all">
                  {fcStatus.project_path}
                </div>
              )}

              {fcHistory.length > 0 && (
                <div className="pt-2 border-t border-neutral-800 space-y-1">
                  <div className="flex items-center justify-between">
                    <span className="text-[10px] uppercase font-bold text-neutral-400">
                      Tool Execution History
                    </span>
                    <button
                      onClick={async () => {
                        await filmcraftClearHistory();
                        setFcHistory([]);
                      }}
                      className="text-[10px] text-red-400 hover:text-red-300"
                    >
                      Clear
                    </button>
                  </div>
                  <div className="max-h-32 overflow-y-auto space-y-1">
                    {fcHistory.slice(-10).reverse().map((h) => (
                      <div
                        key={h.step}
                        className="flex items-center justify-between text-[10px] font-mono bg-black/30 px-2 py-1 rounded"
                      >
                        <span className={h.success ? "text-emerald-400" : "text-red-400"}>
                          #{h.step} {h.tool}
                        </span>
                        <span className="text-neutral-500">{h.success ? "OK" : "ERR"}</span>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </div>
        )}

        {activeTab === "audit" && (
          <div className="space-y-2">
            {logs.length === 0 ? (
              <div className="text-center py-8 text-neutral-400">No actions recorded yet.</div>
            ) : (
              logs.map((log) => (
                <div
                  key={log.id}
                  className="p-2.5 rounded-xl bg-neutral-900 border border-neutral-800 space-y-1 font-mono text-[10px]"
                >
                  <div className="flex justify-between text-neutral-400">
                    <span className="font-semibold text-red-400">{log.tool}</span>
                    <span>{new Date(log.timestamp).toLocaleTimeString()}</span>
                  </div>
                  <div className="flex gap-2">
                    <span
                      className={`px-1.5 py-0.2 rounded text-[9px] uppercase font-bold ${
                        log.risk === "destructive"
                          ? "bg-red-950/80 text-red-300 border border-red-800"
                          : log.risk === "write"
                          ? "bg-amber-950/80 text-amber-300 border border-amber-800"
                          : "bg-emerald-950/80 text-emerald-300 border border-emerald-800"
                      }`}
                    >
                      {log.risk}
                    </span>
                    <span className="text-neutral-400 truncate">
                      {log.decision}
                    </span>
                  </div>
                </div>
              ))
            )}
          </div>
        )}
      </div>
    </div>
  );
}
