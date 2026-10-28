use crate::{
    config,
    workspace::{self, error},
};
use serde_json::{Value, json};
use std::{fs, path::Path};

pub fn load(root: &Path) -> Result<Value, String> {
    match fs::read(root.join(".rockoon/profiles.json")) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(error),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Ok(json!({"profiles":[],"currentId":""}))
        }
        Err(e) => Err(error(e)),
    }
}
fn capture(root: &Path) -> Result<Value, String> {
    let mut opts = config::options(root)?;
    opts.as_object_mut()
        .ok_or("配置格式异常")?
        .retain(|k, _| k != "levelLock" && k != "highscores");
    let mut disabled = Vec::new();
    for dir in ["ModLoader/Maps", "ModLoader/Mods", "BuildingBlocks"] {
        let mut pending = vec![root.join(dir)];
        while let Some(dir) = pending.pop() {
            if !dir.exists() {
                continue;
            }
            for item in fs::read_dir(dir).map_err(error)? {
                let path = item.map_err(error)?.path();
                if workspace::checked(root, &path).is_err() {
                    continue;
                }
                if path.is_dir() {
                    pending.push(path);
                } else {
                    let rel = path
                        .strip_prefix(root)
                        .map_err(error)?
                        .to_string_lossy()
                        .replace('\\', "/");
                    if let Some(active) = rel.strip_suffix(".disable") {
                        disabled.push(active.to_owned());
                    }
                }
            }
        }
    }
    let mut mods = json!({});
    let mut raw = json!({});
    for file in config::files(root)? {
        if file.starts_with(root.join("ModLoader/Configs")) {
            let name = file
                .file_name()
                .ok_or("无效配置名")?
                .to_string_lossy()
                .to_string();
            let doc = config::load(root, config::Source::File(file.clone()))?;
            let mut categories = json!({});
            let mut entries = json!({});
            for group in doc.groups() {
                categories[&group] = json!("");
                entries[&group]=json!(doc.fields.iter().filter(|f|f.group==group).map(|f|json!({
                    "name":f.key,"description":"","datatype":f.prefix.split_whitespace().next().filter(|p|["B","I","F","S","K"].contains(p)).unwrap_or("S"),"value":f.value
                })).collect::<Vec<_>>());
            }
            mods[&name] = json!({"categories":categories,"entries":entries});
            raw[&name] = json!(fs::read_to_string(file).map_err(error)?);
        }
    }
    let launch = fs::read_to_string(root.join("Bin/Player.ini"))
        .ok()
        .map(|s| config::ini_json(&s));
    Ok(
        json!({"disabledFiles":disabled,"gameOptions":opts,"launchConfig":launch,"modConfigs":mods,"nativeRawConfigs":raw}),
    )
}
fn mod_text(value: &Value) -> String {
    let mut text = String::new();
    if let Some(groups) = value["entries"].as_object() {
        for (group, entries) in groups {
            text.push_str(&format!("{group} {{\r\n"));
            if let Some(entries) = entries.as_array() {
                for entry in entries {
                    text.push_str(&format!(
                        "\t{} {} {}\r\n",
                        entry["datatype"].as_str().unwrap_or("S"),
                        entry["name"].as_str().unwrap_or(""),
                        entry["value"].as_str().unwrap_or("")
                    ));
                }
            }
            text.push_str("}\r\n");
        }
    }
    text
}
#[derive(Default)]
struct Plan {
    changes: Vec<(std::path::PathBuf, Vec<u8>)>,
    moves: Vec<(std::path::PathBuf, std::path::PathBuf)>,
}
fn plan(root: &Path, profile: &Value) -> Result<Plan, String> {
    let mut changes = Vec::new();
    if let Some(config) = profile.get("launchConfig").filter(|v| v.is_object()) {
        changes.push((
            root.join("Bin/Player.ini"),
            config::ini_text(config).into_bytes(),
        ));
    }
    if let Some(configs) = profile["modConfigs"].as_object() {
        for (name, value) in configs {
            if name.contains(['/', '\\', ':']) {
                return Err("配置方案包含非法文件名".into());
            }
            let text = profile["nativeRawConfigs"][name]
                .as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| mod_text(value));
            changes.push((root.join("ModLoader/Configs").join(name), text.into_bytes()));
        }
    }
    for (path, _) in &changes {
        workspace::checked(root, path)?;
    }
    let current = capture(root)?;
    let current_disabled: Vec<String> =
        serde_json::from_value(current["disabledFiles"].clone()).map_err(error)?;
    let target_disabled: Vec<String> =
        serde_json::from_value(profile["disabledFiles"].clone()).map_err(error)?;
    let mut moves = Vec::new();
    for (items, enable) in [(&current_disabled, true), (&target_disabled, false)] {
        for rel in items {
            if (enable && target_disabled.contains(rel))
                || (!enable && current_disabled.contains(rel))
            {
                continue;
            }
            if !["ModLoader/Maps/", "ModLoader/Mods/", "BuildingBlocks/"]
                .iter()
                .any(|dir| rel.starts_with(dir))
            {
                return Err("方案的启停文件不在资源目录内".into());
            }
            let (from, to) = if enable {
                (root.join(format!("{rel}.disable")), root.join(rel))
            } else {
                (root.join(rel), root.join(format!("{rel}.disable")))
            };
            workspace::checked(root, &from)?;
            workspace::checked(root, &to)?;
            if !from.exists() {
                continue;
            }
            if to.exists() {
                return Err(format!("启停目标已有同名文件：{rel}"));
            }
            moves.push((from, to));
        }
    }
    let mut opts = config::options(root)?;
    if let Some(values) = profile["gameOptions"].as_object() {
        for (key, value) in values {
            if key != "levelLock" && key != "highscores" && opts.get(key).is_some() {
                opts[key] = value.clone();
            }
        }
    }
    changes.push((
        root.join("Database.tdb"),
        config::encoded_options(root, &opts)?,
    ));
    Ok(Plan { changes, moves })
}

#[derive(Clone, Debug)]
pub enum Action {
    Create(String),
    Rename(String, String),
    Delete(String),
    Switch(String),
}
pub fn change(root: &Path, action: Action) -> Result<Value, String> {
    workspace::idle(root)?;
    change_inner(root, action)
}
fn change_inner(root: &Path, action: Action) -> Result<Value, String> {
    let mut prepared = Plan::default();
    let mut index = load(root)?;
    let current = index["currentId"].as_str().unwrap_or("").to_owned();
    let mut list = index["profiles"]
        .as_array()
        .cloned()
        .ok_or("配置方案格式异常")?;
    match action {
        Action::Create(name) => {
            if name.trim().is_empty() {
                return Err("请输入方案名称".into());
            }
            let id = workspace::id();
            let mut snapshot = capture(root)?;
            if let Some(outgoing) = list.iter_mut().find(|p| p["id"] == current)
                && let Some(values) = snapshot.as_object()
            {
                for (key, value) in values {
                    outgoing[key] = value.clone();
                }
            }
            snapshot["id"] = json!(id);
            snapshot["name"] = json!(name.trim());
            snapshot["createdAt"] = json!(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64
            );
            list.push(snapshot);
            index["currentId"] = json!(id);
        }
        Action::Rename(id, name) => {
            if name.trim().is_empty() {
                return Err("请输入方案名称".into());
            }
            let p = list
                .iter_mut()
                .find(|p| p["id"] == id)
                .ok_or("配置方案已不存在")?;
            p["name"] = json!(name.trim());
        }
        Action::Delete(id) => {
            if id == current {
                return Err("请先切换到其他方案，再删除当前方案".into());
            }
            list.retain(|p| p["id"] != id);
        }
        Action::Switch(id) => {
            let target = list
                .iter()
                .find(|p| p["id"] == id)
                .cloned()
                .ok_or("配置方案已不存在")?;
            if id != current {
                if let Some(outgoing) = list.iter_mut().find(|p| p["id"] == current)
                    && let Some(snapshot) = capture(root)?.as_object()
                {
                    for (k, v) in snapshot {
                        outgoing[k] = v.clone();
                    }
                }
                prepared = plan(root, &target)?;
                index["currentId"] = json!(id);
            }
        }
    }
    index["profiles"] = json!(list);
    prepared.changes.push((
        root.join(".rockoon/profiles.json"),
        serde_json::to_vec_pretty(&index).map_err(error)?,
    ));
    for (i, (from, to)) in prepared.moves.iter().enumerate() {
        if let Err(e) = fs::rename(from, to) {
            for (from, to) in prepared.moves[..i].iter().rev() {
                let _ = fs::rename(to, from);
            }
            return Err(format!("切换文件状态失败，已撤回：{e}"));
        }
    }
    if let Err(e) = workspace::replace(root, &prepared.changes) {
        for (from, to) in prepared.moves.iter().rev() {
            let _ = fs::rename(to, from);
        }
        return Err(e);
    }
    Ok(index)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compatible_profiles_switch_options_and_mod_state_without_changing_scores() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(include_bytes!(
            "../../src-tauri/resources/vanilla/vanilla-core.zip"
        )))
        .unwrap();
        let mut bytes = Vec::new();
        std::io::Read::read_to_end(&mut archive.by_name("Database.tdb").unwrap(), &mut bytes)
            .unwrap();
        fs::write(root.join("Database.tdb"), bytes).unwrap();
        fs::create_dir_all(root.join("ModLoader/Mods")).unwrap();
        fs::create_dir_all(root.join("ModLoader/Configs")).unwrap();
        fs::write(root.join("ModLoader/Mods/Test.bmodp"), b"mod").unwrap();
        fs::write(
            root.join("ModLoader/Configs/Test.cfg"),
            "# retained\nGeneral {\n F Speed 1.5\n}\n",
        )
        .unwrap();
        let original = config::options(root).unwrap();
        let first = change_inner(root, Action::Create("first".into())).unwrap()["currentId"]
            .as_str()
            .unwrap()
            .to_owned();
        let second = change_inner(root, Action::Create("second".into())).unwrap()["currentId"]
            .as_str()
            .unwrap()
            .to_owned();
        fs::rename(
            root.join("ModLoader/Mods/Test.bmodp"),
            root.join("ModLoader/Mods/Test.bmodp.disable"),
        )
        .unwrap();
        let mut opts = original.clone();
        opts["volume"] = json!(0.25);
        config::save_options(root, &opts).unwrap();
        change_inner(root, Action::Switch(first)).unwrap();
        assert!(root.join("ModLoader/Mods/Test.bmodp").exists());
        let current = config::options(root).unwrap();
        assert_eq!(current["volume"], original["volume"]);
        assert_eq!(current["highscores"], original["highscores"]);
        assert!(
            fs::read_to_string(root.join("ModLoader/Configs/Test.cfg"))
                .unwrap()
                .contains("# retained")
        );
        change_inner(root, Action::Switch(second.clone())).unwrap();
        assert_eq!(config::options(root).unwrap()["volume"], json!(0.25));
        let mut opts = config::options(root).unwrap();
        opts["volume"] = json!(0.5);
        config::save_options(root, &opts).unwrap();
        change_inner(root, Action::Create("third".into())).unwrap();
        change_inner(root, Action::Switch(second)).unwrap();
        assert_eq!(config::options(root).unwrap()["volume"], json!(0.5));
    }
}
