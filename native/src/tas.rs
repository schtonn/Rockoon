use crate::workspace::{self, error};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub delta: f32,
    pub keys: u32,
}
impl Default for Frame {
    fn default() -> Self {
        Self {
            delta: 1000.0 / 240.0,
            keys: 0,
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct Project {
    pub frames: Vec<Frame>,
    pub path: Option<PathBuf>,
    pub dirty: bool,
    undo: Vec<Vec<Frame>>,
    redo: Vec<Vec<Frame>>,
    clipboard: Vec<Frame>,
}
pub const KEYS: [&str; 9] = ["↑", "↓", "←", "→", "Shift", "Space", "Q", "Esc", "Enter"];
impl Project {
    pub fn read(path: &Path) -> Result<Self, String> {
        let bytes = fs::read(path).map_err(error)?;
        let header: [u8; 4] = bytes
            .get(..4)
            .ok_or("TAS 文件过短")?
            .try_into()
            .map_err(error)?;
        let expected = u32::from_le_bytes(header) as usize;
        if !expected.is_multiple_of(8) || expected > 32 * 1024 * 1024 {
            return Err("TAS 长度无效或超过 32 MiB".into());
        }
        let mut raw = Vec::new();
        flate2::read::ZlibDecoder::new(&bytes[4..])
            .take(expected as u64 + 1)
            .read_to_end(&mut raw)
            .map_err(error)?;
        if raw.len() != expected {
            return Err("TAS 长度与文件头不一致".into());
        }
        let mut frames = Vec::with_capacity(expected / 8);
        for chunk in raw.as_chunks::<8>().0 {
            let delta = f32::from_le_bytes(chunk[..4].try_into().map_err(error)?);
            if !delta.is_finite() || delta <= 0.0 {
                return Err("TAS 帧时间无效".into());
            }
            frames.push(Frame {
                delta,
                keys: u32::from_le_bytes(chunk[4..].try_into().map_err(error)?),
            });
        }
        Ok(Self {
            frames,
            path: Some(path.to_owned()),
            ..Self::default()
        })
    }
    pub fn bytes(&self) -> Result<Vec<u8>, String> {
        let mut compressed =
            flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        for frame in &self.frames {
            compressed
                .write_all(&frame.delta.to_le_bytes())
                .map_err(error)?;
            compressed
                .write_all(&frame.keys.to_le_bytes())
                .map_err(error)?;
        }
        let mut bytes = ((self.frames.len() * 8) as u32).to_le_bytes().to_vec();
        bytes.extend(compressed.finish().map_err(error)?);
        Ok(bytes)
    }
    pub fn save(&self, path: &Path) -> Result<(), String> {
        workspace::atomic(path, &self.bytes()?)
    }
    fn checkpoint(&mut self) {
        let capacity = (32 * 1024 * 1024 / (self.frames.len().max(1) * 8)).clamp(1, 64);
        if self.undo.len() >= capacity {
            self.undo.remove(0);
        }
        self.undo.push(self.frames.clone());
        self.redo.clear();
        self.dirty = true;
    }
    pub fn toggle(&mut self, frame: usize, key: usize) {
        if frame < self.frames.len() && key < 9 {
            self.checkpoint();
            self.frames[frame].keys ^= 1 << key;
        }
    }
    pub fn range(
        &mut self,
        start: usize,
        end: usize,
        key: usize,
        enabled: bool,
    ) -> Result<(), String> {
        if start > end || end >= self.frames.len() || key >= 9 {
            return Err("选择的帧范围无效".into());
        }
        self.checkpoint();
        for frame in &mut self.frames[start..=end] {
            if enabled {
                frame.keys |= 1 << key
            } else {
                frame.keys &= !(1 << key)
            }
        }
        Ok(())
    }
    pub fn insert(&mut self, at: usize, count: usize) -> Result<(), String> {
        if at > self.frames.len() || count > 100000 || self.frames.len() + count > 4 * 1024 * 1024 {
            return Err("插入帧数超出范围".into());
        }
        self.checkpoint();
        self.frames.splice(at..at, vec![Frame::default(); count]);
        Ok(())
    }
    pub fn copy(&mut self, start: usize, end: usize) -> Result<(), String> {
        if start > end || end >= self.frames.len() {
            return Err("选择的帧范围无效".into());
        }
        self.clipboard = self.frames[start..=end].to_vec();
        Ok(())
    }
    pub fn cut(&mut self, start: usize, end: usize) -> Result<(), String> {
        self.copy(start, end)?;
        self.checkpoint();
        self.frames.drain(start..=end);
        Ok(())
    }
    pub fn paste(&mut self, at: usize) -> Result<(), String> {
        if at > self.frames.len() || self.frames.len() + self.clipboard.len() > 4 * 1024 * 1024 {
            return Err("粘贴位置超出范围".into());
        }
        self.checkpoint();
        self.frames.splice(at..at, self.clipboard.clone());
        Ok(())
    }
    pub fn undo(&mut self) {
        if let Some(frames) = self.undo.pop() {
            self.redo.push(std::mem::replace(&mut self.frames, frames));
            self.dirty = true;
        }
    }
    pub fn redo(&mut self) {
        if let Some(frames) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.frames, frames));
            self.dirty = true;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compressed_format_roundtrip_and_edit_history() {
        let mut project = Project::default();
        project.insert(0, 48).unwrap();
        project.range(4, 9, 0, true).unwrap();
        project.toggle(7, 2);
        let before = project.frames.clone();
        project.copy(4, 9).unwrap();
        project.paste(10).unwrap();
        project.undo();
        assert_eq!(project.frames, before);
        project.redo();
        assert_eq!(project.frames.len(), 54);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.tas");
        project.save(&path).unwrap();
        assert_eq!(Project::read(&path).unwrap().frames, project.frames);
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CfgChange {
    pub group: String,
    pub key: String,
    pub value: Option<String>,
    pub datatype: String,
}
fn patch_cfg(content: &str, changes: &[CfgChange]) -> String {
    let mut lines: Vec<String> = content.lines().map(str::to_owned).collect();
    let mut additions = Vec::new();
    for change in changes {
        let mut group = String::new();
        let mut found = None;
        let mut insert = None;
        for (i, line) in lines.iter().enumerate() {
            let t = line.trim();
            if let Some(cat) = t.strip_suffix('{') {
                group = cat.trim().into();
            } else if t == "}" {
                if group == change.group {
                    insert = Some(i);
                }
                group.clear();
            } else if group == change.group && !t.starts_with('#') {
                let mut parts = t.splitn(3, char::is_whitespace);
                let _ = parts.next();
                if parts.next() == Some(change.key.as_str()) {
                    found = Some(i);
                    break;
                }
            }
        }
        let new = change
            .value
            .as_ref()
            .map(|v| format!("\t{} {} {}", change.datatype, change.key, v));
        match (found, new) {
            (Some(i), Some(new)) => lines[i] = new,
            (Some(i), None) => {
                lines.remove(i);
            }
            (None, Some(new)) => {
                if let Some(i) = insert {
                    lines.insert(i, new);
                } else {
                    additions.push((change.group.clone(), new));
                }
            }
            (None, None) => {}
        }
    }
    let mut groups = std::collections::BTreeMap::<String, Vec<String>>::new();
    for (g, l) in additions {
        groups.entry(g).or_default().push(l);
    }
    for (g, added) in groups {
        lines.push(format!("{g} {{"));
        lines.extend(added);
        lines.push("}".into());
    }
    lines.join("\r\n") + "\r\n"
}
pub fn autoplay(root: &Path) -> Result<Option<String>, String> {
    let path = root.join("ModLoader/Configs/BallanceTAS.cfg");
    if !path.is_file() {
        return Ok(None);
    }
    let doc = crate::config::load(root, crate::config::Source::File(path))?;
    let on = |g: &str, k: &str| {
        doc.fields
            .iter()
            .any(|f| f.group == g && f.key == k && f.value == "1")
    };
    if on("TAS", "Enable") && on("Startup", "Enabled") && on("Startup", "AutoLoad") {
        Ok(Some(
            doc.fields
                .iter()
                .find(|f| f.group == "Startup" && f.key == "Project")
                .map(|f| f.value.clone())
                .unwrap_or_default(),
        ))
    } else {
        Ok(None)
    }
}
pub fn restore(root: &Path) -> Result<(), String> {
    let pending = root.join(".rockoon/native-tas-restore.json");
    if !pending.is_file() {
        return Ok(());
    }
    workspace::idle(root)?;
    let changes: Vec<CfgChange> =
        serde_json::from_slice(&fs::read(&pending).map_err(error)?).map_err(error)?;
    let path = root.join("ModLoader/Configs/BallanceTAS.cfg");
    let text = fs::read_to_string(&path).map_err(error)?;
    workspace::replace(root, &[(path, patch_cfg(&text, &changes).into_bytes())])?;
    fs::remove_file(pending).map_err(error)
}
pub fn prepare(root: &Path, project: &Project, level: u8, skip: usize) -> Result<(), String> {
    workspace::idle(root)?;
    restore(root)?;
    if !(1..=13).contains(&level) {
        return Err("TAS 关卡需要 1–13".into());
    }
    let path = root.join("ModLoader/Configs/BallanceTAS.cfg");
    let text = fs::read_to_string(&path)
        .map_err(|_| "请先安装 BallanceTAS 2.x 并运行一次游戏以生成配置".to_owned())?;
    let doc = crate::config::load(root, crate::config::Source::File(path.clone()))?;
    if !doc
        .fields
        .iter()
        .any(|f| f.group == "TAS" || f.group == "Startup")
    {
        return Err("BallanceTAS.cfg 格式不匹配".into());
    }
    let targets = [
        ("TAS", "Enable", "B", "1".into()),
        ("Startup", "Enabled", "B", "1".into()),
        ("Startup", "AutoLoad", "B", "1".into()),
        ("Startup", "Project", "S", "RockoonNativePreview".into()),
        ("Startup", "Level", "S", format!("Level_{level:02}")),
        ("TAS", "SkipRenderUntilFrame", "I", skip.to_string()),
    ];
    let mut desired = Vec::new();
    let mut restore = Vec::new();
    for (group, key, datatype, value) in targets {
        let old = doc
            .fields
            .iter()
            .find(|f| f.group == group && f.key == key)
            .map(|f| f.value.clone());
        desired.push(CfgChange {
            group: group.into(),
            key: key.into(),
            datatype: datatype.into(),
            value: Some(value),
        });
        restore.push(CfgChange {
            group: group.into(),
            key: key.into(),
            datatype: datatype.into(),
            value: old,
        });
    }
    workspace::replace(
        root,
        &[
            (
                root.join(".rockoon/native-tas-restore.json"),
                serde_json::to_vec_pretty(&restore).map_err(error)?,
            ),
            (
                root.join("ModLoader/TAS/RockoonNativePreview.tas"),
                project.bytes()?,
            ),
            (path, patch_cfg(&text, &desired).into_bytes()),
        ],
    )
}
pub fn disable_autoplay(root: &Path) -> Result<(), String> {
    workspace::idle(root)?;
    let path = root.join("ModLoader/Configs/BallanceTAS.cfg");
    let text = fs::read_to_string(&path).map_err(error)?;
    workspace::replace(
        root,
        &[(
            path,
            patch_cfg(
                &text,
                &[CfgChange {
                    group: "Startup".into(),
                    key: "AutoLoad".into(),
                    datatype: "B".into(),
                    value: Some("0".into()),
                }],
            )
            .into_bytes(),
        )],
    )
}
#[cfg(test)]
mod cfg_tests {
    use super::*;
    #[test]
    fn restores_only_modified_tas_entries() {
        let source =
            "# preserved\nTAS {\n B Enable 0\n I Other 7\n}\nStartup {\n B AutoLoad 0\n}\n";
        let changes = vec![
            CfgChange {
                group: "TAS".into(),
                key: "Enable".into(),
                datatype: "B".into(),
                value: Some("1".into()),
            },
            CfgChange {
                group: "Startup".into(),
                key: "Project".into(),
                datatype: "S".into(),
                value: Some("test project".into()),
            },
        ];
        let edited = patch_cfg(source, &changes);
        assert!(edited.contains("# preserved"));
        assert!(edited.contains("I Other 7"));
        assert!(edited.contains("S Project test project"));
        let restored = patch_cfg(
            &edited,
            &[
                CfgChange {
                    group: "TAS".into(),
                    key: "Enable".into(),
                    datatype: "B".into(),
                    value: Some("0".into()),
                },
                CfgChange {
                    group: "Startup".into(),
                    key: "Project".into(),
                    datatype: "S".into(),
                    value: None,
                },
            ],
        );
        assert!(!restored.contains("Project"));
        assert!(restored.contains("B Enable 0"));
        assert!(restored.contains("I Other 7"));
    }
}
