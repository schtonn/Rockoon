use crate::workspace::{self, error};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Source {
    Game,
    Options,
    Data,
    File(PathBuf),
}
#[derive(Clone, Debug)]
pub struct Field {
    pub key: String,
    pub label: String,
    pub group: String,
    pub value: String,
    pub boolean: bool,
    pub number: bool,
    pub line: Option<usize>,
    pub prefix: String,
}
#[derive(Clone, Debug)]
pub struct Document {
    pub source: Source,
    pub fields: Vec<Field>,
    pub original: Value,
    pub lines: Vec<String>,
    pub dirty: bool,
    parts: Vec<Document>,
}
pub fn options(root: &Path) -> Result<Value, String> {
    let mut opts = rockoon_core::options::BallanceOptions::new();
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        opts.read_from(&root.join("Database.tdb").to_string_lossy())
    }))
    .map_err(|_| "Database.tdb 格式损坏，未改写文件".to_owned())?
    .map_err(error)?;
    serde_json::to_value(opts).map_err(error)
}
pub fn encoded_options(root: &Path, value: &Value) -> Result<Vec<u8>, String> {
    let mut opts: rockoon_core::options::BallanceOptions =
        serde_json::from_value(value.clone()).map_err(error)?;
    let temp = tempfile::tempdir().map_err(error)?;
    let path = temp.path().join("Database.tdb");
    opts.write_to(&path.to_string_lossy()).map_err(error)?;
    // Merge known columns into a copy of the original DB, preserving unrelated tables.
    use rockoon_core::tdb::{Tdb, VtColumn, VtValue, VtValueType};
    let mut fresh = Tdb::new(&path.to_string_lossy());
    fresh.load().map_err(error)?;
    let preserved = temp.path().join("preserved.tdb");
    fs::copy(root.join("Database.tdb"), &preserved).map_err(error)?;
    let mut original = Tdb::new(&preserved.to_string_lossy());
    original.load().map_err(error)?;
    let names = std::iter::once("DB_Options".to_owned())
        .chain(std::iter::once("DB_Levelfreischaltung".to_owned()))
        .chain((1..=13).map(|i| format!("DB_Highscore_Lv{i:02}")));
    for name in names {
        let generated = fresh.get_table(&name).map_err(error)?;
        let existing = original.get_table_mut(&name);
        for (i, col) in generated.cols.iter().enumerate() {
            let values = col
                .values
                .iter()
                .map(|v| VtValue::new(v.get_slice()))
                .collect();
            let new = VtColumn {
                name: col.name.clone(),
                value_type: VtValueType::new(col.value_type.to_int()),
                values,
            };
            if i < existing.cols.len() {
                existing.cols[i] = new;
            } else {
                existing.cols.push(new);
            }
        }
    }
    original.dump().map_err(error)?;
    fs::read(preserved).map_err(error)
}
#[cfg(test)]
pub fn save_options(root: &Path, value: &Value) -> Result<(), String> {
    workspace::replace(
        root,
        &[(root.join("Database.tdb"), encoded_options(root, value)?)],
    )
}

fn label(key: &str) -> String {
    match key {
        "volume" => "音量（0–1）",
        "syncToScreen" => "垂直同步",
        "keyForward" => "前进",
        "keyBackward" => "后退",
        "keyLeft" => "左转",
        "keyRight" => "右转",
        "keyRotateCam" => "旋转相机",
        "keyLiftCam" => "抬升相机",
        "invertCamRotation" => "反转相机",
        "cloudLayer" => "云层",
        "lastPlayer" => "玩家名",
        "Width" => "宽度",
        "Height" => "高度",
        "FullScreen" => "全屏",
        "Borderless" => "无边框",
        "ClipCursor" => "限制鼠标范围",
        "AlwaysHandleInput" => "后台接收输入",
        "ChildWindowRendering" => "子窗口渲染",
        "Language" => "语言",
        "SkipOpening" => "跳过开场",
        "UnlockFramerate" => "解锁帧率",
        "UnlockWidescreen" => "解锁宽屏",
        "UnlockHighResolution" => "解锁高分辨率",
        "ApplyHotfix" => "兼容修复",
        "Rookie" => "新手模式",
        "Antialias" => "抗锯齿",
        "LogMode" => "日志",
        "Verbose" => "详细日志",
        "ManualSetup" => "手动设置",
        _ => key,
    }
    .into()
}
pub fn load(root: &Path, source: Source) -> Result<Document, String> {
    let mut doc = Document {
        source: source.clone(),
        fields: vec![],
        original: Value::Null,
        lines: vec![],
        dirty: false,
        parts: vec![],
    };
    match source {
        Source::Game => {
            let mut parts = vec![load(root, Source::Options)?];
            let player = root.join("Bin/Player.ini");
            if files(root)?.contains(&player) {
                parts.push(load(root, Source::File(player))?);
            }
            doc.fields = parts
                .iter()
                .flat_map(|part| part.fields.iter().cloned())
                .map(|mut field| {
                    field.group = game_group(&field).into();
                    field
                })
                .collect();
            doc.parts = parts;
        }
        Source::Options | Source::Data => {
            doc.original = options(root)?;
            if source == Source::Options {
                for (key, v) in doc.original.as_object().ok_or("配置格式异常")? {
                    if key == "levelLock" || key == "highscores" {
                        continue;
                    }
                    doc.fields.push(Field {
                        key: format!("/{key}"),
                        label: label(key),
                        group: "游戏".into(),
                        value: if key == "volume" {
                            ((v.as_f64().unwrap_or(0.0) * 100.0).round() / 100.0).to_string()
                        } else if v.is_string() {
                            v.as_str().unwrap_or_default().into()
                        } else {
                            v.to_string()
                        },
                        boolean: v.is_boolean(),
                        number: v.is_number(),
                        line: None,
                        prefix: String::new(),
                    });
                }
            } else {
                for i in 0..12 {
                    doc.fields.push(Field {
                        key: format!("/levelLock/{i}"),
                        label: format!("第 {} 关已解锁", i + 1),
                        group: "关卡".into(),
                        value: doc.original["levelLock"][i].to_string(),
                        boolean: true,
                        number: false,
                        line: None,
                        prefix: String::new(),
                    });
                }
                for level in 0..13 {
                    for rank in 0..10 {
                        for key in ["player", "score"] {
                            let v = &doc.original["highscores"][level][rank][key];
                            doc.fields.push(Field {
                                key: format!("/highscores/{level}/{rank}/{key}"),
                                label: format!(
                                    "第 {} 名 · {}",
                                    rank + 1,
                                    if key == "player" { "玩家" } else { "分数" }
                                ),
                                group: format!("第 {} 关", level + 1),
                                value: if key == "player" {
                                    v.as_str().unwrap_or_default().into()
                                } else {
                                    v.to_string()
                                },
                                boolean: false,
                                number: key == "score",
                                line: None,
                                prefix: String::new(),
                            });
                        }
                    }
                }
            }
        }
        Source::File(ref path) => {
            let path = workspace::checked(root, path)?;
            let content = fs::read_to_string(path).map_err(error)?;
            doc.lines = content.lines().map(str::to_owned).collect();
            let mut group = "常规".to_owned();
            for (i, line) in doc.lines.iter().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with(['#', ';']) || trimmed.is_empty() {
                    continue;
                }
                if trimmed.starts_with('[') && trimmed.ends_with(']') {
                    group = trimmed[1..trimmed.len() - 1].to_owned();
                    continue;
                }
                if let Some(cat) = trimmed.strip_suffix('{') {
                    group = cat.trim().to_owned();
                    continue;
                }
                if trimmed == "}" {
                    continue;
                }
                let (key, value, prefix, boolean, number) =
                    if let Some((key, value)) = line.split_once('=') {
                        let key = key.trim();
                        let v = value.trim();
                        (
                            key.to_owned(),
                            v.to_owned(),
                            format!("{}=", line.split_once('=').unwrap().0),
                            matches!(v.to_lowercase().as_str(), "true" | "false"),
                            v.parse::<f64>().is_ok(),
                        )
                    } else {
                        let mut parts = trimmed.splitn(3, char::is_whitespace);
                        let Some(kind) = parts.next() else { continue };
                        let Some(key) = parts.next() else { continue };
                        let Some(value) = parts.next() else { continue };
                        if !["B", "I", "F", "S", "K"].contains(&kind) {
                            continue;
                        }
                        (
                            key.into(),
                            value.trim().into(),
                            format!("\t{kind} {key} "),
                            kind == "B",
                            ["I", "F", "K"].contains(&kind),
                        )
                    };
                doc.fields.push(Field {
                    label: label(&key),
                    key,
                    group: group.clone(),
                    value,
                    boolean,
                    number,
                    line: Some(i),
                    prefix,
                });
            }
        }
    }
    Ok(doc)
}
fn game_group(field: &Field) -> &'static str {
    match field.key.trim_start_matches('/') {
        key if key.starts_with("key") || key == "invertCamRotation" => "键位与相机",
        "volume" => "声音",
        "cloudLayer"
        | "syncToScreen"
        | "Width"
        | "Height"
        | "FullScreen"
        | "Borderless"
        | "UnlockFramerate"
        | "UnlockWidescreen"
        | "UnlockHighResolution"
        | "Antialias" => "画面",
        "lastPlayer" | "Language" | "SkipOpening" | "Rookie" | "ClipCursor" => "常规",
        _ => "高级",
    }
}

impl Document {
    pub fn groups(&self) -> Vec<String> {
        let mut groups = Vec::new();
        for f in &self.fields {
            if !groups.contains(&f.group) {
                groups.push(f.group.clone());
            }
        }
        if self.source == Source::Game {
            ["常规", "画面", "声音", "键位与相机", "高级"]
                .into_iter()
                .filter(|name| groups.iter().any(|group| group == name))
                .map(str::to_owned)
                .collect()
        } else {
            groups
        }
    }
    pub fn value(&self) -> Result<Value, String> {
        let mut value = self.original.clone();
        for f in &self.fields {
            let v = if f.boolean {
                Value::Bool(matches!(
                    f.value.to_lowercase().as_str(),
                    "1" | "true" | "yes"
                ))
            } else if f.number {
                let parsed: Value = serde_json::from_str(&f.value)
                    .map_err(|_| format!("{} 需要有效数字", f.label))?;
                if !parsed.is_number() {
                    return Err(format!("{} 需要数字", f.label));
                }
                parsed
            } else {
                Value::String(f.value.clone())
            };
            if let Some(dest) = value.pointer_mut(&f.key) {
                *dest = v;
            }
        }
        if let Some(volume) = value["volume"].as_f64()
            && !(0.0..=1.0).contains(&volume)
        {
            return Err("音量必须在 0–1 之间".into());
        }
        Ok(value)
    }
    pub fn save(&self, root: &Path) -> Result<(), String> {
        workspace::idle(root)?;
        workspace::replace(root, &self.changes(root)?)
    }
    /// Validate and encode every source before the single write transaction begins.
    fn changes(&self, root: &Path) -> Result<Vec<(PathBuf, Vec<u8>)>, String> {
        match &self.source {
            Source::Game => {
                let mut changes = Vec::new();
                let mut offset = 0;
                for original in &self.parts {
                    let end = offset + original.fields.len();
                    let fields = self
                        .fields
                        .get(offset..end)
                        .ok_or("配置字段不完整，请重新读取")?;
                    if fields
                        .iter()
                        .zip(&original.fields)
                        .any(|(a, b)| a.value != b.value)
                    {
                        let mut edited = original.clone();
                        edited.fields = fields.to_vec();
                        changes.extend(edited.changes(root)?);
                    }
                    offset = end;
                }
                Ok(changes)
            }
            Source::Options | Source::Data => {
                let edited = self.value()?;
                let mut current = options(root)?;
                for field in &self.fields {
                    let before = self.original.pointer(&field.key);
                    let after = edited.pointer(&field.key);
                    let same_number = before
                        .and_then(Value::as_f64)
                        .zip(after.and_then(Value::as_f64))
                        .is_some_and(|(a, b)| (a - b).abs() < 0.000001);
                    if before != after && !same_number {
                        if current.pointer(&field.key) != before {
                            return Err(format!(
                                "{} 已由其他操作修改，请重新读取配置",
                                field.label
                            ));
                        }
                        if let (Some(dest), Some(value)) = (current.pointer_mut(&field.key), after)
                        {
                            *dest = value.clone();
                        }
                    }
                }
                Ok(vec![(
                    root.join("Database.tdb"),
                    encoded_options(root, &current)?,
                )])
            }
            Source::File(path) => {
                if fs::read_to_string(path)
                    .map_err(error)?
                    .lines()
                    .collect::<Vec<_>>()
                    != self.lines.iter().map(String::as_str).collect::<Vec<_>>()
                {
                    return Err("配置已由其他操作修改，请重新读取后编辑".into());
                }
                let mut lines = self.lines.clone();
                for f in &self.fields {
                    if f.value.contains(['\n', '\r']) {
                        return Err("配置值不能包含换行".into());
                    }
                    if f.number && f.value.parse::<f64>().is_err() {
                        return Err(format!("{} 需要数字", f.label));
                    }
                    if let Some(i) = f.line {
                        lines[i] = format!("{}{}", f.prefix, f.value);
                    }
                }
                Ok(vec![(
                    workspace::checked(root, path)?,
                    (lines.join("\r\n") + "\r\n").into_bytes(),
                )])
            }
        }
    }
}
pub fn files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut result = Vec::new();
    if root.join("Bin/Player.ini").is_file()
        && fs::metadata(root.join("Bin/Player.exe")).is_ok_and(|m| m.len() != 155648)
    {
        result.push(root.join("Bin/Player.ini"));
    }
    let folder = root.join("ModLoader/Configs");
    if folder.is_dir() {
        for entry in fs::read_dir(folder).map_err(error)? {
            let path = entry.map_err(error)?.path();
            if path.is_file()
                && path
                    .extension()
                    .is_some_and(|x| x.eq_ignore_ascii_case("cfg") || x.eq_ignore_ascii_case("ini"))
                && workspace::checked(root, &path).is_ok()
            {
                result.push(path);
            }
        }
    }
    result.sort();
    Ok(result)
}
pub fn ini_json(content: &str) -> Value {
    let mut result = json!({});
    let mut section = String::new();
    for line in content.lines() {
        let t = line.trim();
        if t.starts_with([';', '#']) {
            continue;
        }
        if t.starts_with('[') && t.ends_with(']') {
            section = t[1..t.len() - 1].into();
            result[&section] = json!({});
        } else if let Some((k, v)) = t.split_once('=') {
            result[&section][k.trim()] = json!(v.trim());
        }
    }
    result
}
pub fn ini_text(value: &Value) -> String {
    let mut text = String::new();
    if let Some(sections) = value.as_object() {
        for (name, fields) in sections {
            text.push_str(&format!("[{name}]\r\n"));
            if let Some(fields) = fields.as_object() {
                for (key, value) in fields {
                    text.push_str(&format!(
                        "{key}={}\r\n",
                        value
                            .as_str()
                            .map(str::to_owned)
                            .unwrap_or_else(|| value.to_string())
                    ));
                }
            }
        }
    }
    text
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_database_edit_preserves_unrelated_tables_and_scores() {
        use rockoon_core::tdb::{Tdb, VtColumn, VtValue, VtValueType};
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
        let mut db = Tdb::new(&root.join("Database.tdb").to_string_lossy());
        db.load().unwrap();
        db.get_table_mut("NativeExtra").cols.push(VtColumn {
            name: "keep".into(),
            value_type: VtValueType::Int,
            values: vec![VtValue::new_int(4242)],
        });
        db.dump().unwrap();
        let original = options(root).unwrap();
        let mut edited = original.clone();
        edited["volume"] = json!(0.35);
        save_options(root, &edited).unwrap();
        let actual = options(root).unwrap();
        assert_eq!(actual["highscores"], original["highscores"]);
        assert_eq!(actual["levelLock"], original["levelLock"]);
        assert!((actual["volume"].as_f64().unwrap() - 0.35).abs() < 0.001);
        db.load().unwrap();
        assert_eq!(
            db.get_table("NativeExtra").unwrap().cols[0].values[0].to_int(),
            4242
        );
    }
    fn game_fixture() -> tempfile::TempDir {
        use std::io::Read;
        let dir = tempfile::tempdir().unwrap();
        let mut archive = zip::ZipArchive::new(std::io::Cursor::new(include_bytes!(
            "../../src-tauri/resources/vanilla/vanilla-core.zip"
        )))
        .unwrap();
        let mut bytes = Vec::new();
        archive
            .by_name("Database.tdb")
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        fs::write(dir.path().join("Database.tdb"), bytes).unwrap();
        fs::create_dir(dir.path().join("Bin")).unwrap();
        fs::write(dir.path().join("Bin/Player.exe"), b"test only").unwrap();
        fs::write(dir.path().join("Bin/Player.ini"), "; keep this comment\n[Graphics]\nWidth=1024\nFullScreen=false\n[Game]\nSkipOpening=true\nCustomValue=keep-me\n").unwrap();
        dir
    }
    fn edit(doc: &mut Document, key: &str, value: &str) {
        doc.fields
            .iter_mut()
            .find(|field| field.key == key)
            .unwrap()
            .value = value.into();
        doc.dirty = true;
    }
    #[test]
    fn game_settings_save_both_files_and_preserve_unedited_content() {
        let root = game_fixture();
        let mut doc = load(root.path(), Source::Game).unwrap();
        assert!(doc.groups().contains(&"画面".to_owned()));
        assert!(
            doc.fields
                .iter()
                .any(|f| f.key == "/keyForward" && f.group == "键位与相机")
        );
        edit(&mut doc, "/volume", "0.25");
        edit(&mut doc, "Width", "1280");
        let scores = options(root.path()).unwrap()["highscores"].clone();
        doc.save(root.path()).unwrap();
        let actual = options(root.path()).unwrap();
        assert!((actual["volume"].as_f64().unwrap() - 0.25).abs() < 0.001);
        assert_eq!(actual["highscores"], scores);
        let ini = fs::read_to_string(root.path().join("Bin/Player.ini")).unwrap();
        assert!(
            ini.contains("Width=1280")
                && ini.contains("; keep this comment")
                && ini.contains("CustomValue=keep-me")
        );
    }
    #[test]
    fn invalid_second_file_cannot_partially_save_the_first() {
        let root = game_fixture();
        let before_db = fs::read(root.path().join("Database.tdb")).unwrap();
        let before_ini = fs::read(root.path().join("Bin/Player.ini")).unwrap();
        let mut doc = load(root.path(), Source::Game).unwrap();
        edit(&mut doc, "/volume", "0.1");
        edit(&mut doc, "Width", "invalid");
        assert!(doc.save(root.path()).is_err());
        assert_eq!(
            fs::read(root.path().join("Database.tdb")).unwrap(),
            before_db
        );
        assert_eq!(
            fs::read(root.path().join("Bin/Player.ini")).unwrap(),
            before_ini
        );
    }
    #[test]
    fn unedited_player_settings_do_not_overwrite_external_changes() {
        let root = game_fixture();
        let mut doc = load(root.path(), Source::Game).unwrap();
        edit(&mut doc, "/volume", "0.4");
        let path = root.path().join("Bin/Player.ini");
        let external = "; externally updated\n[Graphics]\nWidth=1600\n";
        fs::write(&path, external).unwrap();
        doc.save(root.path()).unwrap();
        assert_eq!(fs::read_to_string(path).unwrap(), external);
    }
    #[test]
    fn original_player_does_not_expose_unused_ini_settings() {
        let root = game_fixture();
        fs::write(root.path().join("Bin/Player.exe"), vec![0u8; 155648]).unwrap();
        let doc = load(root.path(), Source::Game).unwrap();
        assert!(!doc.fields.iter().any(|f| f.key == "Width"));
    }
    #[test]
    fn ini_roundtrip_keeps_values() {
        let v = ini_json("[Graphics]\nWidth=1920\nFullScreen=true\n");
        assert_eq!(ini_json(&ini_text(&v)), v);
    }
    #[test]
    fn cfg_edits_preserve_comments_and_space_in_strings() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("test.cfg");
        fs::write(
            &path,
            "# comment\nGeneral {\n S Name two words\n B Enabled 1\n}\n",
        )
        .unwrap();
        let doc = load(root.path(), Source::File(path)).unwrap();
        assert_eq!(doc.fields[0].value, "two words");
        assert!(doc.fields[1].boolean);
    }
}
