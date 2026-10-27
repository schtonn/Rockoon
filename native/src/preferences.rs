use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{fs, io::Write, path::PathBuf};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preferences {
    pub instance_path: Option<PathBuf>,
}

fn file_path() -> Result<PathBuf, String> {
    ProjectDirs::from("dev", "schtonn", "RockoonNative")
        .map(|dirs| dirs.config_dir().join("preferences.json"))
        .ok_or_else(|| "无法找到用户配置目录".to_owned())
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

pub fn save(preferences: &Preferences) -> Result<(), String> {
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
    Ok(())
}
