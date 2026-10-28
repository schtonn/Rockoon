use crate::workspace::{self, error};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{
        Arc, Condvar, LazyLock, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

static ACTIVE: LazyLock<Mutex<Option<String>>> = LazyLock::new(|| Mutex::new(None));
static INSTALL: Mutex<()> = Mutex::new(());
pub(crate) const ORIGINS: [&str; 2] = ["https://dl.ballance.top", "https://dl.bcrc.site"];
fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(5))
        .timeout_read(Duration::from_secs(15))
        .user_agent("Rockoon/2.7.1")
        .build()
}
pub fn json_url(url: &str) -> Result<Value, String> {
    let response = agent().get(url).call().map_err(error)?;
    let mut text = String::new();
    response
        .into_reader()
        .take(8 * 1024 * 1024)
        .read_to_string(&mut text)
        .map_err(error)?;
    serde_json::from_str(&text).map_err(error)
}
pub fn hub(path: &str) -> Result<Value, String> {
    let mut origins = Vec::new();
    if let Ok(active) = ACTIVE.lock()
        && let Some(active) = &*active
    {
        origins.push(active.clone());
    }
    for origin in ORIGINS {
        if !origins.iter().any(|o| o == origin) {
            origins.push(origin.into());
        }
    }
    let mut failure = "下载站不可达".to_owned();
    for origin in origins {
        match json_url(&format!("{origin}{path}")) {
            Ok(value) => {
                if let Ok(mut active) = ACTIVE.lock() {
                    *active = Some(origin);
                }
                return Ok(value);
            }
            Err(e) => failure = e,
        }
    }
    Err(failure)
}
pub fn origin() -> String {
    ACTIVE
        .lock()
        .ok()
        .and_then(|x| x.clone())
        .unwrap_or_else(|| ORIGINS[0].into())
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Patch {
    pub key: String,
    pub name: String,
    pub install_target: String,
    pub strip_top_level: bool,
    pub marker: String,
    pub package_id: Option<u64>,
    pub latest: String,
    #[serde(skip)]
    pub installed: Option<String>,
}
#[derive(Clone, Debug, Deserialize)]
pub struct Version {
    pub version: u64,
    pub note: Option<String>,
}
impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} · #{}",
            self.note.as_deref().unwrap_or("未标记"),
            self.version
        )
    }
}
impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version
    }
}
impl Eq for Version {}
pub fn patches() -> Result<Vec<Patch>, String> {
    serde_json::from_value(hub("/patches")?["items"].clone()).map_err(error)
}
pub fn versions(id: u64) -> Result<Vec<Version>, String> {
    serde_json::from_value(hub(&format!("/packages/{id}/versions"))?["items"].clone())
        .map_err(error)
}
#[derive(Clone, Debug)]
pub enum Request {
    Update {
        root: PathBuf,
        release: crate::updates::Release,
    },
    #[cfg(not(windows))]
    Wine { root: PathBuf },
    Game {
        root: PathBuf,
        player: bool,
        bmlplus: bool,
        token: String,
    },
    Patch {
        root: PathBuf,
        patch: Patch,
        version: Option<Version>,
    },
    Resource {
        root: PathBuf,
        package: crate::hub::Package,
        choice: crate::hub::Choice,
        sky: String,
    },
    Brp {
        root: PathBuf,
        url: String,
        sky: String,
    },
    SkyFiles {
        root: PathBuf,
        dir: PathBuf,
        files: Vec<PathBuf>,
    },
    Local {
        root: PathBuf,
        files: Vec<PathBuf>,
        sky: String,
    },
}
impl Request {
    pub fn root(&self) -> &Path {
        match self {
            Self::Update { root, .. }
            | Self::Game { root, .. }
            | Self::Patch { root, .. }
            | Self::Resource { root, .. }
            | Self::Brp { root, .. }
            | Self::Local { root, .. }
            | Self::SkyFiles { root, .. } => root,
            #[cfg(not(windows))]
            Self::Wine { root } => root,
        }
    }
    pub fn key(&self) -> String {
        match self {
            Self::Resource {
                root,
                package,
                choice,
                ..
            } => format!(
                "resource:{}:{}:{:?}",
                root.display(),
                package.id,
                choice.version()
            ),
            Self::Brp { root, url, .. } if package_id(url).is_some() => format!(
                "resource:{}:{}:{:?}",
                root.display(),
                package_id(url).unwrap(),
                selected_version(url)
            ),
            _ => format!("{self:?}"),
        }
    }
    pub fn title(&self) -> String {
        match self {
            Self::Update { .. } => "更新 Rockoon".into(),
            #[cfg(not(windows))]
            Self::Wine { .. } => "准备 Wine".into(),
            Self::Game { .. } => "安装 Ballance".into(),
            Self::Patch { patch, .. } => patch.name.clone(),
            Self::Resource { package, .. } => package.title().into(),
            Self::Brp { .. } => "导入资源包".into(),
            Self::Local { .. } => "导入本地资源包".into(),
            Self::SkyFiles { .. } => "导入天空盒".into(),
        }
    }
}
#[derive(Clone, Debug)]
pub struct Progress {
    pub phase: String,
    pub downloaded: u64,
    pub total: u64,
    pub speed: f64,
    pub done: bool,
    pub result: Option<Result<String, String>>,
    pub cancellable: bool,
    pub sky_prompt: Option<String>,
}
#[derive(Clone, Debug)]
pub struct Job {
    pub id: String,
    pub request: Request,
    pub state: Arc<Mutex<Progress>>,
    pub cancelled: Arc<AtomicBool>,
    sky_choice: Arc<(Mutex<Option<String>>, Condvar)>,
}
impl Job {
    pub fn new(request: Request) -> Self {
        Self {
            id: workspace::id(),
            request,
            state: Arc::new(Mutex::new(Progress {
                phase: "等待".into(),
                downloaded: 0,
                total: 0,
                speed: 0.0,
                done: false,
                result: None,
                cancellable: true,
                sky_prompt: None,
            })),
            cancelled: Arc::new(AtomicBool::new(false)),
            sky_choice: Arc::new((Mutex::new(None), Condvar::new())),
        }
    }
    pub fn progress(&self) -> Progress {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
        let _choice = self.sky_choice.0.lock().unwrap_or_else(|e| e.into_inner());
        self.sky_choice.1.notify_all();
    }
    pub fn choose_sky(&self, letter: String) {
        if !matches!(
            letter.as_str(),
            "X" | "A" | "B" | "C" | "D" | "E" | "F" | "G" | "H" | "I" | "J" | "K" | "L" | "M"
        ) {
            return;
        }
        let mut choice = self.sky_choice.0.lock().unwrap_or_else(|e| e.into_inner());
        *choice = Some(letter);
        self.sky_choice.1.notify_all();
    }
    fn choose_sky_target(&self, name: &str) -> Result<String, String> {
        self.check()?;
        let (choice, ready) = &*self.sky_choice;
        let mut choice = choice.lock().map_err(error)?;
        if choice.is_none() {
            let mut state = self.state.lock().map_err(error)?;
            state.phase = "等待选择关卡".into();
            state.cancellable = true;
            state.sky_prompt = Some(name.to_owned());
        }
        while choice.is_none() && !self.cancelled.load(Ordering::Relaxed) {
            choice = ready.wait(choice).map_err(error)?;
        }
        self.state.lock().map_err(error)?.sky_prompt = None;
        self.check()?;
        choice.clone().ok_or_else(|| "未选择天空盒关卡".into())
    }
    fn sky_for_archive(&self, root: &Path, file: &Path, fallback: &str) -> Result<String, String> {
        let info = rockoon_core::brp::validate_brp(file, root).map_err(error)?;
        if info.manifest.category == "sky" {
            self.choose_sky_target(info.manifest.name.as_deref().unwrap_or("天空盒"))
        } else {
            Ok(fallback.into())
        }
    }
    fn check(&self) -> Result<(), String> {
        if self.cancelled.load(Ordering::Relaxed) {
            Err("已取消".into())
        } else {
            Ok(())
        }
    }
    fn phase(&self, phase: &str) {
        if let Ok(mut s) = self.state.lock() {
            s.phase = phase.into();
        }
    }
    pub fn run(&self) -> Result<String, String> {
        let result = self.execute();
        if let Ok(mut s) = self.state.lock() {
            s.done = true;
            s.sky_prompt = None;
            s.cancellable = false;
            s.phase = match &result {
                Ok(_) => "完成",
                Err(e) if e == "已取消" => "已取消",
                Err(_) => "失败",
            }
            .into();
            s.result = Some(result.clone());
        }
        result
    }
    fn download(&self, url: &str) -> Result<tempfile::NamedTempFile, String> {
        self.check()?;
        self.phase("连接");
        if let Ok(mut state) = self.state.lock() {
            state.cancellable = true;
            state.downloaded = 0;
            state.total = 0;
            state.speed = 0.0;
        }
        let parsed = url::Url::parse(url).map_err(error)?;
        if parsed.scheme() != "https" {
            return Err("下载链接需要 HTTPS".into());
        }
        let mut urls = vec![url.to_owned()];
        if ORIGINS.iter().any(|o| url.starts_with(o)) {
            for origin in ORIGINS {
                let mut fallback = parsed.clone();
                let host = url::Url::parse(origin).map_err(error)?;
                fallback.set_host(host.host_str()).map_err(error)?;
                if !urls.contains(&fallback.to_string()) {
                    urls.push(fallback.to_string());
                }
            }
        }
        let mut response = None;
        let mut failure = String::new();
        for url in urls {
            self.check()?;
            match agent().get(&url).call() {
                Ok(r) => {
                    response = Some(r);
                    break;
                }
                Err(e) => failure = error(e),
            }
        }
        let response = response.ok_or(failure)?;
        let total = response
            .header("Content-Length")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        if total > 512 * 1024 * 1024 {
            return Err("下载文件超过 512 MiB".into());
        }
        self.phase("下载");
        let mut temp = tempfile::NamedTempFile::new().map_err(error)?;
        let mut reader = response.into_reader();
        let mut buf = [0u8; 65536];
        let started = Instant::now();
        let mut downloaded = 0;
        loop {
            self.check()?;
            let count = reader.read(&mut buf).map_err(error)?;
            if count == 0 {
                break;
            }
            downloaded += count as u64;
            if downloaded > 512 * 1024 * 1024 {
                return Err("下载文件过大".into());
            }
            temp.write_all(&buf[..count]).map_err(error)?;
            if let Ok(mut s) = self.state.lock() {
                s.downloaded = downloaded;
                s.total = total;
                s.speed = downloaded as f64 / started.elapsed().as_secs_f64().max(0.001);
            }
        }
        if total > 0 && total != downloaded {
            return Err("下载不完整，请重试".into());
        }
        temp.flush().map_err(error)?;
        self.check()?;
        Ok(temp)
    }
    fn patch(&self, root: &Path, patch: &Patch, version: Option<&Version>) -> Result<(), String> {
        let id = patch.package_id.ok_or("该补丁没有可下载版本")?;
        let path = if let Some(version) = version {
            format!("/packages/{id}/versions/{}/download", version.version)
        } else {
            format!("/packages/{id}/download")
        };
        let file = self.download(&format!("{}{path}", origin()))?;
        self.check()?;
        self.phase("安装");
        workspace::idle(root)?;
        if let Ok(mut s) = self.state.lock() {
            s.cancellable = false;
        }
        let target = if patch.install_target == "bin" {
            root.join("Bin")
        } else {
            root.to_path_buf()
        };
        // Validate marker before touching the game, and disable the conflicting loader in the same batch.
        let entries = workspace::archive_entries(file.path(), patch.strip_top_level)?;
        let mut changes = entries
            .into_iter()
            .map(|(p, b)| workspace::checked(root, &target.join(p)).map(|p| (p, b)))
            .collect::<Result<Vec<_>, _>>()?;
        let marker = workspace::checked(root, &target.join(&patch.marker))?;
        if !changes.iter().any(|(p, _)| p == &marker) {
            return Err("补丁包不包含安装清单标记文件".into());
        }
        let conflict = match patch.key.as_str() {
            "bml" => "BMLPlus.dll",
            "bmlplus" => "BML.dll",
            _ => "",
        };
        let conflict_file = root.join("BuildingBlocks").join(conflict);
        if !conflict.is_empty() && conflict_file.is_file() {
            let disabled = conflict_file.with_file_name(format!("{conflict}.disable"));
            changes.push((disabled, fs::read(&conflict_file).map_err(error)?));
        }
        let record_path = root.join("ModLoader/.rockoon-patch.json");
        let mut record: Value = fs::read(&record_path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_else(|| json!({}));
        if patch.key.starts_with("player-") {
            record
                .as_object_mut()
                .ok_or("补丁记录损坏")?
                .retain(|k, _| !k.starts_with("player-"));
        }
        let tag = version
            .and_then(|v| v.note.clone())
            .unwrap_or_else(|| patch.latest.clone());
        record[&patch.key] = json!({"tag":tag,"at":workspace::id()});
        changes.push((
            record_path,
            serde_json::to_vec_pretty(&record).map_err(error)?,
        ));
        workspace::replace(root, &changes)?;
        if !conflict.is_empty() && conflict_file.is_file() {
            workspace::trash(root, &conflict_file)?;
        }
        if patch.key == "bmlplus" {
            ensure_io(root)?;
        }
        Ok(())
    }
    fn execute(&self) -> Result<String, String> {
        let _lock = INSTALL.lock().map_err(error)?;
        self.check()?;
        match &self.request {
            Request::Update { root, release } => {
                use sha2::{Digest, Sha256};
                let file = self.download(&release.url)?;
                let bytes = fs::read(file.path()).map_err(error)?;
                if format!("{:x}", Sha256::digest(&bytes)) != release.sha256 {
                    return Err("更新文件 SHA-256 不一致".into());
                }
                self.check()?;
                #[cfg(windows)]
                if !bytes.starts_with(b"MZ") {
                    return Err("更新文件不是 Windows 可执行程序".into());
                }
                workspace::atomic(&root.join("native-update.bin"), &bytes)?;
                Ok("更新已下载，可在设置中安装".into())
            }
            #[cfg(not(windows))]
            Request::Wine { .. } => {
                if !rockoon_platform::process::find_processes("Player.exe".into())
                    .map_err(error)?
                    .is_empty()
                {
                    return Err("请先退出使用共享 Wine 环境的游戏，再重建 Wine 环境".into());
                }
                if let Ok(mut state) = self.state.lock() {
                    state.cancellable = false;
                }
                let state = self.state.clone();
                let context = rockoon_platform::Context {
                    progress: Some(Arc::new(move |phase| {
                        if let Ok(mut state) = state.lock() {
                            state.phase = phase.into();
                        }
                    })),
                };
                rockoon_platform::wine::install_wine(context, None).map_err(error)?;
                Ok("Wine 环境已准备".into())
            }
            Request::Game {
                root,
                player,
                bmlplus,
                token,
            } => {
                workspace::idle(root)?;
                let marker = root.join(".rockoon/native-install.json");
                let previous: Option<Value> = fs::read(&marker)
                    .ok()
                    .and_then(|b| serde_json::from_slice(&b).ok());
                let owned = previous.as_ref().is_some_and(|v| v["token"] == *token);
                if root.exists() && fs::read_dir(root).map_err(error)?.next().is_some() && !owned {
                    return Err("请选择空文件夹安装游戏".into());
                }
                fs::create_dir_all(root).map_err(error)?;
                if !owned {
                    workspace::atomic(
                        &marker,
                        &serde_json::to_vec(&json!({"token":token,"installed":false}))
                            .map_err(error)?,
                    )?;
                }
                let _ = hub("/patches")?;
                if !(owned
                    && previous.as_ref().is_some_and(|v| v["installed"] == true)
                    && root.join("Bin/Player.exe").is_file())
                {
                    let file = self.download(&format!("{}/game/download", origin()))?;
                    self.check()?;
                    self.phase("安装");
                    let entries = workspace::archive_entries(file.path(), false)?;
                    if !entries
                        .iter()
                        .any(|(p, _)| p == Path::new("Bin/Player.exe"))
                    {
                        return Err("游戏包中缺少 Bin/Player.exe".into());
                    }
                    if let Ok(mut s) = self.state.lock() {
                        s.cancellable = false;
                    }
                    let mut changes = entries
                        .into_iter()
                        .map(|(p, b)| workspace::checked(root, &root.join(p)).map(|p| (p, b)))
                        .collect::<Result<Vec<_>, _>>()?;
                    changes.push((
                        marker.clone(),
                        serde_json::to_vec(&json!({"token":token,"installed":true}))
                            .map_err(error)?,
                    ));
                    workspace::replace(root, &changes)?;
                }
                if *player || *bmlplus {
                    let list = patches()?;
                    if let Some(patch) = list.iter().find(|p| p.key == "player-vc6") {
                        self.patch(root, patch, None)?;
                    } else {
                        return Err("原版游戏已安装，但补丁清单缺少 NewPlayer".into());
                    }
                    if *bmlplus {
                        let patch = list
                            .iter()
                            .find(|p| p.key == "bmlplus")
                            .ok_or("原版游戏已安装，但清单缺少 BML+")?;
                        self.patch(root, patch, None)?;
                    }
                }
                fs::remove_file(marker).map_err(error)?;
                Ok("游戏安装完成".into())
            }
            Request::Patch {
                root,
                patch,
                version,
            } => {
                workspace::idle(root)?;
                if patch.key == "bmlplus"
                    && fs::metadata(root.join("Bin/Player.exe"))
                        .map_err(error)?
                        .len()
                        == 155648
                {
                    let list = patches()?;
                    let player = list
                        .iter()
                        .find(|p| p.key == "player-vc6")
                        .ok_or("请先安装 NewPlayer")?;
                    self.patch(root, player, None)?;
                }
                self.patch(root, patch, version.as_ref())?;
                Ok("补丁安装完成".into())
            }
            Request::SkyFiles { root, dir, files } => {
                let sky = self.choose_sky_target("本地天空盒")?;
                self.check()?;
                self.phase("导入");
                self.state.lock().map_err(error)?.cancellable = false;
                workspace::import_sky_files(root, dir, files, &sky)?;
                Ok("已导入天空盒".into())
            }
            Request::Local { root, files, sky } => {
                let mut reports = Vec::new();
                for file in files {
                    self.check()?;
                    let sky = self.sky_for_archive(root, file, sky)?;
                    self.phase("导入");
                    if let Ok(mut s) = self.state.lock() {
                        s.cancellable = false;
                    }
                    reports.push(workspace::brp(root, file, &sky)?);
                }
                Ok(reports.join("\n"))
            }
            Request::Resource {
                root,
                package,
                choice,
                sky,
            } => {
                let current = crate::hub::package(package.id)?;
                let suffix = choice
                    .version()
                    .map(|v| format!("versions/{v}/download"))
                    .unwrap_or_else(|| "download".into());
                let url = format!("{}/packages/{}/{suffix}", origin(), package.id);
                let queue = import_queue(&url)?;
                let mut reports = Vec::new();
                let mut paths = Vec::new();
                for url in queue {
                    let file = self.download(&url)?;
                    self.check()?;
                    let sky = self.sky_for_archive(root, file.path(), sky)?;
                    self.phase("安装");
                    if let Ok(mut state) = self.state.lock() {
                        state.cancellable = false;
                    }
                    let (message, installed) = workspace::brp_files(root, file.path(), &sky)?;
                    reports.push(message);
                    paths.extend(installed);
                }
                if let Err(e) = crate::hub::record(root, &current, choice, paths) {
                    reports.push(format!("安装已完成，记录未保存：{e}"));
                }
                Ok(reports.join("\n"))
            }
            Request::Brp { root, url, sky } => {
                let queue = import_queue(url)?;
                let mut reports = Vec::new();
                for url in queue {
                    let file = self.download(&url)?;
                    self.check()?;
                    let sky = self.sky_for_archive(root, file.path(), sky)?;
                    self.phase("导入");
                    if let Ok(mut s) = self.state.lock() {
                        s.cancellable = false;
                    }
                    reports.push(workspace::brp(root, file.path(), &sky)?);
                }
                Ok(reports.join("\n"))
            }
        }
    }
}
pub fn ensure_io(root: &Path) -> Result<(), String> {
    if !root.join("BuildingBlocks/BMLPlus.dll").is_file() {
        return Ok(());
    }
    let path = root.join("ModLoader/Mods/RockoonIO.bmodp");
    if !path.exists() && !path.with_file_name("RockoonIO.bmodp.disable").exists() {
        workspace::idle(root)?;
        workspace::replace(
            root,
            &[(
                path,
                include_bytes!("../resources/RockoonIO.bmodp").to_vec(),
            )],
        )?;
    }
    Ok(())
}
fn package_id(url: &str) -> Option<u64> {
    let url = url::Url::parse(url).ok()?;
    if !ORIGINS
        .iter()
        .any(|o| url.origin().ascii_serialization() == *o)
    {
        return None;
    }
    let mut parts = url.path_segments()?;
    while let Some(part) = parts.next() {
        if part == "packages" {
            return parts.next()?.parse().ok();
        }
    }
    None
}
fn selected_version(input: &str) -> Option<u64> {
    let parsed = url::Url::parse(input).ok()?;
    let parts: Vec<_> = parsed.path_segments()?.collect();
    if parts.len() == 5
        && parts[0] == "packages"
        && parts[2] == "versions"
        && parts[4] == "download"
    {
        parts[3].parse().ok()
    } else {
        None
    }
}
fn import_queue(url: &str) -> Result<Vec<String>, String> {
    plan_import(url, |id| hub(&format!("/packages/{id}")))
}
fn plan_import(
    url: &str,
    mut lookup: impl FnMut(u64) -> Result<Value, String>,
) -> Result<Vec<String>, String> {
    fn visit(
        id: u64,
        depth: usize,
        seen: &mut std::collections::HashSet<u64>,
        queue: &mut Vec<String>,
        lookup: &mut impl FnMut(u64) -> Result<Value, String>,
    ) -> Result<(), String> {
        if depth > 5 {
            return Err("资源依赖超过 5 层".into());
        }
        if !seen.insert(id) {
            return Ok(());
        }
        if seen.len() > 32 {
            return Err("单次安装依赖超过 32 个资源".into());
        }
        let info = lookup(id)?;
        if let Some(deps) = info["dependencies"].as_array() {
            for dep in deps {
                let id = dep
                    .as_u64()
                    .or_else(|| dep.as_str().and_then(|s| s.parse().ok()))
                    .ok_or("资源依赖编号无效")?;
                visit(id, depth + 1, seen, queue, lookup)?;
            }
        }
        queue.push(format!("{}/packages/{id}/download", origin()));
        Ok(())
    }
    if let Some(id) = package_id(url) {
        let mut queue = Vec::new();
        visit(
            id,
            0,
            &mut std::collections::HashSet::new(),
            &mut queue,
            &mut lookup,
        )?;
        if let Some(version) = selected_version(url)
            && let Some(target) = queue.last_mut()
        {
            *target = format!("{}/packages/{id}/versions/{version}/download", origin());
        }
        Ok(queue)
    } else {
        Ok(vec![url.into()])
    }
}
pub fn deep_link(value: &str) -> Result<String, String> {
    let url = url::Url::parse(value).map_err(error)?;
    if url.scheme() != "rockoon" || url.host_str() != Some("import") {
        return Err("不支持的 Rockoon 链接".into());
    }
    let target = url
        .query_pairs()
        .find(|(k, _)| k == "url")
        .map(|(_, v)| v.to_string())
        .ok_or("导入链接缺少 url")?;
    if url::Url::parse(&target).map_err(error)?.scheme() != "https" {
        return Err("导入地址需要 HTTPS".into());
    }
    Ok(target)
}
pub fn installed(root: &Path, patch: &Patch) -> Option<String> {
    let target = if patch.install_target == "bin" {
        root.join("Bin")
    } else {
        root.to_owned()
    };
    let marker = workspace::checked(root, &target.join(&patch.marker)).ok()?;
    if !marker.is_file() {
        return None;
    }
    if patch.key.starts_with("player-") && fs::metadata(&marker).ok()?.len() == 155648 {
        return None;
    }
    let record: Value = fs::read(root.join("ModLoader/.rockoon-patch.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_else(|| json!({}));
    Some(record[&patch.key]["tag"].as_str().unwrap_or("").to_string())
}
pub fn for_instance(root: Option<&Path>) -> Result<Vec<Patch>, String> {
    let mut list = patches()?;
    if let Some(root) = root {
        for patch in &mut list {
            patch.installed = installed(root, patch);
        }
    }
    Ok(list)
}
pub fn patch_updates(root: &Path) -> Result<Vec<Patch>, String> {
    Ok(for_instance(Some(root))?
        .into_iter()
        .filter(|p| {
            p.installed
                .as_ref()
                .is_some_and(|tag| !tag.is_empty() && tag != &p.latest)
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn wait_for_sky_prompt(job: &Job) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while job.progress().sky_prompt.is_none() {
            assert!(
                !job.progress().done,
                "job ended before asking for a sky target"
            );
            assert!(Instant::now() < deadline, "sky prompt was not reached");
            std::thread::sleep(Duration::from_millis(2));
        }
    }
    fn sky_archive(path: &Path) {
        let mut zip = zip::ZipWriter::new(fs::File::create(path).unwrap());
        let options = zip::write::SimpleFileOptions::default();
        zip.start_file("manifest.json", options).unwrap();
        zip.write_all(br#"{"manifest_version":1,"category":"sky","name":"Sky test"}"#)
            .unwrap();
        for face in ["Back", "Down", "Front", "Left", "Right"] {
            zip.start_file(format!("content/Sky_X_{face}.bmp"), options)
                .unwrap();
            zip.write_all(b"replacement face").unwrap();
        }
        zip.finish().unwrap();
    }
    #[test]
    fn sky_archive_waits_for_target_and_cancelling_preserves_files() {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir_all(root.path().join("Textures/Sky")).unwrap();
        let target = root.path().join("Textures/Sky/Sky_L_Front.bmp");
        fs::write(&target, b"original").unwrap();
        let archive = root.path().join("sky.brp");
        sky_archive(&archive);
        let job = Job::new(Request::Local {
            root: root.path().to_owned(),
            files: vec![archive.clone()],
            sky: "X".into(),
        });
        let worker = job.clone();
        let running = std::thread::spawn(move || worker.run());
        wait_for_sky_prompt(&job);
        assert_eq!(fs::read(&target).unwrap(), b"original");
        assert!(!root.path().join(".rockoon/backups").exists());
        job.cancel();
        assert_eq!(running.join().unwrap().unwrap_err(), "已取消");
        assert_eq!(fs::read(&target).unwrap(), b"original");
        assert!(!root.path().join("Textures/Sky/Sky_X_Front.bmp").exists());

        let job = Job::new(Request::Local {
            root: root.path().to_owned(),
            files: vec![archive],
            sky: "X".into(),
        });
        let worker = job.clone();
        let running = std::thread::spawn(move || worker.run());
        wait_for_sky_prompt(&job);
        job.choose_sky("L".into());
        running.join().unwrap().unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"replacement face");
        assert!(!root.path().join("Textures/Sky/Sky_X_Front.bmp").exists());
        assert!(root.path().join(".rockoon/backups").is_dir());
    }
    #[test]
    fn raw_sky_import_replaces_only_selected_target() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("Textures/Sky");
        fs::create_dir_all(&dir).unwrap();
        let source = root.path().join("Sky_X_Front.bmp");
        fs::write(&source, b"new").unwrap();
        fs::write(dir.join("Sky_A_Front.bmp"), b"old").unwrap();
        fs::write(dir.join("Sky_L_Front.bmp"), b"untouched").unwrap();
        let job = Job::new(Request::SkyFiles {
            root: root.path().to_owned(),
            dir: dir.clone(),
            files: vec![source],
        });
        let worker = job.clone();
        let running = std::thread::spawn(move || worker.run());
        wait_for_sky_prompt(&job);
        job.choose_sky("A".into());
        running.join().unwrap().unwrap();
        assert_eq!(fs::read(dir.join("Sky_A_Front.bmp")).unwrap(), b"new");
        assert_eq!(fs::read(dir.join("Sky_L_Front.bmp")).unwrap(), b"untouched");
    }
    #[test]
    fn resource_plan_keeps_selected_version_and_deduplicates_shared_dependencies() {
        let queue = plan_import(
            "https://dl.bcrc.site/packages/7/versions/3/download",
            |id| Ok(json!({"dependencies":match id{7=>vec![8,9],8|9=>vec![10],_=>vec![]}})),
        )
        .unwrap();
        let paths: Vec<_> = queue
            .iter()
            .map(|u| url::Url::parse(u).unwrap().path().to_owned())
            .collect();
        assert_eq!(
            paths,
            [
                "/packages/10/download",
                "/packages/8/download",
                "/packages/9/download",
                "/packages/7/versions/3/download"
            ]
        );
        assert!(package_id("https://example.org/packages/7/download").is_none());
        assert!(
            plan_import("https://dl.bcrc.site/packages/1/download", |id| Ok(
                json!({"dependencies":[id+1]})
            ))
            .is_err()
        );
    }
    #[test]
    fn optional_native_catalog_install() {
        if std::env::var("ROCKOON_HUB_NETWORK_TEST").as_deref() != Ok("1") {
            return;
        }
        let root = tempfile::tempdir().unwrap();
        let package = crate::hub::package(445).unwrap();
        let name = package.title().to_owned();
        let job = Job::new(Request::Resource {
            root: root.path().to_owned(),
            package,
            choice: crate::hub::Choice::Latest,
            sky: "X".into(),
        });
        let report = job.run().unwrap();
        let receipts = crate::hub::receipts(root.path()).unwrap();
        assert!(report.contains(&name));
        assert_eq!(receipts.len(), 1);
        assert_eq!(receipts[0].id, 445);
        assert!(!receipts[0].files.is_empty());
        assert!(
            receipts[0]
                .files
                .iter()
                .all(|p| root.path().join(p).is_file())
        );
        eprintln!("Native catalogue installed package 445 into isolated fixture: {report}");
    }

    #[test]
    #[ignore = "downloads the official game archive; run explicitly"]
    fn official_game_download_and_archive_validation() {
        let dir = tempfile::tempdir().unwrap();
        let job = Job::new(Request::Brp {
            root: dir.path().to_owned(),
            url: String::new(),
            sky: "X".into(),
        });
        let _ = hub("/patches").unwrap();
        let archive = job
            .download(&format!("{}/game/download", origin()))
            .unwrap();
        let files = workspace::archive_entries(archive.path(), false).unwrap();
        assert!(files.iter().any(|(p, _)| p == Path::new("Bin/Player.exe")));
        assert!(files.iter().any(|(p, _)| p == Path::new("Database.tdb")));
        assert_eq!(
            files
                .iter()
                .filter(|(p, _)| p.to_string_lossy().starts_with("3D Entities")
                    && p.extension().is_some_and(|e| e.eq_ignore_ascii_case("nmo")))
                .count(),
            51
        );
    }

    #[test]
    fn deep_link_decodes_once_and_rejects_unexpected_schemes() {
        assert_eq!(
            deep_link(
                "rockoon://import?url=https%3A%2F%2Fdl.ballance.top%2Fpackages%2F12%2Fdownload"
            )
            .unwrap(),
            "https://dl.ballance.top/packages/12/download"
        );
        assert!(deep_link("rockoon://import?url=file%3A%2F%2FC%3A%2Fsecret").is_err());
    }
}
