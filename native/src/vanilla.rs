use crate::workspace::{self, error};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path};
fn entries() -> Result<BTreeMap<String, String>, String> {
    let value: serde_json::Value = serde_json::from_str(include_str!(
        "../../src-tauri/resources/vanilla/manifest.json"
    ))
    .map_err(error)?;
    let files: BTreeMap<String, String> =
        serde_json::from_value(value["files"].clone()).map_err(error)?;
    Ok(files
        .into_iter()
        .filter(|(p, _)| p.starts_with("3D Entities/") && p.to_lowercase().ends_with(".nmo"))
        .collect())
}
pub fn verify(root: &Path) -> Result<Vec<String>, String> {
    let mut diff = Vec::new();
    for (rel, want) in entries()? {
        let file = workspace::checked(root, &root.join(&rel))?;
        let hash = fs::read(file)
            .ok()
            .map(|b| format!("{:x}", Sha256::digest(b)));
        if hash.as_deref() != Some(&want) {
            diff.push(rel);
        }
    }
    Ok(diff)
}
pub fn restore(root: &Path) -> Result<String, String> {
    workspace::idle()?;
    let wanted = entries()?;
    let mut changes = Vec::new();
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(include_bytes!(
        "../../src-tauri/resources/vanilla/vanilla-core.zip"
    )))
    .map_err(error)?;
    for rel in verify(root)? {
        use std::io::Read;
        let mut entry = archive.by_name(&rel).map_err(error)?;
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).map_err(error)?;
        if format!("{:x}", Sha256::digest(&bytes)) != wanted[&rel] {
            return Err("内置恢复文件校验失败".into());
        }
        changes.push((root.join(rel), bytes));
    }
    workspace::replace(root, &changes)?;
    Ok(format!("已恢复 {} 个原版文件", changes.len()))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn restores_only_manifest_levels_and_preserves_extras() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("3D Entities")).unwrap();
        fs::write(root.path().join("3D Entities/custom.nmo"), b"custom").unwrap();
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(include_bytes!(
            "../../src-tauri/resources/vanilla/vanilla-core.zip"
        )))
        .unwrap();
        let mut changes = Vec::new();
        for (rel, want) in entries().unwrap() {
            use std::io::Read;
            let mut bytes = Vec::new();
            archive
                .by_name(&rel)
                .unwrap()
                .read_to_end(&mut bytes)
                .unwrap();
            assert_eq!(format!("{:x}", Sha256::digest(&bytes)), want);
            changes.push((root.path().join(rel), bytes));
        }
        workspace::replace(root.path(), &changes).unwrap();
        assert!(verify(root.path()).unwrap().is_empty());
        assert_eq!(
            fs::read(root.path().join("3D Entities/custom.nmo")).unwrap(),
            b"custom"
        );
    }
}
