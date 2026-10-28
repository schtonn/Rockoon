//! Native client for the resource site's public, read-only catalogue.
use crate::{jobs, preferences, workspace};
use iced::widget::{image, markdown};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    io::Read,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
    time::Duration,
};
pub const PAGE_SIZE: usize = 12;
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Category {
    #[default]
    All,
    Map,
    Mod,
    Sky,
    Texture,
    Sound,
    Patch,
}
impl Category {
    pub fn from_kind(kind: workspace::Kind) -> Self {
        match kind {
            workspace::Kind::Maps => Self::Map,
            workspace::Kind::Mods => Self::Mod,
            workspace::Kind::Skies => Self::Sky,
            workspace::Kind::Textures => Self::Texture,
            workspace::Kind::Sounds => Self::Sound,
        }
    }
    pub fn local_kind(self) -> Option<workspace::Kind> {
        match self {
            Self::Map => Some(workspace::Kind::Maps),
            Self::Mod => Some(workspace::Kind::Mods),
            Self::Sky => Some(workspace::Kind::Skies),
            Self::Texture => Some(workspace::Kind::Textures),
            Self::Sound => Some(workspace::Kind::Sounds),
            _ => None,
        }
    }
    pub const ALL: [Self; 7] = [
        Self::All,
        Self::Map,
        Self::Mod,
        Self::Sky,
        Self::Texture,
        Self::Sound,
        Self::Patch,
    ];
    pub fn key(self) -> &'static str {
        match self {
            Self::All => "",
            Self::Map => "map",
            Self::Mod => "mod",
            Self::Sky => "sky",
            Self::Texture => "texture",
            Self::Sound => "sound",
            Self::Patch => "x-patch",
        }
    }
}
impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::All => "全部",
            Self::Map => "地图",
            Self::Mod => "模组",
            Self::Sky => "天空盒",
            Self::Texture => "纹理",
            Self::Sound => "音效",
            Self::Patch => "组件与补丁",
        })
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Sort {
    #[default]
    Newest,
    Oldest,
    Name,
}
impl Sort {
    pub const ALL: [Self; 3] = [Self::Newest, Self::Oldest, Self::Name];
    fn key(self) -> &'static str {
        match self {
            Self::Newest => "newest",
            Self::Oldest => "oldest",
            Self::Name => "name",
        }
    }
}
impl std::fmt::Display for Sort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Newest => "最新发布",
            Self::Oldest => "最早发布",
            Self::Name => "按名称",
        })
    }
}
#[derive(Clone, Debug, Default)]
pub struct Query {
    pub text: String,
    pub category: Category,
    pub sort: Sort,
    pub page: usize,
}
impl Query {
    pub fn path(&self) -> String {
        let mut q = url::form_urlencoded::Serializer::new(String::new());
        q.append_pair("limit", &PAGE_SIZE.to_string())
            .append_pair("skip", &self.page.saturating_mul(PAGE_SIZE).to_string())
            .append_pair("sort", self.sort.key());
        if !self.text.trim().is_empty() {
            q.append_pair("q", self.text.trim());
        }
        if !self.category.key().is_empty() {
            q.append_pair("category", self.category.key());
        }
        format!("/packages?{}", q.finish())
    }
}
#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
pub enum Author {
    Name(String),
    Display {
        #[serde(default)]
        name: String,
        #[serde(default)]
        display: String,
    },
}
impl Author {
    pub fn name(&self) -> &str {
        match self {
            Self::Name(s) => s,
            Self::Display { name, display } => {
                if display.is_empty() {
                    name
                } else {
                    display
                }
            }
        }
    }
}
#[derive(Clone, Debug, Default, Deserialize)]
pub struct Package {
    pub id: u64,
    #[serde(default)]
    pub name: Option<String>,
    pub category: String,
    #[serde(default)]
    pub authors: Vec<Author>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub preview_images: Vec<String>,
    #[serde(default)]
    pub file_size: u64,
    #[serde(default)]
    pub mod_formats: Vec<String>,
    #[serde(default)]
    pub updated_at: String,
    #[serde(default)]
    pub source_key: String,
}
impl Package {
    pub fn title(&self) -> &str {
        self.name
            .as_deref()
            .filter(|n| !n.trim().is_empty())
            .unwrap_or("未命名资源")
    }
    pub fn authors(&self) -> String {
        self.authors
            .iter()
            .map(Author::name)
            .filter(|n| !n.is_empty())
            .collect::<Vec<_>>()
            .join("、")
    }
    pub fn managed_component(&self) -> Option<&str> {
        matches!(
            self.source_key.as_str(),
            "player-vc6" | "player-msvc2022" | "bml" | "bmlplus"
        )
        .then_some(self.source_key.as_str())
    }
    pub fn cover(&self) -> Option<&str> {
        self.preview_images.first().map(String::as_str)
    }
}
#[derive(Clone, Debug, Deserialize)]
pub struct Listing {
    pub total: usize,
    pub items: Vec<Package>,
}
pub fn list(query: &Query) -> Result<Listing, String> {
    let mut result: Listing =
        serde_json::from_value(jobs::hub(&query.path())?).map_err(workspace::error)?;
    result.items.truncate(PAGE_SIZE);
    Ok(result)
}
pub fn package(id: u64) -> Result<Package, String> {
    serde_json::from_value(jobs::hub(&format!("/packages/{id}"))?).map_err(workspace::error)
}
#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct Release {
    pub version: u64,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub is_current: bool,
}
impl std::fmt::Display for Release {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "v{}{}",
            self.version,
            self.note
                .as_ref()
                .filter(|s| !s.is_empty())
                .map(|s| format!(" · {s}"))
                .unwrap_or_default()
        )
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum Choice {
    #[default]
    Latest,
    Version(Release),
}
impl Choice {
    pub fn version(&self) -> Option<u64> {
        match self {
            Self::Latest => None,
            Self::Version(v) => Some(v.version),
        }
    }
    pub fn current(&self) -> bool {
        match self {
            Self::Latest => true,
            Self::Version(v) => v.is_current,
        }
    }
}
impl std::fmt::Display for Choice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Latest => f.write_str("最新版本"),
            Self::Version(v) => v.fmt(f),
        }
    }
}
#[derive(Clone, Debug)]
pub struct Detail {
    pub package: Package,
    pub versions: Vec<Release>,
    pub version_error: Option<String>,
    pub dependencies: Vec<(u64, String)>,
}
pub fn detail(id: u64) -> Result<Detail, String> {
    let package = package(id)?;
    let versions = jobs::hub(&format!("/packages/{id}/versions")).and_then(|v| {
        serde_json::from_value::<Vec<Release>>(v["items"].clone()).map_err(workspace::error)
    });
    let (versions, version_error) = match versions {
        Ok(v) => (v, None),
        Err(e) => (Vec::new(), Some(e)),
    };
    let dependencies = package
        .dependencies
        .iter()
        .take(16)
        .filter_map(|d| d.parse::<u64>().ok())
        .map(|id| {
            (
                id,
                self::package(id)
                    .map(|p| p.title().to_owned())
                    .unwrap_or_else(|_| format!("#{id}")),
            )
        })
        .collect();
    Ok(Detail {
        package,
        versions,
        version_error,
        dependencies,
    })
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub id: u64,
    pub updated_at: String,
    pub version: Option<u64>,
    pub current: bool,
    pub files: Vec<PathBuf>,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub version_note: Option<String>,
}
pub fn receipts(root: &Path) -> Result<Vec<Receipt>, String> {
    let bytes = match std::fs::read(root.join(".rockoon/online-resources.json")) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(workspace::error(e)),
    };
    let mut all: Vec<Receipt> = serde_json::from_slice(&bytes).map_err(workspace::error)?;
    all.retain(|r| {
        !r.files.is_empty()
            && r.files
                .iter()
                .all(|p| workspace::checked(root, &root.join(p)).is_ok_and(|p| p.is_file()))
    });
    Ok(all)
}
pub fn record(
    root: &Path,
    package: &Package,
    choice: &Choice,
    paths: Vec<PathBuf>,
) -> Result<(), String> {
    let mut all = receipts(root)?;
    all.retain(|r| r.id != package.id);
    let files = paths
        .into_iter()
        .map(|p| {
            p.strip_prefix(root)
                .map(Path::to_owned)
                .map_err(workspace::error)
        })
        .collect::<Result<_, _>>()?;
    all.push(Receipt {
        id: package.id,
        updated_at: package.updated_at.clone(),
        version: choice.version(),
        current: choice.current(),
        files,
        name: package.title().into(),
        category: package.category.clone(),
        version_note: match choice {
            Choice::Version(v) => v.note.clone(),
            Choice::Latest => None,
        },
    });
    workspace::replace(
        root,
        &[(
            root.join(".rockoon/online-resources.json"),
            serde_json::to_vec_pretty(&all).map_err(workspace::error)?,
        )],
    )
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImageKey {
    pub path: String,
    pub large: bool,
}
impl ImageKey {
    pub fn thumb(path: &str) -> Self {
        Self {
            path: path.into(),
            large: false,
        }
    }
    pub fn large(path: &str) -> Self {
        Self {
            path: path.into(),
            large: true,
        }
    }
}
fn image_path(input: &str) -> Result<String, String> {
    let url = url::Url::parse(input)
        .or_else(|_| url::Url::parse(&jobs::origin()).unwrap().join(input))
        .map_err(workspace::error)?;
    if url.scheme() != "https"
        || !jobs::ORIGINS
            .iter()
            .any(|o| url.origin().ascii_serialization() == *o)
        || !url.path().starts_with("/files/")
    {
        return Err("预览图地址不受支持".into());
    }
    Ok(url.path().to_owned())
}
static IMAGES: LazyLock<Arc<tokio::sync::Semaphore>> =
    LazyLock::new(|| Arc::new(tokio::sync::Semaphore::new(4)));
pub async fn fetch_image(key: ImageKey) -> Result<image::Handle, String> {
    let _permit = IMAGES.acquire().await.map_err(workspace::error)?;
    tokio::task::spawn_blocking(move || load_image(&key))
        .await
        .map_err(workspace::error)?
}
fn load_image(key: &ImageKey) -> Result<image::Handle, String> {
    let path = image_path(&key.path)?;
    let cache = preferences::directory()?.join("cache/resource-covers");
    let hash = format!("{:x}", Sha256::digest(path.as_bytes()));
    let file = cache.join(format!("{hash}.img"));
    let cached = std::fs::metadata(&file).ok().is_some_and(|m| {
        m.len() <= 8 * 1024 * 1024
            && m.modified()
                .ok()
                .and_then(|t| t.elapsed().ok())
                .is_some_and(|age| age < Duration::from_secs(7 * 86400))
    });
    let bytes = if cached {
        std::fs::read(&file).map_err(workspace::error)?
    } else {
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(5))
            .timeout_read(Duration::from_secs(10))
            .build();
        let active = jobs::origin();
        let mut last = String::new();
        let mut bytes = None;
        for origin in std::iter::once(active.as_str()).chain(jobs::ORIGINS) {
            let result = (|| {
                let response = agent
                    .get(&format!("{origin}{path}"))
                    .call()
                    .map_err(workspace::error)?;
                let mut data = Vec::new();
                response
                    .into_reader()
                    .take(8 * 1024 * 1024 + 1)
                    .read_to_end(&mut data)
                    .map_err(workspace::error)?;
                if data.len() > 8 * 1024 * 1024 {
                    return Err("预览图过大".into());
                }
                Ok(data)
            })();
            match result {
                Ok(data) => {
                    bytes = Some(data);
                    break;
                }
                Err(e) => last = e,
            }
        }
        let bytes = bytes.ok_or(last)?;
        // Cache is optional. A cache failure must not prevent displaying a decoded cover.
        if std::fs::create_dir_all(&cache).is_ok() {
            let temp = cache.join(format!("{hash}.{}.tmp", workspace::id()));
            if std::fs::write(&temp, &bytes).is_ok() && std::fs::rename(&temp, &file).is_err() {
                let _ = std::fs::remove_file(&temp);
            }
        }
        prune_cache(&cache);
        bytes
    };
    let decoded = (|| {
        let mut reader = bitmap::ImageReader::new(std::io::Cursor::new(&bytes))
            .with_guessed_format()
            .map_err(workspace::error)?;
        let mut limits = bitmap::Limits::default();
        limits.max_image_width = Some(4096);
        limits.max_image_height = Some(4096);
        limits.max_alloc = Some(64 * 1024 * 1024);
        reader.limits(limits);
        let pixels = reader.decode().map_err(workspace::error)?;
        let (w, h) = if key.large { (1200, 900) } else { (512, 288) };
        let pixels = pixels.thumbnail(w, h).into_rgba8();
        Ok(image::Handle::from_rgba(
            pixels.width(),
            pixels.height(),
            pixels.into_raw(),
        ))
    })();
    if decoded.is_err() {
        let _ = std::fs::remove_file(&file);
    }
    decoded
}

fn image_bytes(image: &image::Handle) -> usize {
    match image {
        image::Handle::Rgba { width, height, .. } => *width as usize * *height as usize * 4,
        _ => 0,
    }
}
fn prune_cache(cache: &Path) {
    let Ok(entries) = std::fs::read_dir(cache) else {
        return;
    };
    let mut files: Vec<_> = entries
        .filter_map(Result::ok)
        .filter(|e| {
            e.path().extension().is_some_and(|s| s == "img")
                && e.file_type().is_ok_and(|t| t.is_file())
        })
        .filter_map(|e| {
            e.metadata()
                .ok()
                .map(|m| (m.modified().ok(), m.len(), e.path()))
        })
        .collect();
    let mut size: u64 = files.iter().map(|(_, n, _)| *n).sum();
    if size <= 128 * 1024 * 1024 {
        return;
    }
    files.sort_by_key(|(modified, _, _)| *modified);
    for (_, bytes, path) in files {
        if size <= 96 * 1024 * 1024 {
            break;
        }
        if std::fs::remove_file(path).is_ok() {
            size = size.saturating_sub(bytes);
        }
    }
}

#[derive(Default)]
pub struct State {
    pub query: Query,
    pub generation: u64,
    pub total: usize,
    pub items: Vec<Package>,
    pub loading: bool,
    pub error: Option<String>,
    pub loaded: bool,
    pub selected: Option<Package>,
    pub detail: Option<Detail>,
    pub detail_generation: u64,
    pub detail_loading: bool,
    pub detail_error: Option<String>,
    pub markdown: Vec<markdown::Item>,
    pub choice: Choice,
    pub preview: usize,
    pub images: HashMap<ImageKey, image::Handle>,
    pub failed_images: HashSet<ImageKey>,
    pub pending_images: HashSet<ImageKey>,
    order: VecDeque<ImageKey>,
    pub receipts: Vec<Receipt>,
    pub receipt_root: Option<PathBuf>,
}
impl State {
    pub fn image(&self, key: &ImageKey) -> Option<&image::Handle> {
        self.images.get(key)
    }
    pub fn put_image(&mut self, key: ImageKey, result: Result<image::Handle, String>) {
        self.pending_images.remove(&key);
        match result {
            Ok(image) => {
                self.failed_images.remove(&key);
                self.order.retain(|k| *k != key);
                self.order.push_back(key.clone());
                self.images.insert(key, image);
                while self.order.len() > 64
                    || self.images.values().map(image_bytes).sum::<usize>() > 32 * 1024 * 1024
                {
                    if let Some(key) = self.order.pop_front() {
                        self.images.remove(&key);
                    }
                }
            }
            Err(_) => {
                self.failed_images.insert(key);
                if self.failed_images.len() > 128 {
                    self.failed_images.clear();
                }
            }
        }
    }
    pub fn installed(&self, p: &Package) -> bool {
        self.receipts
            .iter()
            .any(|r| r.id == p.id && r.current && r.updated_at == p.updated_at)
    }
    pub fn choices(&self) -> Vec<Choice> {
        std::iter::once(Choice::Latest)
            .chain(
                self.detail
                    .iter()
                    .flat_map(|d| d.versions.iter().cloned().map(Choice::Version)),
            )
            .collect()
    }
}
pub fn size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MiB", bytes as f64 / 1048576.0)
    } else {
        format!("{:.0} KiB", bytes as f64 / 1024.0)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_known_runtime_packages_use_the_game_editor() {
        for key in ["player-vc6", "player-msvc2022", "bml", "bmlplus"] {
            let package = Package {
                source_key: key.into(),
                category: "x-patch".into(),
                ..Default::default()
            };
            assert_eq!(package.managed_component(), Some(key));
        }
        for key in ["", "custom-patch"] {
            let package = Package {
                source_key: key.into(),
                category: "x-patch".into(),
                ..Default::default()
            };
            assert!(package.managed_component().is_none());
        }
    }
    #[test]
    fn extra_patch_installs_to_its_relative_paths_and_keeps_its_receipt() {
        use std::io::Write;
        let root = tempfile::tempdir().unwrap();
        std::fs::create_dir(root.path().join("Bin")).unwrap();
        std::fs::write(root.path().join("Bin/extra.dll"), b"before").unwrap();
        let archive = tempfile::NamedTempFile::new().unwrap();
        let mut zip = zip::ZipWriter::new(archive.reopen().unwrap());
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file("manifest.json", options).unwrap();
        zip.write_all(br#"{"manifest_version":1,"category":"x-patch","name":"Extra patch"}"#)
            .unwrap();
        zip.start_file("content/Bin/extra.dll", options).unwrap();
        zip.write_all(b"after").unwrap();
        zip.finish().unwrap();
        let (_, files) = workspace::brp_files(root.path(), archive.path(), "X").unwrap();
        assert_eq!(
            std::fs::read(root.path().join("Bin/extra.dll")).unwrap(),
            b"after"
        );
        let package = Package {
            id: 700,
            name: Some("Extra patch".into()),
            category: "x-patch".into(),
            ..Default::default()
        };
        record(
            root.path(),
            &package,
            &Choice::Version(Release {
                version: 3,
                note: Some("v1.2".into()),
                is_current: true,
            }),
            files,
        )
        .unwrap();
        let installed = receipts(root.path()).unwrap();
        assert_eq!(installed[0].name, "Extra patch");
        assert_eq!(installed[0].category, "x-patch");
        assert_eq!(installed[0].version_note.as_deref(), Some("v1.2"));
        assert!(root.path().join(".rockoon/backups").exists());
    }
    #[test]
    fn query_encodes_keywords_and_keeps_pagination() {
        let query = Query {
            text: "天空 & @author".into(),
            category: Category::Sky,
            sort: Sort::Name,
            page: 2,
        };
        let url = url::Url::parse(&format!("https://dl.bcrc.site{}", query.path())).unwrap();
        let pairs: HashMap<_, _> = url.query_pairs().into_owned().collect();
        assert_eq!(pairs["q"], query.text);
        assert_eq!(pairs["skip"], "24");
        assert_eq!(pairs["category"], "sky");
    }
    #[test]
    fn images_stay_on_the_public_resource_origins() {
        assert_eq!(image_path("/files/cover.png").unwrap(), "/files/cover.png");
        assert!(image_path("https://localhost/files/x").is_err());
        assert!(image_path("https://dl.bcrc.site.evil/files/x").is_err());
        assert!(image_path("file:///etc/passwd").is_err());
        assert!(image_path("/files/../../private").is_err());
    }
    #[test]
    fn receipts_follow_installed_files_and_version() {
        let root = tempfile::tempdir().unwrap();
        let file = root.path().join("ModLoader/Maps/test.nmo");
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(&file, b"fixture").unwrap();
        let p: Package = serde_json::from_value(
            serde_json::json!({"id":9,"category":"map","name":null,"updated_at":"2026-10-28"}),
        )
        .unwrap();
        record(root.path(), &p, &Choice::Latest, vec![file.clone()]).unwrap();
        let state = State {
            receipts: receipts(root.path()).unwrap(),
            ..State::default()
        };
        assert!(state.installed(&p));
        let mut newer = p.clone();
        newer.updated_at = "2026-10-29".into();
        assert!(!state.installed(&newer));
        std::fs::remove_file(file).unwrap();
        assert!(receipts(root.path()).unwrap().is_empty());
    }
}
