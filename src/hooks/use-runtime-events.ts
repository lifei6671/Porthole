import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import {
  getRuntimeStatus,
  logsAppendedEvent,
  processExitedEvent,
  runtimeChangedEvent,
  startAllEnabledRules,
  stopAllRules,
  type LogsAppendedPayload,
  type ProcessExitedPayload,
  type RuntimeChangedPayload,
} from "../lib/api";
import type { ProcessLogEntry, RuntimeState } from "../lib/types";

const maxLogEntries = 500;

export function useRuntimeEvents() {
  const [runtime, setRuntime] = useState<RuntimeState | null>(null);
  const [logs, setLogs] = useState<ProcessLogEntry[]>([]);
  const [processExitReason, setProcessExitReason] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  async function refresh() {
    const snapshot = await getRuntimeStatus();
    setRuntime(snapshot.runtime);
    setLogs(snapshot.logs);
    setError(null);
    return snapshot;
  }

  async function startAll() {
    await startAllEnabledRules();
    await refresh();
  }

  async function stopAll() {
    await stopAllRules();
    await refresh();
  }

  useEffect(() => {
    let disposed = false;
    const unlisteners: Array<() => void> = [];

    async function bootstrap() {
      try {
        const [snapshot, runtimeUnlisten, logsUnlisten, exitUnlisten] =
          await Promise.all([
            refresh(),
            listen<RuntimeChangedPayload>(runtimeChangedEvent, (event) => {
              setRuntime(event.payload.runtime);
            }),
            listen<LogsAppendedPayload>(logsAppendedEvent, (event) => {
              setLogs((current) => {
                const merged = current.concat(event.payload.entries);
                return merged.slice(-maxLogEntries);
              });
            }),
            listen<ProcessExitedPayload>(processExitedEvent, (event) => {
              setRuntime(event.payload.runtime);
              setProcessExitReason(event.payload.reason);
            }),
          ]);

        unlisteners.push(runtimeUnlisten, logsUnlisten, exitUnlisten);

        if (!disposed) {
          setRuntime(snapshot.runtime);
          setLogs(snapshot.logs);
          setProcessExitReason(null);
          setError(null);
        }
      } catch (err) {
        if (!disposed) {
          setError(err instanceof Error ? err.message : String(err));
        }
      } finally {
        if (!disposed) {
          setLoading(false);
        }
      }
    }

    bootstrap();

    return () => {
      disposed = true;
      for (const unlisten of unlisteners) {
        unlisten();
      }
    };
  }, []);

  return {
    runtime,
    logs,
    processExitReason,
    loading,
    error,
    refresh,
    startAll,
    stopAll,
    clearProcessExitReason() {
      setProcessExitReason(null);
    },
    clearLocalLogs() {
      setLogs([]);
    },
  };
}
