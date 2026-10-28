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
        "Driver" => "显示驱动",
        "BitsPerPixel" => "色深",
        "X" => "窗口横坐标",
        "Y" => "窗口纵坐标",
        "Debug" => "调试模式",
        "DisableDithering" => "禁用色彩抖动",
        "DisableSpecular" => "禁用镜面反射",
        "DisableMipmap" => "禁用多级纹理",
        "DisablePerspectiveCorrection" => "禁用透视校正",
        "ForceLinearFog" => "强制线性雾",
        "ForceSoftware" => "强制软件处理",
        "DisableFilter" => "禁用纹理过滤",
        "EnsureVertexShader" => "确保顶点着色器可用",
        "UseIndexBuffers" => "使用索引缓冲",
        "SortTransparentObjects" => "透明物体排序",
        "TextureCacheManagement" => "纹理缓存管理",
        "EnableDebugMode" => "启用调试",
        "EnableScreenDump" => "允许截图",
        _ => key,
    }
    .into()
}
fn builtin_config(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|s| s.to_str()),
        Some("BML.cfg" | "CameraUtilities.cfg" | "DebugUtilities.cfg")
    )
}
fn mod_label(path: &Path, key: &str) -> String {
    if !builtin_config(path) {
        return label(key);
    }
    match key {
        "FontFilename" => "字体文件",
        "FontSize" => "字号",
        "FontRanges" => "字形范围",
        "EnableSecondaryFont" => "使用备用字体",
        "SecondaryFontFilename" => "备用字体文件",
        "SecondaryFontSize" => "备用字号",
        "SecondaryFontRanges" => "备用字形范围",
        "EnableIniSettings" => "记住菜单布局",
        "ShowTitle" => "显示标题",
        "ShowFPS" => "显示帧率",
        "ShowSRTimer" => "显示速通计时",
        "FPSUpdateFrequency" => "帧率刷新间隔",
        "UnlockFrameRate" => "解除帧率限制",
        "SetMaxFrameRate" => "帧率上限",
        "WidescreenFix" => "宽屏适配",
        "LanternAlphaTest" => "优化灯笼透明效果",
        "FixLifeBallFreeze" => "修复拾取生命球卡顿",
        "Overclock" => "跳过出场等待",
        "MessageDuration" => "消息显示时长",
        "TabColumns" => "制表符宽度",
        "LineSpacing" => "消息行距",
        "MessageBackgroundAlpha" => "消息背景不透明度",
        "WindowBackgroundAlpha" => "命令窗口不透明度",
        "FadeMaxAlpha" => "淡入时最大不透明度",
        "LevelNumber" => "自制地图承载关卡",
        "ShowTooltip" => "显示地图信息",
        "MaxDepth" => "文件夹浏览深度",
        "Enable" => "启用相机工具",
        "Reset" => "重置相机",
        "Rotate45" => "切换为 45 度视角",
        "RotateLeft" => "相机左转",
        "RotateRight" => "相机右转",
        "MoveUp" => "向上移动",
        "MoveDown" => "向下移动",
        "MoveFront" => "向前移动",
        "MoveBack" => "向后移动",
        "MoveLeft" => "向左移动",
        "MoveRight" => "向右移动",
        "EnableSuicideKey" => "启用自杀快捷键",
        "Suicide" => "自杀",
        "BallUp" => "向上推球",
        "BallDown" => "向下推球",
        "TurnPaper" => "变成纸球",
        "TurnWood" => "变成木球",
        "TurnStone" => "变成石球",
        "ResetBall" => "重置球和机关",
        "AddLife" => "增加一条生命",
        "BallSpeedUp" => "三倍球速",
        "SpeedNotification" => "显示球速变化",
        "SkipRender" => "按住时跳过画面绘制",
        "PaperBall" => "生成纸球",
        "WoodBall" => "生成木球",
        "StoneBall" => "生成石球",
        "Box" => "生成箱子",
        _ => return label(key),
    }
    .into()
}
fn ini_boolean(key: &str, value: &str) -> bool {
    matches!(value.to_ascii_lowercase().as_str(), "true" | "false")
        || matches!(value, "0" | "1")
            && matches!(
                key,
                "FullScreen"
                    | "Borderless"
                    | "ClipCursor"
                    | "AlwaysHandleInput"
                    | "ChildWindowRendering"
                    | "SkipOpening"
                    | "UnlockFramerate"
                    | "UnlockWidescreen"
                    | "UnlockHighResolution"
                    | "ApplyHotfix"
                    | "Rookie"
                    | "Verbose"
                    | "ManualSetup"
                    | "Debug"
                    | "LogMode"
                    | "DisableDithering"
                    | "DisableSpecular"
                    | "DisableMipmap"
                    | "DisablePerspectiveCorrection"
                    | "ForceLinearFog"
                    | "ForceSoftware"
                    | "DisableFilter"
                    | "EnsureVertexShader"
                    | "UseIndexBuffers"
                    | "SortTransparentObjects"
                    | "TextureCacheManagement"
                    | "EnableDebugMode"
                    | "EnableScreenDump"
            )
}
impl Field {
    pub fn enabled(&self) -> bool {
        matches!(
            self.value.to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        )
    }
    pub fn toggled(&self, enabled: bool) -> String {
        if matches!(self.value.as_str(), "0" | "1") {
            if enabled { "1" } else { "0" }
        } else {
            if enabled { "true" } else { "false" }
        }
        .into()
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Choice(pub String);
impl std::fmt::Display for Choice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self.0.as_str() {
            "游戏设置" => "游戏设置",
            "BML.cfg" => "加载器设置",
            "CameraUtilities.cfg" => "相机工具",
            "DebugUtilities.cfg" => "调试工具",
            _ => &self.0,
        })
    }
}
pub fn group_label(group: &str) -> &str {
    match group {
        "GUI" => "游戏内菜单",
        "HUD" => "游戏内信息",
        "Graphics" => "渲染设置",
        "Tweak" => "辅助功能",
        "Misc" => "其他",
        "Camera" => "相机",
        "Debug" => "调试",
        "Auxiliaries" => "临时辅助物体",
        "General" | "Settings" | "Game" => "常规",
        "Window" => "窗口",
        "CommandBar" => "命令与消息",
        "CustomMap" => "自制地图",
        _ => group,
    }
}
impl Document {
    pub fn field_help(&self, field: &Field) -> Option<String> {
        if self.source == Source::Game {
            return self.parts.iter().find_map(|part| {
                part.fields
                    .iter()
                    .find(|f| f.key == field.key)
                    .and_then(|f| part.field_help(f))
            });
        }
        if let Source::File(path) = &self.source
            && builtin_config(path)
        {
            return match field.key.as_str() {
                "Enable" if field.group == "Camera" => Some("仅在游戏作弊模式下生效。"),
                "SetMaxFrameRate" => Some("设为 0 使用垂直同步，解除帧率限制后此项无效。"),
                "FontRanges" | "SecondaryFontRanges" => {
                    Some("显示中文请选择 Chinese 或 ChineseFull。")
                }
                "LevelNumber" => Some("设为 0 随机借用第 2–11 关，否则使用指定关卡。"),
                "MaxDepth" => Some("限制地图菜单可以进入的文件夹层数。"),
                "Overclock" => Some("跳过开局和复活时等待球落下的过程。"),
                "LanternAlphaTest" => Some("改变灯笼的透明绘制方式来提高帧率。"),
                "FPSUpdateFrequency" => Some("每隔指定帧数更新一次帧率数值。"),
                _ => None,
            }
            .map(str::to_owned);
        }
        let end = field.line?;
        let mut comments: Vec<_> = self
            .lines
            .get(..end)?
            .iter()
            .rev()
            .skip_while(|s| s.trim().is_empty())
            .take_while(|s| s.trim().starts_with(['#', ';']))
            .map(|s| s.trim().trim_start_matches(['#', ';']).trim())
            .collect();
        comments.reverse();
        let text = comments.join(" ");
        let text = text
            .split(". ")
            .next()
            .unwrap_or("")
            .split('。')
            .next()
            .unwrap_or("")
            .replace('；', "，")
            .replace(';', ",");
        (!text.is_empty()).then_some(text)
    }
}
pub fn source_help(name: &str) -> &'static str {
    match name {
        "游戏设置" => "包含原版游戏设置和 NewPlayer 的窗口与兼容参数。",
        "BML.cfg" => "调整加载器的菜单、游戏内信息和帧率等功能。",
        "CameraUtilities.cfg" => "配置相机工具的移动、旋转和重置功能。",
        "DebugUtilities.cfg" => "配置调试工具的重置球、切换球种等功能。",
        _ => "这是所选模组的独立配置。",
    }
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
                                    "第 {} 名{}",
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
            let content = fs::read_to_string(&path).map_err(error)?;
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
                            ini_boolean(key, v),
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
                    label: mod_label(&path, &key),
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
    fn builtin_cfg_labels_are_chinese_but_saved_keys_stay_original() {
        let root = tempfile::tempdir().unwrap();
        for (file, content, expected) in [
            (
                "BML.cfg",
                "HUD {\n # FPS counter update frequency in frames\n I FPSUpdateFrequency 30\n}\n",
                "帧率刷新间隔",
            ),
            (
                "CameraUtilities.cfg",
                "Camera {\n B Enable 0\n}\n",
                "启用相机工具",
            ),
            (
                "DebugUtilities.cfg",
                "Debug {\n K TurnPaper 23\n}\n",
                "变成纸球",
            ),
        ] {
            let path = root.path().join(file);
            fs::write(&path, content).unwrap();
            let mut doc = load(root.path(), Source::File(path.clone())).unwrap();
            assert_eq!(doc.fields[0].label, expected);
            doc.fields[0].value = "1".into();
            doc.dirty = true;
            doc.save(root.path()).unwrap();
            let written = fs::read_to_string(path).unwrap();
            assert!(written.contains(&doc.fields[0].key));
            assert!(!written.contains(expected));
        }
        let path = root.path().join("ThirdParty.cfg");
        fs::write(&path, "General {\n B Enable 0\n}\n").unwrap();
        let doc = load(root.path(), Source::File(path)).unwrap();
        assert_eq!(doc.fields[0].label, "Enable");
    }
    #[test]
    fn editing_one_level_preserves_other_level_rankings() {
        let fixture = game_fixture();
        let before = options(fixture.path()).unwrap();
        let mut doc = load(fixture.path(), Source::Data).unwrap();
        doc.fields
            .iter_mut()
            .find(|f| f.key == "/highscores/3/0/player")
            .unwrap()
            .value = "Native test".into();
        doc.fields
            .iter_mut()
            .find(|f| f.key == "/highscores/3/0/score")
            .unwrap()
            .value = "4321".into();
        doc.fields
            .iter_mut()
            .find(|f| f.key == "/levelLock/3")
            .unwrap()
            .value = "true".into();
        doc.save(fixture.path()).unwrap();
        let after = options(fixture.path()).unwrap();
        assert_eq!(after["highscores"][3][0]["player"], "Native test");
        assert_eq!(after["highscores"][3][0]["score"], 4321);
        assert_eq!(after["levelLock"][3], true);
        for level in 0..13 {
            if level != 3 {
                assert_eq!(after["highscores"][level], before["highscores"][level]);
            }
        }
    }
    #[test]
    fn merged_game_fields_read_help_from_their_own_source() {
        let fixture = game_fixture();
        fs::write(
            fixture.path().join("Bin/Player.ini"),
            "[Game]\n; Only affects startup\nSkipOpening=1\n",
        )
        .unwrap();
        let doc = load(fixture.path(), Source::Game).unwrap();
        for field in &doc.fields {
            let _ = doc.field_help(field);
        }
        let field = doc.fields.iter().find(|f| f.key == "SkipOpening").unwrap();
        assert_eq!(
            doc.field_help(field).as_deref(),
            Some("Only affects startup")
        );
    }
    #[test]
    fn numeric_ini_flags_keep_encoding_and_preserve_numeric_parameters() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("Player.ini");
        fs::write(&path, "[Graphics]\nFullScreen=0\nWidth=1\nAntialias=0\n[Game]\nLanguage=1\nSkipOpening=1\n[Startup]\nLogMode=1\n").unwrap();
        let mut doc = load(root.path(), Source::File(path.clone())).unwrap();
        for field in &mut doc.fields {
            if ["FullScreen", "SkipOpening", "LogMode"].contains(&field.key.as_str()) {
                assert!(field.boolean, "{} should be a switch", field.key);
                field.value = field.toggled(!field.enabled());
            } else {
                assert!(!field.boolean, "{} must remain numeric", field.key);
            }
        }
        doc.save(root.path()).unwrap();
        let saved = fs::read_to_string(path).unwrap();
        assert!(
            saved.contains("FullScreen=1")
                && saved.contains("SkipOpening=0")
                && saved.contains("LogMode=0")
        );
        assert!(
            saved.contains("Width=1")
                && saved.contains("Language=1")
                && saved.contains("Antialias=0")
        );
    }
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
