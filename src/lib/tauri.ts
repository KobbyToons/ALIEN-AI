/**
 * Thin wrappers around Tauri IPC for type safety.
 */
import { invoke } from "@tauri-apps/api/core";
import type {
  AuditEntry,
  ChatMessageInput,
  FileEntry,
  GoogleAccountStatus,
  LearningConfig,
  LoadedSkill,
  MemoryNote,
  ModelProvider,
  PermissionDecision,
  ProcessInfo,
  ScenarioQuestion,
  ShellOutput,
  ThinkingProfile,
  VaultInfo,
} from "../types";

export async function greet(name: string): Promise<string> {
  return invoke<string>("greet", { name });
}

export async function shellExec(
  command: string,
  args?: string[],
  cwd?: string,
): Promise<ShellOutput> {
  return invoke<ShellOutput>("shell_exec", { command, args, cwd });
}

export async function fsRead(path: string): Promise<string> {
  return invoke<string>("fs_read", { path });
}

export async function fsWrite(path: string, content: string): Promise<void> {
  return invoke<void>("fs_write", { path, content });
}

export async function fsList(path: string): Promise<FileEntry[]> {
  return invoke<FileEntry[]>("fs_list", { path });
}

export async function fsRemove(
  path: string,
  recursive = false,
): Promise<void> {
  return invoke<void>("fs_remove", { path, recursive });
}

export async function fsCopy(src: string, dest: string): Promise<void> {
  return invoke<void>("fs_copy", { src, dest });
}

export async function fsMove(src: string, dest: string): Promise<void> {
  return invoke<void>("fs_move", { src, dest });
}

export async function processList(): Promise<ProcessInfo[]> {
  return invoke<ProcessInfo[]>("process_list");
}

export async function processKill(pid: number): Promise<void> {
  return invoke<void>("process_kill", { pid });
}

export async function clipboardRead(): Promise<string> {
  return invoke<string>("clipboard_read");
}

export async function clipboardWrite(text: string): Promise<void> {
  return invoke<void>("clipboard_write", { text });
}

export async function openUrl(url: string): Promise<void> {
  return invoke<void>("open_url", { url });
}

export async function respondPermission(
  id: string,
  decision: PermissionDecision,
): Promise<void> {
  return invoke<void>("respond_permission", { id, decision });
}

export async function toggleKillSwitch(enabled: boolean): Promise<boolean> {
  return invoke<boolean>("toggle_kill_switch", { enabled });
}

export async function getKillSwitch(): Promise<boolean> {
  return invoke<boolean>("get_kill_switch");
}

export async function getAuditLog(limit?: number): Promise<AuditEntry[]> {
  return invoke<AuditEntry[]>("get_audit_log", { limit });
}

export async function resizeWindow(width: number, height: number): Promise<void> {
  return invoke<void>("resize_window", { width, height });
}

// ── AI and Credentials ──────────────────────────────────────────

export async function saveApiKey(
  provider: ModelProvider,
  apiKey: string,
): Promise<void> {
  return invoke<void>("save_api_key", { provider, apiKey });
}

export async function hasApiKey(provider: ModelProvider): Promise<boolean> {
  return invoke<boolean>("has_api_key", { provider });
}

export async function deleteApiKey(provider: ModelProvider): Promise<void> {
  return invoke<void>("delete_api_key", { provider });
}

export async function getGoogleAccountStatus(): Promise<GoogleAccountStatus> {
  return invoke<GoogleAccountStatus>("get_google_account_status");
}

export async function connectGoogleAccount(): Promise<GoogleAccountStatus> {
  return invoke<GoogleAccountStatus>("connect_google_account");
}

export async function modelStreamChat(
  requestId: string,
  provider: ModelProvider,
  model: string,
  messages: ChatMessageInput[],
  customEndpoint?: string,
): Promise<string> {
  return invoke<string>("model_stream_chat", {
    requestId,
    provider,
    model,
    messages,
    customEndpoint,
  });
}

// ── Autonomous Agent Task ───────────────────────────────────────

export async function runAgentTask(
  taskId: string,
  provider: ModelProvider,
  model: string,
  prompt: string,
  customEndpoint?: string,
): Promise<string> {
  return invoke<string>("run_agent_task", {
    taskId,
    provider,
    model,
    prompt,
    customEndpoint,
  });
}

// ── Obsidian Memory Vault ───────────────────────────────────────

export async function setVaultPath(path: string): Promise<VaultInfo> {
  return invoke<VaultInfo>("set_vault_path", { path });
}

export async function getVaultInfo(): Promise<VaultInfo> {
  return invoke<VaultInfo>("get_vault_info");
}

export async function searchVault(query: string): Promise<MemoryNote[]> {
  return invoke<MemoryNote[]>("search_vault", { query });
}

export async function createVaultTestNote(): Promise<string> {
  return invoke<string>("create_vault_test_note");
}

export async function rewriteVaultNote(
  category: string,
  title: string,
  content: string,
): Promise<string> {
  return invoke<string>("rewrite_vault_note", { category, title, content });
}

export async function listVaultNotes(): Promise<MemoryNote[]> {
  return invoke<MemoryNote[]>("list_vault_notes");
}

export async function openVaultInExplorer(): Promise<void> {
  return invoke<void>("open_vault_in_explorer");
}

export async function getSystemIdentity(): Promise<string> {
  return invoke<string>("get_system_identity");
}

export async function saveSystemIdentity(content: string): Promise<void> {
  return invoke<void>("save_system_identity", { content });
}

export async function resetSystemIdentity(): Promise<string> {
  return invoke<string>("reset_system_identity");
}

// ── Learning & User Profiling ───────────────────────────────────

export async function getLearningConfig(): Promise<LearningConfig> {
  return invoke<LearningConfig>("get_learning_config");
}

export async function updateLearningConfig(config: LearningConfig): Promise<void> {
  return invoke<void>("update_learning_config", { config });
}

export async function getNextQuestion(): Promise<ScenarioQuestion | null> {
  return invoke<ScenarioQuestion | null>("get_next_question");
}

export async function answerQuestion(
  questionId: string,
  answer: string,
): Promise<void> {
  return invoke<void>("answer_question", { questionId, answer });
}

export async function getThinkingProfile(): Promise<ThinkingProfile> {
  return invoke<ThinkingProfile>("get_thinking_profile");
}

// ── Skills Plug-in System ───────────────────────────────────────

export async function listSkills(): Promise<LoadedSkill[]> {
  return invoke<LoadedSkill[]>("list_skills");
}

export async function reloadSkills(): Promise<LoadedSkill[]> {
  return invoke<LoadedSkill[]>("reload_skills");
}
