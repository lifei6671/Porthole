import { invoke } from "@tauri-apps/api/core";
import type {
  LogsAppendedPayload,
  ProcessExitedPayload,
  Rule,
  RulesChangedPayload,
  RuntimeChangedPayload,
  RuntimeStatusPayload,
} from "./types";

export const runtimeChangedEvent = "runtime://changed";
export const rulesChangedEvent = "rules://changed";
export const logsAppendedEvent = "logs://appended";
export const processExitedEvent = "runtime://process-exited";

export async function listRules(): Promise<Rule[]> {
  return invoke<Rule[]>("list_rules");
}

export async function getRuntimeStatus(): Promise<RuntimeStatusPayload> {
  return invoke<RuntimeStatusPayload>("get_runtime_status");
}

export async function clearLogs(): Promise<void> {
  await invoke("clear_logs");
}

export type {
  LogsAppendedPayload,
  ProcessExitedPayload,
  RulesChangedPayload,
  RuntimeChangedPayload,
};
