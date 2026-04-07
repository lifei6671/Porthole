export type Protocol = "tcp" | "udp";

export interface Rule {
  id: string;
  name: string;
  enabled: boolean;
  protocol: Protocol;
  listen_host: string;
  listen_port: number;
  target_host: string;
  target_port: number;
  remark: string;
  created_at: string;
  updated_at: string;
}

export interface RuleInput {
  name: string;
  enabled: boolean;
  protocol: Protocol;
  listen_host: string;
  listen_port: number;
  target_host: string;
  target_port: number;
  remark: string;
}

export type ProcessStatus =
  | "stopped"
  | "starting"
  | "running"
  | "stopping"
  | "failed";

export type RuleProcessStatus = "stopped" | "starting" | "running" | "failed";

export interface RuleRuntimeStatus {
  rule_id: string;
  status: RuleProcessStatus;
  last_error_summary: string | null;
}

export interface RuntimeErrorSummary {
  summary: string;
  observed_at: string;
}

export interface RuntimeState {
  process_status: ProcessStatus;
  rule_statuses: RuleRuntimeStatus[];
  last_error: RuntimeErrorSummary | null;
  active_rule_ids: string[];
}

export type ProcessLogSource =
  | "Stdout"
  | "Stderr"
  | "AppInfo"
  | "AppError";

export interface ProcessLogEntry {
  source: ProcessLogSource;
  message: string;
  observed_at: string;
}

export interface RuntimeStatusPayload {
  runtime: RuntimeState;
  logs: ProcessLogEntry[];
}

export interface RuntimeChangedPayload {
  runtime: RuntimeState;
}

export interface RulesChangedPayload {
  rules: Rule[];
}

export interface LogsAppendedPayload {
  entries: ProcessLogEntry[];
}

export interface ProcessExitedPayload {
  runtime: RuntimeState;
  reason: string | null;
}

export type UILogSource = "app" | "gost";
export type UILogLevel = "info" | "error";

export interface UILogEntry {
  id: string;
  source: UILogSource;
  level: UILogLevel;
  message: string;
  observedAt: string;
}
