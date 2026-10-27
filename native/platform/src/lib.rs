//! Platform adapters for the existing Win32/Wine algorithms. No Tauri dependency.
pub use rockoon_core::common;
pub mod ballance {
    pub use rockoon_core::input_fix;
}
pub mod commands {
    pub use crate::wine;
}
pub mod embed {
    include!(concat!(env!("OUT_DIR"), "/embed.rs"));
}
pub mod frame_limit {
    include!(concat!(env!("OUT_DIR"), "/frame_limit.rs"));
}
pub mod wine {
    include!(concat!(env!("OUT_DIR"), "/wine.rs"));
}
pub mod process {
    include!(concat!(env!("OUT_DIR"), "/process.rs"));
}

use common::exception::{RcError, RcResult, RcResultWith};
use std::{path::PathBuf, sync::Arc};
pub type ProgressCallback = dyn Fn(&str) + Send + Sync;
#[derive(Clone, Default)]
pub struct Context {
    pub progress: Option<Arc<ProgressCallback>>,
}
pub enum BaseDirectory {
    Resource,
}
impl Context {
    pub fn path(&self) -> &Self {
        self
    }
    pub fn app_data_dir(&self) -> RcResultWith<PathBuf> {
        directories::ProjectDirs::from("dev", "schtonn", "RockoonNative")
            .map(|d| d.data_local_dir().to_owned())
            .ok_or_else(|| RcError::Other("No user data directory".into()))
    }
    pub fn app_log_dir(&self) -> RcResultWith<PathBuf> {
        Ok(self.app_data_dir()?.join("logs"))
    }
    pub fn resolve(&self, path: &str, _base: BaseDirectory) -> RcResultWith<PathBuf> {
        let executable = std::env::current_exe()?;
        let parent = executable
            .parent()
            .ok_or_else(|| RcError::Other("Executable has no parent".into()))?;
        let bundled = parent.parent().map(|p| p.join("Resources").join(path));
        if let Some(bundled) = bundled.filter(|p| p.is_file()) {
            return Ok(bundled);
        }
        Ok(parent.join(path))
    }
    pub fn emit(&self, _event: &str, phase: &str) -> RcResult {
        if let Some(callback) = &self.progress {
            callback(phase);
        }
        Ok(())
    }
}
#[cfg(not(windows))]
pub struct UnixGame {
    pid: u32,
}
#[cfg(not(windows))]
impl UnixGame {
    pub fn new(pid: u32) -> Self {
        Self { pid }
    }
    pub fn id(&self) -> u32 {
        self.pid
    }
    pub fn try_wait(&mut self) -> std::io::Result<Option<std::process::ExitStatus>> {
        use std::os::unix::process::ExitStatusExt;
        process::check(self.pid)
            .map(|running| {
                if running {
                    None
                } else {
                    Some(std::process::ExitStatus::from_raw(0))
                }
            })
            .map_err(|e| std::io::Error::other(e.to_string()))
    }
    pub fn kill(&mut self) -> std::io::Result<()> {
        process::kill(self.pid).map_err(|e| std::io::Error::other(e.to_string()))
    }
    pub fn wait(&mut self) -> std::io::Result<std::process::ExitStatus> {
        loop {
            if let Some(status) = self.try_wait()? {
                return Ok(status);
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}
