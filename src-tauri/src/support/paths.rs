use std::io;
use std::path::{Path, PathBuf};

use tauri::Manager;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    data_dir: PathBuf,
    rules_file: PathBuf,
    runtime_state_file: PathBuf,
    gost_config_file: PathBuf,
    gost_pid_file: PathBuf,
    sidecar_dir: PathBuf,
    gost_runtime_executable: PathBuf,
}

impl AppPaths {
    pub fn new(data_dir: impl Into<PathBuf>) -> Self {
        let data_dir = data_dir.into();
        let sidecar_dir = data_dir.join("sidecar");
        Self {
            rules_file: data_dir.join("rules.toml"),
            runtime_state_file: data_dir.join("runtime-state.toml"),
            gost_config_file: data_dir.join("gost.yaml"),
            gost_pid_file: data_dir.join("gost.pid"),
            gost_runtime_executable: sidecar_dir.join("gost.exe"),
            sidecar_dir,
            data_dir,
        }
    }

    pub fn from_app_handle(app_handle: &tauri::AppHandle) -> io::Result<Self> {
        let data_dir = app_handle.path().app_data_dir().map_err(|err| {
            io::Error::new(io::ErrorKind::Other, format!("获取应用数据目录失败: {err}"))
        })?;
        Ok(Self::new(data_dir))
    }

    pub fn ensure_data_dir(&self) -> io::Result<()> {
        std::fs::create_dir_all(&self.data_dir)
    }

    pub fn ensure_sidecar_dir(&self) -> io::Result<()> {
        std::fs::create_dir_all(&self.sidecar_dir)
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn rules_file(&self) -> &Path {
        &self.rules_file
    }

    pub fn gost_config_file(&self) -> &Path {
        &self.gost_config_file
    }

    pub fn runtime_state_file(&self) -> &Path {
        &self.runtime_state_file
    }

    pub fn gost_pid_file(&self) -> &Path {
        &self.gost_pid_file
    }

    pub fn sidecar_dir(&self) -> &Path {
        &self.sidecar_dir
    }

    pub fn gost_runtime_executable(&self) -> &Path {
        &self.gost_runtime_executable
    }
}
