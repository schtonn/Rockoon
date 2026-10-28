//! Components belong to one installation; versions come from its files, not the server's latest tag.
use std::{fs, path::Path};
#[derive(Clone, Debug)]
pub struct Installed {
    pub key: String,
    pub name: String,
    pub version: String,
}
pub fn scan(root: &Path) -> Vec<Installed> {
    let record: serde_json::Value = fs::read(root.join("ModLoader/.rockoon-patch.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default();
    let player = if record.get("player-msvc2022").is_some() {
        "player-msvc2022"
    } else {
        "player-vc6"
    };
    [
        (player, "NewPlayer", "Bin/Player.exe"),
        ("bmlplus", "BML+", "BuildingBlocks/BMLPlus.dll"),
        ("bml", "BML", "BuildingBlocks/BML.dll"),
    ]
    .into_iter()
    .filter_map(|(key, name, relative)| {
        let file = root.join(relative);
        let metadata = fs::metadata(&file).ok()?;
        if !metadata.is_file() || (key.starts_with("player-") && metadata.len() == 155648) {
            return None;
        }
        let version = file_version(&file)
            .or_else(|| {
                record[key]["tag"]
                    .as_str()
                    .filter(|v| !v.is_empty())
                    .map(str::to_owned)
            })
            .unwrap_or_else(|| "版本未识别".into());
        Some(Installed {
            key: key.into(),
            name: name.into(),
            version,
        })
    })
    .collect()
}
#[cfg(windows)]
fn file_version(path: &Path) -> Option<String> {
    use std::{ffi::c_void, os::windows::ffi::OsStrExt, ptr};
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileVersionInfoSizeW, GetFileVersionInfoW, VS_FIXEDFILEINFO, VerQueryValueW,
    };
    let path: Vec<_> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    unsafe {
        let size = GetFileVersionInfoSizeW(path.as_ptr(), ptr::null_mut());
        if size == 0 || size > 4 * 1024 * 1024 {
            return None;
        }
        let mut bytes = vec![0u32; (size as usize).div_ceil(4)];
        if GetFileVersionInfoW(path.as_ptr(), 0, size, bytes.as_mut_ptr().cast()) == 0 {
            return None;
        }
        let mut query = |key: &str| -> Option<(*mut c_void, u32)> {
            let key: Vec<_> = key.encode_utf16().chain(Some(0)).collect();
            let mut data = ptr::null_mut();
            let mut len = 0;
            (VerQueryValueW(bytes.as_mut_ptr().cast(), key.as_ptr(), &mut data, &mut len) != 0
                && !data.is_null())
            .then_some((data, len))
        };
        if let Some((data, len)) = query("\\VarFileInfo\\Translation") {
            for pair in std::slice::from_raw_parts(data.cast::<u16>(), len as usize / 2)
                .as_chunks::<2>()
                .0
            {
                for field in ["ProductVersion", "FileVersion"] {
                    if let Some((text, len)) = query(&format!(
                        "\\StringFileInfo\\{:04x}{:04x}\\{field}",
                        pair[0], pair[1]
                    )) {
                        let text = String::from_utf16_lossy(std::slice::from_raw_parts(
                            text.cast::<u16>(),
                            len as usize,
                        ));
                        let text = text.trim_matches('\0').trim();
                        if !text.is_empty() {
                            return Some(text.into());
                        }
                    }
                }
            }
        }
        let (data, len) = query("\\")?;
        if (len as usize) < std::mem::size_of::<VS_FIXEDFILEINFO>() {
            return None;
        }
        let info = ptr::read_unaligned(data.cast::<VS_FIXEDFILEINFO>());
        if info.dwSignature != 0xfeef04bd {
            return None;
        }
        Some(format!(
            "{}.{}.{}.{}",
            info.dwFileVersionMS >> 16,
            info.dwFileVersionMS & 0xffff,
            info.dwFileVersionLS >> 16,
            info.dwFileVersionLS & 0xffff
        ))
    }
}
#[cfg(not(windows))]
fn file_version(_: &Path) -> Option<String> {
    None
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Target {
    pub index: usize,
    pub label: String,
}
impl std::fmt::Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}
pub fn targets(patches: &[crate::jobs::Patch]) -> Vec<Target> {
    patches
        .iter()
        .enumerate()
        .map(|(index, patch)| Target {
            index,
            label: match patch.key.as_str() {
                "player-vc6" => "NewPlayer",
                "player-msvc2022" => "NewPlayer 备用构建",
                "bmlplus" => "BML+",
                "bml" => "BML 旧版",
                _ => &patch.name,
            }
            .into(),
        })
        .collect()
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReleaseChoice {
    Latest,
    Version(crate::jobs::Version),
}
impl std::fmt::Display for ReleaseChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Latest => f.write_str("最新版本"),
            Self::Version(v) => f.write_str(
                v.note
                    .as_deref()
                    .filter(|s| !s.is_empty())
                    .unwrap_or(&format!("版本 #{}", v.version)),
            ),
        }
    }
}

pub fn help(key: &str) -> &'static str {
    if key.starts_with("player-") {
        "修复兼容问题。"
    } else if matches!(key, "bml" | "bmlplus") {
        "加载地图和模组。"
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        for dir in ["Bin", "BuildingBlocks", "ModLoader"] {
            fs::create_dir(root.path().join(dir)).unwrap();
        }
        root
    }
    #[test]
    fn installed_summary_hides_absent_and_disabled_components_and_merges_player_builds() {
        let root = fixture();
        fs::write(root.path().join("Bin/Player.exe"), b"new player").unwrap();
        fs::write(
            root.path().join("BuildingBlocks/BML.dll.disable"),
            b"disabled",
        )
        .unwrap();
        fs::write(root.path().join("BuildingBlocks/BMLPlus.dll"), b"loader").unwrap();
        fs::write(
            root.path().join("ModLoader/.rockoon-patch.json"),
            br#"{"player-msvc2022":{"tag":"v0.3.9"},"bmlplus":{"tag":"v0.3.13"}}"#,
        )
        .unwrap();
        let items = scan(root.path());
        assert_eq!(items.len(), 2);
        assert_eq!(
            (items[0].key.as_str(), items[0].version.as_str()),
            ("player-msvc2022", "v0.3.9")
        );
        assert_eq!(
            (items[1].key.as_str(), items[1].version.as_str()),
            ("bmlplus", "v0.3.13")
        );
        let other = fixture();
        fs::write(other.path().join("Bin/Player.exe"), vec![0u8; 155648]).unwrap();
        assert!(scan(other.path()).is_empty());
        fs::write(other.path().join("BuildingBlocks/BML.dll"), b"unknown").unwrap();
        let items = scan(other.path());
        assert_eq!(items[0].version, "版本未识别");
    }
    #[cfg(windows)]
    #[test]
    fn embedded_version_takes_precedence_over_old_install_record() {
        let source = std::path::PathBuf::from(std::env::var_os("WINDIR").unwrap())
            .join("System32/version.dll");
        let actual = file_version(&source).expect("Windows version library has version resources");
        let root = fixture();
        fs::copy(source, root.path().join("BuildingBlocks/BMLPlus.dll")).unwrap();
        fs::write(
            root.path().join("ModLoader/.rockoon-patch.json"),
            br#"{"bmlplus":{"tag":"stale"}}"#,
        )
        .unwrap();
        assert_eq!(scan(root.path())[0].version, actual);
    }
    #[test]
    #[ignore = "read-only inspection of a supplied game directory"]
    fn read_installed_game_versions() {
        let root =
            std::env::var_os("ROCKOON_COMPONENT_VERSION_TEST_ROOT").expect("provide game root");
        let items = scan(Path::new(&root));
        assert!(!items.is_empty());
        for item in items {
            assert_ne!(item.version, "版本未识别");
            println!("{} {}", item.name, item.version);
        }
    }
}
