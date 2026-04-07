import { invoke } from "@tauri-apps/api/core";
import type {
  LogsAppendedPayload,
  ProcessExitedPayload,
  Rule,
  RuleInput,
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

export async function createRule(input: RuleInput): Promise<Rule> {
  return invoke<Rule>("create_rule", { input });
}

export async function updateRule(ruleID: string, input: RuleInput): Promise<Rule> {
  return invoke<Rule>("update_rule", { ruleId: ruleID, input });
}

export async function deleteRule(ruleID: string): Promise<void> {
  await invoke("delete_rule", { ruleId: ruleID });
}

export async function getRuntimeStatus(): Promise<RuntimeStatusPayload> {
  return invoke<RuntimeStatusPayload>("get_runtime_status");
}

export async function startRule(ruleID: string) {
  return invoke("start_rule", { ruleId: ruleID });
}

export async function stopRule(ruleID: string) {
  return invoke("stop_rule", { ruleId: ruleID });
}

export async function startAllEnabledRules() {
  return invoke("start_all_enabled_rules");
}

export async function stopAllRules() {
  return invoke("stop_all_rules");
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
