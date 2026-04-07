use std::fmt::{Display, Formatter};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::model::rule::RuleSet;
use crate::support::paths::AppPaths;

#[derive(Clone)]
pub struct RuleStore {
    inner: Arc<RuleStoreInner>,
}

struct RuleStoreInner {
    paths: AppPaths,
    write_lock: Mutex<()>,
}

impl RuleStore {
    pub fn new(paths: AppPaths) -> Self {
        Self {
            inner: Arc::new(RuleStoreInner {
                paths,
                write_lock: Mutex::new(()),
            }),
        }
    }

    pub fn paths(&self) -> &AppPaths {
        &self.inner.paths
    }

    pub fn ensure_initialized(&self) -> Result<RuleSet, RuleStoreError> {
        self.load()
    }

    pub fn load(&self) -> Result<RuleSet, RuleStoreError> {
        self.inner
            .paths
            .ensure_data_dir()
            .map_err(|err| RuleStoreError::Io("创建应用数据目录失败".to_string(), err))?;

        let rules_path = self.inner.paths.rules_file();
        if !rules_path.exists() {
            let snapshot = RuleSet::default();
            self.write_snapshot(&snapshot)?;
            return Ok(snapshot);
        }

        let contents = fs::read_to_string(rules_path).map_err(|err| {
            RuleStoreError::Io(format!("读取规则文件失败: {}", rules_path.display()), err)
        })?;

        if contents.trim().is_empty() {
            let snapshot = RuleSet::default();
            self.write_snapshot(&snapshot)?;
            return Ok(snapshot);
        }

        toml::from_str(&contents).map_err(|err| {
            RuleStoreError::TomlDecode(format!("解析规则文件失败: {}", rules_path.display()), err)
        })
    }

    pub fn save(&self, snapshot: &RuleSet) -> Result<(), RuleStoreError> {
        self.write_snapshot(snapshot)
    }

    fn write_snapshot(&self, snapshot: &RuleSet) -> Result<(), RuleStoreError> {
        let _guard = self
            .inner
            .write_lock
            .lock()
            .map_err(|_| RuleStoreError::LockPoisoned)?;

        self.inner
            .paths
            .ensure_data_dir()
            .map_err(|err| RuleStoreError::Io("创建应用数据目录失败".to_string(), err))?;

        let rules_path = self.inner.paths.rules_file();
        let temp_path = temporary_rules_path(rules_path);
        let contents = toml::to_string_pretty(snapshot)
            .map_err(|err| RuleStoreError::TomlEncode("序列化规则配置失败".to_string(), err))?;

        {
            let mut file = fs::File::create(&temp_path).map_err(|err| {
                RuleStoreError::Io(
                    format!("创建临时规则文件失败: {}", temp_path.display()),
                    err,
                )
            })?;
            file.write_all(contents.as_bytes()).map_err(|err| {
                RuleStoreError::Io(
                    format!("写入临时规则文件失败: {}", temp_path.display()),
                    err,
                )
            })?;
            file.sync_all().map_err(|err| {
                RuleStoreError::Io(
                    format!("刷新临时规则文件失败: {}", temp_path.display()),
                    err,
                )
            })?;
        }

        match fs::rename(&temp_path, rules_path) {
            Ok(()) => Ok(()),
            Err(rename_err) => {
                if rules_path.exists() {
                    fs::remove_file(rules_path).map_err(|remove_err| {
                        RuleStoreError::Io(
                            format!("替换旧规则文件失败: {}", rules_path.display()),
                            remove_err,
                        )
                    })?;

                    fs::rename(&temp_path, rules_path).map_err(|retry_err| {
                        RuleStoreError::Io(
                            format!(
                                "重命名临时规则文件失败: {} -> {} (初次错误: {})",
                                temp_path.display(),
                                rules_path.display(),
                                rename_err
                            ),
                            retry_err,
                        )
                    })?;
                    Ok(())
                } else {
                    Err(RuleStoreError::Io(
                        format!(
                            "重命名临时规则文件失败: {} -> {}",
                            temp_path.display(),
                            rules_path.display()
                        ),
                        rename_err,
                    ))
                }
            }
        }
    }
}

fn temporary_rules_path(rules_path: &Path) -> PathBuf {
    let file_name = rules_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("rules.toml");
    rules_path.with_file_name(format!("{file_name}.tmp"))
}

#[derive(Debug)]
pub enum RuleStoreError {
    Io(String, std::io::Error),
    TomlDecode(String, toml::de::Error),
    TomlEncode(String, toml::ser::Error),
    LockPoisoned,
}

impl Display for RuleStoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(message, err) => write!(f, "{message}: {err}"),
            Self::TomlDecode(message, err) => write!(f, "{message}: {err}"),
            Self::TomlEncode(message, err) => write!(f, "{message}: {err}"),
            Self::LockPoisoned => write!(f, "规则存储写锁已损坏"),
        }
    }
}

impl std::error::Error for RuleStoreError {}
