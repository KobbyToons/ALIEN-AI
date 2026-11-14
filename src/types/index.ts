// ── Shared types ─────────────────────────────────────────────────────

/** Visual state of the Buddy orb */
export type BuddyState =
  | "idle"
  | "listening"
  | "thinking"
  | "speaking"
  | "working";

/** Supported LLM Providers */
export type ModelProvider =
  | "gemini"
  | "openai"
  | "anthropic"
  | "deepseek"
  | "lmstudio"
  | "openrouter"
  | "ollama";

export interface ChatAttachment {
  type: "image" | "file" | "folder";
  name: string;
  path: string;
  previewUrl?: string;
  size?: number;
}

/** A single chat message */
export interface ChatMessage {
  id: string;
  role: "user" | "assistant" | "system";
  content: string;
  timestamp: number;
  isStreaming?: boolean;
  attachments?: ChatAttachment[];
}

export interface ChatMessageInput {
  role: string;
  content: string;
}

export interface AgentStepEvent {
  taskId: string;
  step: number;
  status: "thinking" | "executing" | "done" | "error";
  description: string;
}

export interface VoiceConfig {
  autoSpeak: boolean;
  voiceName: string;
  rate: number;
  pitch: number;
}

export interface VaultInfo {
  path: string;
  isConnected: boolean;
  noteCount: number;
  categories: { name: string; count: number }[];
}

export interface MemoryNote {
  category: string;
  title: string;
  content: string;
  path: string;
}

export interface LearningConfig {
  passive_learning: boolean;
  scenario_questions: boolean;
  last_question_time?: number;
}

export interface ScenarioQuestion {
  id: string;
  category: string;
  question: string;
  options: string[];
}

export interface ThinkingProfile {
  summary: string;
  last_updated: string;
}

export interface SkillToolDefinition {
  name: string;
  description: string;
  risk?: string;
  input_schema: unknown;
  handler: string;
}

export interface SkillManifest {
  id: string;
  name: string;
  description: string;
  version: string;
  tools: SkillToolDefinition[];
}

export interface LoadedSkill {
  manifest: SkillManifest;
  directory: string;
}

/** Risk level for the permission gate */
export type RiskLevel = "read_only" | "write" | "destructive";

/** Permission response decisions */
export type PermissionDecision =
  | "auto_approved"
  | "approved"
  | "always_allow"
  | "denied"
  | "kill_switch_blocked";

/** A pending permission request shown as an approval card */
export interface PermissionRequest {
  id: string;
  tool: string;
  description: string;
  risk: RiskLevel;
  args?: Record<string, unknown>;
}

export interface AuditEntry {
  timestamp: string;
  id: string;
  tool: string;
  args: unknown;
  risk: RiskLevel;
  decision: PermissionDecision;
  success: boolean;
  error?: string;
}

export interface ShellOutput {
  stdout: string;
  stderr: string;
  exitCode: number;
}

export interface FileEntry {
  name: string;
  path: string;
  isDir: boolean;
  size: number;
  modified?: number;
}

export interface ProcessInfo {
  pid: number;
  name: string;
  memoryBytes: number;
  cpuPercent: number;
}

export interface GoogleAccountStatus {
  is_connected: boolean;
  email?: string;
  method: string;
}

export type WidgetMode = "orb" | "chat" | "full";

