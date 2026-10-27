use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command},
    sync::{Arc, Mutex},
};

pub type RunningGame = Arc<Mutex<Child>>;

#[derive(Clone, Debug)]
pub struct Resource {
    pub path: PathBuf,
    pub name: String,
    pub disabled: bool,
}

#[derive(Clone, Debug)]
pub struct Instance {
    pub root: PathBuf,
    pub name: String,
    pub player: String,
    pub loader: String,
    pub maps: Vec<Resource>,
    pub mods: Vec<Resource>,
}

pub fn discover(root: PathBuf) -> Result<Instance, String> {
    let root = std::path::absolute(root).map_err(error)?;
    for file in ["Bin/Player.exe", "base.cmo", "Database.tdb"] {
        if !root.join(file).is_file() {
            return Err(format!("请选择 Ballance 游戏根目录，缺少 {file}"));
        }
    }
    for directory in ["BuildingBlocks", "3D Entities"] {
        if !root.join(directory).is_dir() {
            return Err(format!("请选择 Ballance 游戏根目录，缺少 {directory} 目录"));
        }
    }
    let player_size = fs::metadata(root.join("Bin/Player.exe"))
        .map_err(error)?
        .len();
    let loader = if root.join("BuildingBlocks/BMLPlus.dll").is_file() {
        "BML+"
    } else if root.join("BuildingBlocks/BML.dll").is_file() {
        "BML"
    } else {
        "原版"
    };
    Ok(Instance {
        name: root
            .file_name()
            .unwrap_or(root.as_os_str())
            .to_string_lossy()
            .into_owned(),
        player: if player_size == 155_648 {
            "原版 Player"
        } else {
            "自定义 Player"
        }
        .to_owned(),
        loader: loader.to_owned(),
        maps: list_resources(&root.join("ModLoader/Maps"), &["nmo", "cmo"], false)?,
        mods: list_resources(
            &root.join("ModLoader/Mods"),
            &["bmod", "bmodp", "zip"],
            true,
        )?,
        root,
    })
}

fn error(error: impl std::fmt::Display) -> String {
    error.to_string()
}

fn list_resources(
    root: &Path,
    extensions: &[&str],
    include_disabled: bool,
) -> Result<Vec<Resource>, String> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut pending = vec![root.to_path_buf()];
    let mut resources = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory).map_err(error)? {
            let entry = entry.map_err(error)?;
            let kind = entry.file_type().map_err(error)?;
            if kind.is_symlink() {
                continue;
            }
            if kind.is_dir() {
                pending.push(entry.path());
            } else if kind.is_file() {
                let path = entry.path();
                let name = path
                    .strip_prefix(root)
                    .map_err(error)?
                    .to_string_lossy()
                    .into_owned();
                let lower = name.to_lowercase();
                let (active_name, disabled) = match lower.strip_suffix(".disable") {
                    Some(active) => (active, true),
                    None => (lower.as_str(), false),
                };
                if (!disabled || include_disabled)
                    && extensions
                        .iter()
                        .any(|ext| active_name.ends_with(&format!(".{ext}")))
                {
                    resources.push(Resource {
                        path,
                        name,
                        disabled,
                    });
                }
            }
        }
    }
    resources.sort_by_cached_key(|entry| entry.name.to_lowercase());
    Ok(resources)
}

/// Matches the existing launcher: Player.exe runs with Bin as its working directory.
pub fn launch(instance: &Instance, map: Option<&Path>) -> Result<RunningGame, String> {
    if !cfg!(windows) {
        return Err("此原生预览版暂时只支持在 Windows 启动游戏".to_owned());
    }
    if player_running()? {
        return Err("Ballance 已在运行，请先退出游戏".to_owned());
    }
    let mut command = launch_command(instance, map)?;
    let child = command
        .spawn()
        .map_err(|error| format!("启动失败：{error}"))?;
    Ok(Arc::new(Mutex::new(child)))
}

fn launch_command(instance: &Instance, map: Option<&Path>) -> Result<Command, String> {
    let bin = instance.root.join("Bin");
    let mut command = Command::new(bin.join("Player.exe"));
    command.current_dir(&bin);
    if let Some(map) = map {
        if !instance.root.join("BuildingBlocks/BMLPlus.dll").is_file()
            || !instance
                .root
                .join("ModLoader/Mods/RockoonIO.bmodp")
                .is_file()
        {
            return Err("启动指定地图需要已启用的 BML+ 和 RockoonIO 模组".to_owned());
        }
        let canonical_map = map.canonicalize().map_err(error)?;
        let maps = instance
            .root
            .join("ModLoader/Maps")
            .canonicalize()
            .map_err(error)?;
        if !canonical_map.starts_with(maps) || !canonical_map.is_file() {
            return Err("地图文件已移动，请刷新列表".to_owned());
        }
        command.env("ROCKOON_STARTUP", map);
        command.env("ROCKOON_MAP_ONLY", "1");
        command.env("ROCKOON_LEVEL_SLOT", "0");
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    Ok(command)
}

pub fn toggle_mod(instance: &Instance, resource: &Resource) -> Result<(), String> {
    if player_running()? {
        return Err("请先退出游戏，再更改模组".to_owned());
    }
    rename_mod(instance, resource)
}

fn rename_mod(instance: &Instance, resource: &Resource) -> Result<(), String> {
    let parent = instance
        .root
        .join("ModLoader/Mods")
        .canonicalize()
        .map_err(error)?;
    let path = resource.path.canonicalize().map_err(error)?;
    if !path.starts_with(parent)
        || !path.is_file()
        || fs::symlink_metadata(&resource.path)
            .map_err(error)?
            .file_type()
            .is_symlink()
    {
        return Err("模组文件已移动，请刷新列表".to_owned());
    }
    let filename = path
        .file_name()
        .ok_or("无效的模组文件名")?
        .to_string_lossy();
    if filename.to_lowercase().ends_with(".disable") != resource.disabled {
        return Err("模组状态已变化，请刷新列表".to_owned());
    }
    let target = if resource.disabled {
        path.with_file_name(&filename[..filename.len() - ".disable".len()])
    } else {
        path.with_file_name(format!("{filename}.disable"))
    };
    if target.exists() {
        return Err("已存在同名模组文件，请先在文件夹中检查".to_owned());
    }
    fs::rename(path, target).map_err(error)
}

pub fn open_folder(path: &Path) -> Result<(), String> {
    #[cfg(windows)]
    let executable = "explorer.exe";
    #[cfg(target_os = "macos")]
    let executable = "open";
    #[cfg(all(not(windows), not(target_os = "macos")))]
    let executable = "xdg-open";
    Command::new(executable).arg(path).spawn().map_err(error)?;
    Ok(())
}

#[cfg(windows)]
pub fn player_running() -> Result<bool, String> {
    use windows_sys::Win32::{
        Foundation::{CloseHandle, INVALID_HANDLE_VALUE},
        System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
            TH32CS_SNAPPROCESS,
        },
    };
    // The snapshot handle is closed on every path. Process entries own their UTF-16 buffer.
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return Err(format!(
                "无法检查游戏进程：{}",
                std::io::Error::last_os_error()
            ));
        }
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        let mut found = false;
        let mut valid = Process32FirstW(snapshot, &mut entry);
        if valid == 0 {
            let failure = std::io::Error::last_os_error();
            CloseHandle(snapshot);
            return Err(format!("无法检查游戏进程：{failure}"));
        }
        while valid != 0 {
            let end = entry
                .szExeFile
                .iter()
                .position(|&unit| unit == 0)
                .unwrap_or(entry.szExeFile.len());
            if String::from_utf16_lossy(&entry.szExeFile[..end]).eq_ignore_ascii_case("Player.exe")
            {
                found = true;
                break;
            }
            valid = Process32NextW(snapshot, &mut entry);
        }
        CloseHandle(snapshot);
        Ok(found)
    }
}

#[cfg(not(windows))]
pub fn player_running() -> Result<bool, String> {
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn instance() -> tempfile::TempDir {
        let temporary = tempfile::tempdir().unwrap();
        for directory in [
            "Bin",
            "BuildingBlocks",
            "3D Entities",
            "ModLoader/Maps",
            "ModLoader/Mods",
        ] {
            fs::create_dir_all(temporary.path().join(directory)).unwrap();
        }
        for file in ["Bin/Player.exe", "base.cmo", "Database.tdb"] {
            fs::write(temporary.path().join(file), []).unwrap();
        }
        temporary
    }

    #[test]
    fn rejects_incomplete_game_without_writing_defaults() {
        let temporary = instance();
        fs::remove_file(temporary.path().join("Database.tdb")).unwrap();
        assert!(
            discover(temporary.path().to_owned())
                .unwrap_err()
                .contains("Database.tdb")
        );
        assert!(!temporary.path().join("Database.tdb").exists());
    }

    #[test]
    fn scans_nested_unicode_maps_and_disabled_mods() {
        let temporary = instance();
        fs::create_dir_all(temporary.path().join("ModLoader/Maps/作者")).unwrap();
        fs::write(temporary.path().join("ModLoader/Maps/作者/地图.NMO"), []).unwrap();
        fs::write(temporary.path().join("ModLoader/Maps/readme.txt"), []).unwrap();
        fs::write(
            temporary.path().join("ModLoader/Maps/hidden.cmo.disable"),
            [],
        )
        .unwrap();
        fs::write(
            temporary.path().join("ModLoader/Mods/Sample.BMODP.disable"),
            [],
        )
        .unwrap();
        let game = discover(temporary.path().to_owned()).unwrap();
        assert_eq!(game.maps.len(), 1);
        assert!(game.maps[0].name.contains("地图.NMO"));
        assert_eq!(game.mods.len(), 1);
        assert!(game.mods[0].disabled);
    }

    #[test]
    fn ignores_disabled_loaders_and_prefers_enabled_bml_plus() {
        let temporary = instance();
        fs::write(temporary.path().join("BuildingBlocks/BML.dll.disable"), []).unwrap();
        assert_eq!(
            discover(temporary.path().to_owned()).unwrap().loader,
            "原版"
        );
        fs::write(temporary.path().join("BuildingBlocks/BML.dll"), []).unwrap();
        fs::write(temporary.path().join("BuildingBlocks/BMLPlus.dll"), []).unwrap();
        assert_eq!(
            discover(temporary.path().to_owned()).unwrap().loader,
            "BML+"
        );
    }

    #[test]
    fn toggles_mods_without_losing_bytes_and_refuses_collisions() {
        let temporary = instance();
        let mod_path = temporary.path().join("ModLoader/Mods/示例.bmodp");
        fs::write(&mod_path, b"original mod bytes").unwrap();
        let game = discover(temporary.path().to_owned()).unwrap();
        rename_mod(&game, &game.mods[0]).unwrap();
        let disabled = mod_path.with_file_name("示例.bmodp.disable");
        assert!(!mod_path.exists());
        assert_eq!(fs::read(&disabled).unwrap(), b"original mod bytes");
        let game = discover(temporary.path().to_owned()).unwrap();
        fs::write(&mod_path, b"other mod bytes").unwrap();
        assert!(rename_mod(&game, &game.mods[0]).is_err());
        assert_eq!(fs::read(&mod_path).unwrap(), b"other mod bytes");
        assert_eq!(fs::read(&disabled).unwrap(), b"original mod bytes");
        fs::remove_file(&mod_path).unwrap();
        rename_mod(&game, &game.mods[0]).unwrap();
        assert_eq!(fs::read(&mod_path).unwrap(), b"original mod bytes");
        assert!(!disabled.exists());
    }

    #[test]
    fn refuses_mod_files_outside_the_selected_instance() {
        let temporary = instance();
        let outside = tempfile::tempdir().unwrap();
        let path = outside.path().join("outside.bmodp");
        fs::write(&path, b"keep").unwrap();
        let game = discover(temporary.path().to_owned()).unwrap();
        let resource = Resource {
            path: path.clone(),
            name: "outside.bmodp".to_owned(),
            disabled: false,
        };
        assert!(rename_mod(&game, &resource).is_err());
        assert_eq!(fs::read(path).unwrap(), b"keep");
    }

    #[test]
    fn launch_command_preserves_bin_working_directory_and_unicode_map_environment() {
        let temporary = instance();
        fs::write(temporary.path().join("BuildingBlocks/BMLPlus.dll"), []).unwrap();
        fs::write(temporary.path().join("ModLoader/Mods/RockoonIO.bmodp"), []).unwrap();
        let map = temporary.path().join("ModLoader/Maps/中文地图.nmo");
        fs::write(&map, []).unwrap();
        let instance = discover(temporary.path().to_owned()).unwrap();
        let command = launch_command(&instance, Some(&map)).unwrap();
        assert_eq!(
            command.get_program(),
            temporary.path().join("Bin").join("Player.exe").as_os_str()
        );
        assert_eq!(
            command.get_current_dir(),
            Some(temporary.path().join("Bin").as_path())
        );
        let environment: std::collections::BTreeMap<_, _> = command
            .get_envs()
            .map(|(key, value)| {
                (
                    key.to_string_lossy().into_owned(),
                    value.unwrap().to_string_lossy().into_owned(),
                )
            })
            .collect();
        assert_eq!(environment["ROCKOON_STARTUP"], map.to_string_lossy());
        assert_eq!(environment["ROCKOON_MAP_ONLY"], "1");
        assert_eq!(environment["ROCKOON_LEVEL_SLOT"], "0");
        assert_eq!(
            launch_command(&instance, None).unwrap().get_envs().count(),
            0
        );
    }

    #[test]
    fn map_launch_requires_integration_and_rejects_external_files() {
        let temporary = instance();
        let map = temporary.path().join("ModLoader/Maps/example.nmo");
        fs::write(&map, []).unwrap();
        let instance = discover(temporary.path().to_owned()).unwrap();
        assert!(
            launch_command(&instance, Some(&map))
                .unwrap_err()
                .contains("RockoonIO")
        );
        fs::write(temporary.path().join("BuildingBlocks/BMLPlus.dll"), []).unwrap();
        fs::write(temporary.path().join("ModLoader/Mods/RockoonIO.bmodp"), []).unwrap();
        let outside = temporary.path().join("outside.nmo");
        fs::write(&outside, []).unwrap();
        assert!(launch_command(&instance, Some(&outside)).is_err());
    }
}
