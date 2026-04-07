use std::collections::BTreeSet;
use std::fmt::{Display, Formatter};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::support::paths::AppPaths;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedRuntimeState {
    #[serde(default)]
    pub last_active_rule_ids: BTreeSet<String>,
}

#[derive(Clone)]
pub struct RuntimeStateStore {
    inner: Arc<RuntimeStateStoreInner>,
}

struct RuntimeStateStoreInner {
    paths: AppPaths,
    write_lock: Mutex<()>,
}

impl RuntimeStateStore {
    pub fn new(paths: AppPaths) -> Self {
        Self {
            inner: Arc::new(RuntimeStateStoreInner {
                paths,
                write_lock: Mutex::new(()),
            }),
        }
    }

    pub fn load(&self) -> Result<PersistedRuntimeState, RuntimeStateStoreError> {
        self.inner
            .paths
            .ensure_data_dir()
            .map_err(|err| RuntimeStateStoreError::Io("创建应用数据目录失败".to_string(), err))?;

        let runtime_state_path = self.inner.paths.runtime_state_file();
        if !runtime_state_path.exists() {
            let snapshot = PersistedRuntimeState::default();
            self.write_snapshot(&snapshot)?;
            return Ok(snapshot);
        }

        let contents = fs::read_to_string(runtime_state_path).map_err(|err| {
            RuntimeStateStoreError::Io(
                format!("读取运行态文件失败: {}", runtime_state_path.display()),
                err,
            )
        })?;

        if contents.trim().is_empty() {
            let snapshot = PersistedRuntimeState::default();
            self.write_snapshot(&snapshot)?;
            return Ok(snapshot);
        }

        toml::from_str(&contents).map_err(|err| {
            RuntimeStateStoreError::TomlDecode(
                format!("解析运行态文件失败: {}", runtime_state_path.display()),
                err,
            )
        })
    }

    pub fn save(&self, snapshot: &PersistedRuntimeState) -> Result<(), RuntimeStateStoreError> {
        self.write_snapshot(snapshot)
    }

    fn write_snapshot(
        &self,
        snapshot: &PersistedRuntimeState,
    ) -> Result<(), RuntimeStateStoreError> {
        let _guard = self
            .inner
            .write_lock
            .lock()
            .map_err(|_| RuntimeStateStoreError::LockPoisoned)?;

        self.inner
            .paths
            .ensure_data_dir()
            .map_err(|err| RuntimeStateStoreError::Io("创建应用数据目录失败".to_string(), err))?;

        let runtime_state_path = self.inner.paths.runtime_state_file();
        let temp_path = temporary_runtime_state_path(runtime_state_path);
        let contents = toml::to_string_pretty(snapshot).map_err(|err| {
            RuntimeStateStoreError::TomlEncode("序列化运行态失败".to_string(), err)
        })?;

        {
            let mut file = fs::File::create(&temp_path).map_err(|err| {
                RuntimeStateStoreError::Io(
                    format!("创建临时运行态文件失败: {}", temp_path.display()),
                    err,
                )
            })?;
            file.write_all(contents.as_bytes()).map_err(|err| {
                RuntimeStateStoreError::Io(
                    format!("写入临时运行态文件失败: {}", temp_path.display()),
                    err,
                )
            })?;
            file.sync_all().map_err(|err| {
                RuntimeStateStoreError::Io(
                    format!("刷新临时运行态文件失败: {}", temp_path.display()),
                    err,
                )
            })?;
        }

        match fs::rename(&temp_path, runtime_state_path) {
            Ok(()) => Ok(()),
            Err(rename_err) => {
                if runtime_state_path.exists() {
                    fs::remove_file(runtime_state_path).map_err(|remove_err| {
                        RuntimeStateStoreError::Io(
                            format!("替换旧运行态文件失败: {}", runtime_state_path.display()),
                            remove_err,
                        )
                    })?;

                    fs::rename(&temp_path, runtime_state_path).map_err(|retry_err| {
                        RuntimeStateStoreError::Io(
                            format!(
                                "重命名临时运行态文件失败: {} -> {} (初次错误: {})",
                                temp_path.display(),
                                runtime_state_path.display(),
                                rename_err
                            ),
                            retry_err,
                        )
                    })?;
                    Ok(())
                } else {
                    Err(RuntimeStateStoreError::Io(
                        format!(
                            "重命名临时运行态文件失败: {} -> {}",
                            temp_path.display(),
                            runtime_state_path.display()
                        ),
                        rename_err,
                    ))
                }
            }
        }
    }
}

fn temporary_runtime_state_path(runtime_state_path: &Path) -> PathBuf {
    let file_name = runtime_state_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("runtime-state.toml");
    runtime_state_path.with_file_name(format!("{file_name}.tmp"))
}

#[derive(Debug)]
pub enum RuntimeStateStoreError {
    Io(String, std::io::Error),
    TomlDecode(String, toml::de::Error),
    TomlEncode(String, toml::ser::Error),
    LockPoisoned,
}

impl Display for RuntimeStateStoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(message, err) => write!(f, "{message}: {err}"),
            Self::TomlDecode(message, err) => write!(f, "{message}: {err}"),
            Self::TomlEncode(message, err) => write!(f, "{message}: {err}"),
            Self::LockPoisoned => write!(f, "运行态存储写锁已损坏"),
        }
    }
}

impl std::error::Error for RuntimeStateStoreError {}
