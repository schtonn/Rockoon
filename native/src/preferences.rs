use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::PathBuf};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Preferences {
    pub instance_path: Option<PathBuf>,
    pub hide_on_launch: bool,
    pub map_only: bool,
    pub level_slot: u8,
    pub motd: String,
    pub language: String,
    pub enable_tas: bool,
    pub playtime: u64,
    pub last_played: Option<u64>,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            instance_path: None,
            hide_on_launch: false,
            map_only: true,
            level_slot: 0,
            motd: String::new(),
            language: "zh".into(),
            enable_tas: false,
            playtime: 0,
            last_played: None,
        }
    }
}

fn base_directory() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("ROCKOON_NATIVE_CONFIG_DIR") {
        return Ok(PathBuf::from(path));
    }
    ProjectDirs::from("dev", "schtonn", "RockoonNative")
        .map(|dirs| dirs.config_dir().to_owned())
        .ok_or_else(|| "无法找到用户配置目录".to_owned())
}
pub fn directory() -> Result<PathBuf, String> {
    let root = base_directory()?;
    Ok(if std::env::args().any(|a| a == "--tas-editor") {
        root.join("tas-editor")
    } else {
        root
    })
}
fn file_path() -> Result<PathBuf, String> {
    Ok(directory()?.join("preferences.json"))
}

pub fn load() -> Result<Preferences, String> {
    let path = file_path()?;
    match fs::read(path) {
        Ok(bytes) => {
            serde_json::from_slice(&bytes).map_err(|error| format!("无法读取启动器配置：{error}"))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Preferences::default()),
        Err(error) => Err(format!("无法读取启动器配置：{error}")),
    }
}

static SAVE_SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static SAVED: std::sync::Mutex<u64> = std::sync::Mutex::new(0);
pub fn next_save() -> u64 {
    SAVE_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1
}
pub fn save(preferences: &Preferences, sequence: u64) -> Result<(), String> {
    let mut saved = SAVED.lock().map_err(|e| e.to_string())?;
    if sequence < *saved {
        return Ok(());
    }
    let path = file_path()?;
    let parent = path.parent().ok_or("无法找到用户配置目录")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let mut pending = tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
    let bytes = serde_json::to_vec_pretty(preferences).map_err(|error| error.to_string())?;
    pending
        .write_all(&bytes)
        .map_err(|error| error.to_string())?;
    pending
        .as_file()
        .sync_all()
        .map_err(|error| error.to_string())?;
    pending.persist(path).map_err(|error| error.to_string())?;
    *saved = sequence;
    Ok(())
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Stats {
    pub total: u64,
    pub last: Option<u64>,
}
pub fn stats() -> Result<Stats, String> {
    let root = base_directory()?;
    if let Ok(bytes) = fs::read(root.join("stats.json")) {
        return serde_json::from_slice(&bytes).map_err(|e| e.to_string());
    }
    let old: Preferences = fs::read(root.join("preferences.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    Ok(Stats {
        total: old.playtime,
        last: old.last_played,
    })
}
pub fn record(seconds: u64, last: Option<u64>) -> Result<Stats, String> {
    use fs2::FileExt;
    let root = base_directory()?;
    fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(root.join("stats.lock"))
        .map_err(|e| e.to_string())?;
    lock.lock_exclusive().map_err(|e| e.to_string())?;
    let mut stats = stats()?;
    stats.total = stats.total.saturating_add(seconds);
    stats.last = stats.last.into_iter().chain(last).max();
    crate::workspace::atomic(
        &root.join("stats.json"),
        &serde_json::to_vec(&stats).map_err(|e| e.to_string())?,
    )?;
    Ok(stats)
}
