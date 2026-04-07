import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { listRules, rulesChangedEvent, type RulesChangedPayload } from "../lib/api";
import type { Rule } from "../lib/types";

export function useRules() {
  const [rules, setRules] = useState<Rule[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  async function refresh() {
    const nextRules = await listRules();
    setRules(nextRules);
    setError(null);
    return nextRules;
  }

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;

    async function bootstrap() {
      try {
        const [initialRules, detach] = await Promise.all([
          refresh(),
          listen<RulesChangedPayload>(rulesChangedEvent, (event) => {
            setRules(event.payload.rules);
          }),
        ]);

        unlisten = detach;
        if (!disposed) {
          setRules(initialRules);
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
      unlisten?.();
    };
  }, []);

  return {
    rules,
    loading,
    error,
    refresh,
  };
}
