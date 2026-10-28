use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

pub fn error(e: impl std::fmt::Display) -> String {
    e.to_string()
}
pub fn idle(root: &Path) -> Result<(), String> {
    if crate::game::player_running(root)? {
        Err("当前游戏目录正在使用，请先退出该目录的游戏再修改文件或配置".into())
    } else {
        Ok(())
    }
}
pub fn id() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .to_string()
}
fn link(path: &Path) -> Result<bool, String> {
    let m = fs::symlink_metadata(path).map_err(error)?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        Ok(m.file_attributes() & 0x400 != 0)
    }
    #[cfg(not(windows))]
    {
        Ok(m.file_type().is_symlink())
    }
}
/// Reject traversal and every symlink/junction, including dangling links.
pub fn checked(root: &Path, path: &Path) -> Result<PathBuf, String> {
    let root = std::path::absolute(root).map_err(error)?;
    let path = std::path::absolute(path).map_err(error)?;
    let relative = path
        .strip_prefix(&root)
        .map_err(|_| "文件不在当前游戏目录中")?;
    let mut walk = root.clone();
    for part in relative.components() {
        if !matches!(part, Component::Normal(_)) {
            return Err("不允许越过游戏目录".into());
        }
        walk.push(part);
        match fs::symlink_metadata(&walk) {
            Ok(_) if link(&walk)? => return Err("不通过符号链接或目录联接修改文件".into()),
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(error(e)),
        }
    }
    Ok(path)
}
pub fn atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path.parent().ok_or("路径缺少上级目录")?;
    fs::create_dir_all(parent).map_err(error)?;
    let mut temp = tempfile::NamedTempFile::new_in(parent).map_err(error)?;
    temp.write_all(bytes).map_err(error)?;
    temp.as_file().sync_all().map_err(error)?;
    temp.persist(path).map_err(error)?;
    Ok(())
}
/// Keep backups of replaced files and roll back the whole batch on failure.
pub fn replace(root: &Path, changes: &[(PathBuf, Vec<u8>)]) -> Result<(), String> {
    let backup = checked(root, &root.join(".rockoon/backups").join(id()))?;
    let mut journal = Vec::new();
    for (path, _) in changes {
        let path = checked(root, path)?;
        let saved = if path.exists() {
            let saved = backup.join(path.strip_prefix(root).map_err(error)?);
            fs::create_dir_all(saved.parent().ok_or("无效备份路径")?).map_err(error)?;
            fs::copy(&path, &saved).map_err(error)?;
            Some(saved)
        } else {
            None
        };
        journal.push((path, saved));
    }
    for (i, (path, bytes)) in changes.iter().enumerate() {
        if let Err(e) = atomic(path, bytes) {
            for (target, saved) in journal[..=i].iter().rev() {
                match saved {
                    Some(saved) => {
                        let _ = fs::read(saved)
                            .map_err(error)
                            .and_then(|b| atomic(target, &b));
                    }
                    None => {
                        let _ = fs::remove_file(target);
                    }
                }
            }
            return Err(format!("写入失败，已回退：{e}"));
        }
    }
    Ok(())
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Maps,
    Mods,
    Skies,
    Textures,
    Sounds,
}
impl Kind {
    pub fn dir(self) -> &'static str {
        match self {
            Self::Maps => "ModLoader/Maps",
            Self::Mods => "ModLoader/Mods",
            Self::Skies => "Textures/Sky",
            Self::Textures => "Textures",
            Self::Sounds => "Sounds",
        }
    }
    pub fn accepts(self, name: &str) -> bool {
        let lower = name.to_lowercase();
        let name = lower.strip_suffix(".disable").unwrap_or(&lower);
        let exts: &[&str] = match self {
            Self::Maps => &["nmo", "cmo"],
            Self::Mods => &["bmod", "bmodp", "zip"],
            Self::Skies => &["bmp"],
            Self::Textures => &["bmp", "tga"],
            Self::Sounds => &["wav"],
        };
        exts.iter().any(|ext| name.ends_with(&format!(".{ext}")))
    }
}
impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Maps => "地图",
            Self::Mods => "模组",
            Self::Skies => "天空盒",
            Self::Textures => "纹理",
            Self::Sounds => "音效",
        })
    }
}
#[derive(Clone, Debug)]
pub struct Entry {
    pub kind: Kind,
    pub path: PathBuf,
    pub name: String,
    pub directory: bool,
    pub disabled: bool,
}
pub fn list(root: &Path, kind: Kind, dir: &Path) -> Result<Vec<Entry>, String> {
    let base = root.join(kind.dir());
    let dir = checked(root, dir)?;
    if !dir.starts_with(&base) {
        return Err("资源目录越界".into());
    }
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut entries = Vec::new();
    for entry in fs::read_dir(dir).map_err(error)? {
        let entry = entry.map_err(error)?;
        if link(&entry.path())? {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let directory = entry.file_type().map_err(error)?.is_dir();
        if directory || kind.accepts(&name) {
            entries.push(Entry {
                kind,
                disabled: name.to_lowercase().ends_with(".disable"),
                name,
                path: entry.path(),
                directory,
            });
        }
    }
    entries.sort_by_key(|e| (!e.directory, e.name.to_lowercase()));
    Ok(entries)
}
/// Flatten known local resource roots without duplicate sky textures or following links.
pub fn collection(root: &Path) -> Result<Vec<Entry>, String> {
    let mut result = Vec::new();
    for kind in [
        Kind::Maps,
        Kind::Mods,
        Kind::Skies,
        Kind::Textures,
        Kind::Sounds,
    ] {
        let base = root.join(kind.dir());
        let mut pending = vec![(base.clone(), 0usize)];
        while let Some((dir, depth)) = pending.pop() {
            if depth > 32 {
                return Err("资源目录层级超过 32 层".into());
            }
            for mut entry in list(root, kind, &dir)? {
                if kind == Kind::Textures && entry.path.starts_with(root.join("Textures/Sky")) {
                    continue;
                }
                if entry.directory {
                    pending.push((entry.path, depth + 1));
                } else {
                    entry.name = format!(
                        "{} / {}",
                        kind,
                        entry.path.strip_prefix(&base).map_err(error)?.display()
                    );
                    result.push(entry);
                    if result.len() > 20000 {
                        return Err("资源数量超过 20000，请选择具体类型浏览".into());
                    }
                }
            }
        }
    }
    result.sort_by_cached_key(|e| e.name.to_lowercase());
    Ok(result)
}
pub fn matches(name: &str, query: &str) -> bool {
    let name = name.to_lowercase();
    let name = name.strip_suffix(".disable").unwrap_or(&name);
    let stem = Path::new(name)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    query
        .split_whitespace()
        .all(|q| stem.contains(&q.to_lowercase()))
}
pub fn folder(root: &Path, dir: &Path, name: &str) -> Result<(), String> {
    idle(root)?;
    if name.trim().is_empty() || name.contains(['/', '\\', ':']) || name == "." || name == ".." {
        return Err("请输入有效文件夹名".into());
    }
    fs::create_dir(checked(root, &dir.join(name.trim()))?).map_err(error)
}
pub fn relocate(root: &Path, source: &Path, target: &Path) -> Result<(), String> {
    idle(root)?;
    let source = checked(root, source)?;
    let target = checked(root, target)?;
    if target.exists() {
        return Err("目标已有同名文件".into());
    }
    if target.starts_with(&source) {
        return Err("不能把文件夹移动到自身内部".into());
    }
    fs::rename(source, target).map_err(error)
}
pub fn trash(root: &Path, path: &Path) -> Result<PathBuf, String> {
    idle(root)?;
    let path = checked(root, path)?;
    let saved = checked(
        root,
        &root
            .join(".rockoon/trash")
            .join(id())
            .join(path.file_name().ok_or("无效文件名")?),
    )?;
    fs::create_dir_all(saved.parent().ok_or("无效回收目录")?).map_err(error)?;
    fs::rename(path, &saved).map_err(error)?;
    Ok(saved)
}
pub fn import_files(root: &Path, dir: &Path, kind: Kind, paths: &[PathBuf]) -> Result<(), String> {
    idle(root)?;
    let mut changes = Vec::new();
    for path in paths {
        let name = path.file_name().ok_or("无效文件名")?;
        if !kind.accepts(&name.to_string_lossy()) {
            return Err("文件类型不适合当前资源页".into());
        }
        let target = checked(root, &dir.join(name))?;
        if target.exists() {
            return Err(format!(
                "已存在 {}；请先移动或删除旧文件",
                name.to_string_lossy()
            ));
        }
        changes.push((target, fs::read(path).map_err(error)?));
    }
    replace(root, &changes)
}
/// Install selected sky faces with the same backup/atomic replacement as BRP imports.
pub fn import_sky_files(
    root: &Path,
    dir: &Path,
    paths: &[PathBuf],
    sky: &str,
) -> Result<(), String> {
    idle(root)?;
    if !matches!(
        sky,
        "X" | "A" | "B" | "C" | "D" | "E" | "F" | "G" | "H" | "I" | "J" | "K" | "L" | "M"
    ) {
        return Err("无效的天空盒关卡".into());
    }
    let mut changes = Vec::new();
    let mut names = std::collections::HashSet::new();
    for path in paths {
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or("无效文件名")?;
        let parts: Vec<_> = name.split('_').collect();
        if parts.len() != 3 || !parts[0].eq_ignore_ascii_case("Sky") {
            return Err("天空盒文件应使用 Sky_X_Front.bmp 这样的名称".into());
        }
        let direction = ["Back", "Down", "Front", "Left", "Right"]
            .into_iter()
            .find(|d| parts[2].eq_ignore_ascii_case(&format!("{d}.bmp")))
            .ok_or("无法识别天空盒方位")?;
        let target = checked(root, &dir.join(format!("Sky_{sky}_{direction}.bmp")))?;
        if !names.insert(target.clone()) {
            return Err("同一方位只能导入一张图片".into());
        }
        changes.push((target, fs::read(path).map_err(error)?));
    }
    replace(root, &changes)
}
pub fn archive_entries(path: &Path, strip: bool) -> Result<Vec<(PathBuf, Vec<u8>)>, String> {
    let mut archive = zip::ZipArchive::new(fs::File::open(path).map_err(error)?).map_err(error)?;
    let mut result = Vec::new();
    let mut total = 0u64;
    let mut names = std::collections::HashSet::new();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(error)?;
        if entry.is_dir() {
            continue;
        }
        if entry.unix_mode().is_some_and(|m| m & 0o170000 == 0o120000) {
            return Err("压缩包包含链接".into());
        }
        let name = entry.name().replace('\\', "/");
        if name.split('/').any(|p| p == ".." || p.contains(':')) || name.starts_with('/') {
            return Err("压缩包包含越界路径".into());
        }
        let mut relative = PathBuf::from(name);
        if strip {
            relative = relative.components().skip(1).collect();
        }
        if relative.as_os_str().is_empty() {
            continue;
        }
        if !names.insert(relative.to_string_lossy().to_lowercase()) {
            return Err("压缩包包含重复路径".into());
        }
        if !relative
            .components()
            .all(|p| matches!(p, Component::Normal(_)))
        {
            return Err("压缩包包含非法路径".into());
        }
        total = total.checked_add(entry.size()).ok_or("资源包过大")?;
        if total > 512 * 1024 * 1024 {
            return Err("解压后超过 512 MiB".into());
        }
        let size = entry.size();
        let mut bytes = Vec::new();
        entry
            .by_ref()
            .take(size + 1)
            .read_to_end(&mut bytes)
            .map_err(error)?;
        if bytes.len() as u64 != size {
            return Err("压缩包条目长度不正确".into());
        }
        result.push((relative, bytes));
    }
    Ok(result)
}
pub fn brp(root: &Path, archive: &Path, sky: &str) -> Result<String, String> {
    brp_files(root, archive, sky).map(|(message, _)| message)
}
pub fn brp_files(root: &Path, archive: &Path, sky: &str) -> Result<(String, Vec<PathBuf>), String> {
    idle(root)?;
    let info = rockoon_core::brp::validate_brp(archive, root).map_err(error)?;
    archive_entries(archive, false)?;
    let staging = tempfile::tempdir().map_err(error)?;
    for folder in ["Sounds", "Textures", "BuildingBlocks"] {
        let source = root.join(folder);
        let target = staging.path().join(folder);
        fs::create_dir_all(&target).map_err(error)?;
        if source.exists() {
            for entry in fs::read_dir(source).map_err(error)? {
                let entry = entry.map_err(error)?;
                if entry.file_type().map_err(error)?.is_file() {
                    fs::write(target.join(entry.file_name()), []).map_err(error)?;
                }
            }
        }
    }
    let installed =
        rockoon_core::brp::install_brp(archive, staging.path(), Some(sky)).map_err(error)?;
    let mut changes = Vec::new();
    for file in &installed.installed_paths {
        let file = PathBuf::from(file);
        let rel = file.strip_prefix(staging.path()).map_err(error)?;
        changes.push((
            checked(root, &root.join(rel))?,
            fs::read(file).map_err(error)?,
        ));
    }
    replace(root, &changes)?;
    let message = format!(
        "{}：已导入 {} 个文件{}",
        info.manifest.name.unwrap_or_else(|| "资源包".into()),
        changes.len(),
        if installed.mod_variant_mismatch {
            "；包内模组与当前加载器不匹配"
        } else {
            ""
        }
    );
    Ok((message, changes.into_iter().map(|(p, _)| p).collect()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_resources_include_nested_files_and_do_not_duplicate_sky_textures() {
        let dir = tempfile::tempdir().unwrap();
        for name in [
            "ModLoader/Maps/nested/test.nmo",
            "ModLoader/Mods/test.bmodp",
            "Textures/Sky/Sky_A_Front.bmp",
            "Textures/Ball_Wood.bmp",
            "Sounds/test.wav",
        ] {
            let path = dir.path().join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, b"fixture").unwrap();
        }
        let entries = collection(dir.path()).unwrap();
        assert_eq!(entries.len(), 5);
        assert_eq!(entries.iter().filter(|e| e.kind == Kind::Skies).count(), 1);
        assert_eq!(
            entries.iter().filter(|e| e.kind == Kind::Textures).count(),
            1
        );
        assert!(entries.iter().any(|e| e.name.contains("nested")));
    }
    #[test]
    fn search_normalizes_extensions_and_word_order() {
        assert!(matches("Sky Return.NMO.disable", "return sky"));
        assert!(!matches("Sky Return.NMO", "return missing"));
    }
    #[test]
    fn traversal_is_rejected_and_backups_preserve_original() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        assert!(checked(root, &root.join("../escape")).is_err());
        let file = root.join("Database.tdb");
        fs::write(&file, b"before").unwrap();
        replace(root, &[(file.clone(), b"after".to_vec())]).unwrap();
        assert_eq!(fs::read(file).unwrap(), b"after");
        let snapshot = fs::read_dir(root.join(".rockoon/backups"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        assert_eq!(fs::read(snapshot.join("Database.tdb")).unwrap(), b"before");
    }
    #[test]
    fn archive_rejects_windows_traversal() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.zip");
        let mut zip = zip::ZipWriter::new(fs::File::create(&path).unwrap());
        zip.start_file(
            "content\\..\\escape",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
        zip.write_all(b"bad").unwrap();
        zip.finish().unwrap();
        assert!(archive_entries(&path, false).is_err());
    }
}

pub fn classify(paths: Vec<PathBuf>) -> Result<(Vec<PathBuf>, Vec<PathBuf>), String> {
    let mut packages = Vec::new();
    let mut raw = Vec::new();
    for file in paths {
        let ext = file
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        let package = if ext == "brp" {
            true
        } else if ext == "zip" {
            fs::File::open(&file)
                .ok()
                .and_then(|f| zip::ZipArchive::new(f).ok())
                .is_some_and(|mut archive| {
                    let Ok(entry) = archive.by_name("manifest.json") else {
                        return false;
                    };
                    let mut bytes = Vec::new();
                    if entry.take(256 * 1024).read_to_end(&mut bytes).is_err() {
                        return false;
                    }
                    serde_json::from_slice::<serde_json::Value>(&bytes).is_ok_and(|v| {
                        v.get("manifest_version").is_some() && v.get("category").is_some()
                    })
                })
        } else {
            false
        };
        if package {
            packages.push(file)
        } else {
            raw.push(file)
        }
    }
    Ok((packages, raw))
}
#[cfg(test)]
mod transaction_tests {
    use super::*;
    #[test]
    #[cfg(windows)]
    fn failed_second_write_rolls_back_first() {
        use std::os::windows::fs::OpenOptionsExt;
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let first = root.join("first.ini");
        let second = root.join("second.ini");
        fs::write(&first, b"first-before").unwrap();
        fs::write(&second, b"second-before").unwrap();
        let _held = fs::OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(&second)
            .unwrap();
        assert!(
            replace(
                root,
                &[
                    (first.clone(), b"first-after".to_vec()),
                    (second.clone(), b"second-after".to_vec())
                ]
            )
            .is_err()
        );
        assert_eq!(fs::read(first).unwrap(), b"first-before");
        assert_eq!(fs::read(second).unwrap(), b"second-before");
    }
    #[test]
    fn ordinary_mod_zip_manifest_is_not_a_brp() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("mod.zip");
        let mut zip = zip::ZipWriter::new(fs::File::create(&file).unwrap());
        zip.start_file("manifest.json", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(br#"{"name":"ordinary mod"}"#).unwrap();
        zip.finish().unwrap();
        let (packages, raw) = classify(vec![file]).unwrap();
        assert!(packages.is_empty());
        assert_eq!(raw.len(), 1);
    }
}
