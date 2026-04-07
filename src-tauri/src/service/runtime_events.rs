use std::sync::Arc;
use std::thread;
use std::time::Duration;

use serde::Serialize;

use crate::model::rule::Rule;
use crate::model::runtime::{ProcessStatus, RuntimeState};
use crate::service::gost_process::{GostProcessManager, ProcessLogEntry};

pub const EVENT_RUNTIME_CHANGED: &str = "runtime://changed";
pub const EVENT_RULES_CHANGED: &str = "rules://changed";
pub const EVENT_LOGS_APPENDED: &str = "logs://appended";
pub const EVENT_PROCESS_EXITED: &str = "runtime://process-exited";

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RuntimeChangedPayload {
    pub runtime: RuntimeState,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RulesChangedPayload {
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LogsAppendedPayload {
    pub entries: Vec<ProcessLogEntry>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProcessExitedPayload {
    pub runtime: RuntimeState,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeEventMessage {
    RuntimeChanged(RuntimeChangedPayload),
    RulesChanged(RulesChangedPayload),
    LogsAppended(LogsAppendedPayload),
    ProcessExited(ProcessExitedPayload),
}

type EventSink = dyn Fn(RuntimeEventMessage) + Send + Sync + 'static;

#[derive(Clone)]
pub struct RuntimeEventEmitter {
    sink: Arc<EventSink>,
}

impl RuntimeEventEmitter {
    pub fn new(sink: Arc<EventSink>) -> Self {
        Self { sink }
    }

    pub fn emit_runtime_changed(&self, runtime: RuntimeState) {
        (self.sink)(RuntimeEventMessage::RuntimeChanged(RuntimeChangedPayload {
            runtime,
        }));
    }

    pub fn emit_rules_changed(&self, rules: Vec<Rule>) {
        (self.sink)(RuntimeEventMessage::RulesChanged(RulesChangedPayload {
            rules,
        }));
    }

    pub fn emit_logs_appended(&self, entries: Vec<ProcessLogEntry>) {
        if entries.is_empty() {
            return;
        }
        (self.sink)(RuntimeEventMessage::LogsAppended(LogsAppendedPayload {
            entries,
        }));
    }

    pub fn emit_process_exited(&self, runtime: RuntimeState, reason: Option<String>) {
        (self.sink)(RuntimeEventMessage::ProcessExited(ProcessExitedPayload {
            runtime,
            reason,
        }));
    }
}

pub fn spawn_runtime_event_bridge(
    manager: GostProcessManager,
    emitter: RuntimeEventEmitter,
    poll_interval: Duration,
) {
    thread::spawn(move || {
        let mut last_runtime = manager.runtime_snapshot();
        let mut last_log_len = manager.log_snapshot().len();

        loop {
            let runtime = manager.runtime_snapshot();
            if runtime != last_runtime {
                let previous_status = last_runtime.process_status;
                let current_status = runtime.process_status;
                emitter.emit_runtime_changed(runtime.clone());

                let was_active = matches!(
                    previous_status,
                    ProcessStatus::Starting | ProcessStatus::Running | ProcessStatus::Stopping
                );
                let is_terminal = matches!(
                    current_status,
                    ProcessStatus::Stopped | ProcessStatus::Failed
                );
                if was_active && is_terminal {
                    emitter.emit_process_exited(
                        runtime.clone(),
                        runtime
                            .last_error
                            .as_ref()
                            .map(|error| error.summary.clone()),
                    );
                }

                last_runtime = runtime;
            }

            let logs = manager.log_snapshot();
            if logs.len() > last_log_len {
                emitter.emit_logs_appended(logs[last_log_len..].to_vec());
                last_log_len = logs.len();
            } else if logs.len() < last_log_len {
                last_log_len = logs.len();
            }

            thread::sleep(poll_interval);
        }
    });
}
