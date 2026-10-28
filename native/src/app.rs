use crate::{
    config::{self, Document, Source},
    game::{self, Instance, RunningGame},
    jobs::{self, Job, Patch, Request, Version},
    preferences::{self, Preferences},
    profiles, style, tas, vanilla,
    workspace::{self, Entry, Kind},
};
use iced::{Element, Subscription, Task};
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
mod catalog;
mod views;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Home,
    Hub,
    Local,
    Maps,
    Mods,
    Assets,
    Config,
    Data,
    Profiles,
    Install,
    Tasks,
    Tas,
    Settings,
}
impl Page {
    fn section(self) -> Self {
        match self {
            Self::Hub | Self::Local | Self::Maps | Self::Mods | Self::Assets => Self::Maps,
            Self::Config | Self::Data | Self::Profiles => Self::Config,
            Self::Install | Self::Tasks => Self::Tasks,
            other => other,
        }
    }
    fn section_label(self) -> &'static str {
        match self.section() {
            Self::Maps => "资源",
            Self::Tasks => "下载",
            other => other.label(),
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Home => "Ballance",
            Self::Hub => "在线",
            Self::Local => "本地",
            Self::Maps => "地图",
            Self::Mods => "模组",
            Self::Assets => "素材",
            Self::Config => "配置",
            Self::Data => "存档",
            Self::Profiles => "配置预设",
            Self::Install => "游戏与组件",
            Self::Tasks => "任务",
            Self::Tas => "TAS",
            Self::Settings => "设置",
        }
    }
}
#[derive(Clone, Debug)]
pub enum PromptKind {
    NewFolder,
    Delete(PathBuf),
    Rename(PathBuf),
    CreateProfile,
    RenameProfile(String),
    DeleteProfile(String),
    Discard(Box<Message>),
    Restore,
    ResetData,
}
#[derive(Clone, Debug)]
struct Prompt {
    title: String,
    input: String,
    kind: PromptKind,
}
#[derive(Clone, Debug)]
pub enum TasAction {
    New,
    Open,
    Save,
    SaveAs,
    Undo,
    Redo,
    Copy,
    Cut,
    Paste,
    Insert,
    Toggle(usize, usize),
    Range(usize, bool),
}
#[derive(Clone, Debug)]
pub struct Startup {
    instance: Option<Instance>,
    preferences: Preferences,
    problem: Option<String>,
    protocol_ready: bool,
}
#[derive(Clone, Debug)]
pub enum Message {
    Hub(catalog::Action),
    Startup(Result<Startup, String>),
    PickFolder,
    FolderPicked(Option<PathBuf>),
    Loaded(Result<Instance, String>),
    Rescanned(PathBuf, Result<Instance, String>),
    Page(Page),
    FolderMenu,
    CloseMenus,
    NoticeTick,
    LatestVersion,
    Search(String),
    Refresh,
    Open(PathBuf),
    Opened(Result<(), String>),
    EntryMenu(PathBuf),
    Browse(PathBuf),
    Listed(u64, Result<Vec<Entry>, String>),
    ResourceCategory(crate::hub::Category),
    ResourceSource(bool),
    Import,
    FilesPicked(Vec<PathBuf>),
    Classified(
        PathBuf,
        PathBuf,
        Option<Kind>,
        String,
        Result<(Vec<PathBuf>, Vec<PathBuf>), String>,
    ),
    ImportUrl(String),
    ImportRemote,
    Move(PathBuf),
    MovePicked(PathBuf, Option<PathBuf>),
    Prompt(PromptKind),
    PromptText(String),
    Confirm,
    CancelPrompt,
    Discard(Box<Message>),
    Changed(Result<String, String>),
    UndoDelete,
    Toggle(Entry),
    Preview(PathBuf),
    Previewed(u64, Result<iced::widget::image::Handle, String>),
    SkyLoaded(u64, Result<std::sync::Arc<crate::sky::Skybox>, String>),
    SkyAngle(bool, f32),
    SkyRender,
    ClosePreview,
    Sound(PathBuf),
    StopSound,
    FindConfig(PathBuf),
    FoundConfig(Result<PathBuf, String>),
    UnlockAll,
    ResetLevel(usize),
    SelectConfig(String),
    DocumentLoaded(u64, Result<(Document, Vec<PathBuf>), String>),
    ConfigSection(String, bool),
    Field(usize, String),
    SaveDocument,
    ProfilesLoaded(u64, Result<Value, String>),
    Profile(profiles::Action),
    Patches(u64, Result<Vec<Patch>, String>),
    Versions(usize),
    VersionsLoaded(u64, Result<Vec<Version>, String>),
    Version(Version),
    InstallPatch(usize),
    InstallGame,
    InstallPicked(Option<PathBuf>),
    InstallChoice(bool, bool),
    Sky(String),
    JobFinished(String, Result<String, String>),
    CancelJob(String),
    RetryJob(String),
    ClearJobs,
    Launch(Option<PathBuf>),
    LaunchChecked(Option<PathBuf>, Result<Vec<String>, String>),
    LaunchAnyway(Option<PathBuf>),
    Started(Result<RunningGame, String>),
    Stop,
    Stopped(Result<(), String>),
    Tick,
    Incoming,
    Hide(Option<iced::window::Id>),
    Show(Option<iced::window::Id>),
    PrefBool(String, bool),
    ColorMode(preferences::ColorMode),
    PrefText(String, String),
    Saved(Result<(), String>),
    Tas(TasAction),
    TasOpened(Result<tas::Project, String>),
    TasPath(Option<PathBuf>),
    TasSaved(PathBuf, Result<(), String>),
    TasStart(String),
    TasEnd(String),
    TasCount(String),
    TasOffset(usize),
    Event(iced::Event),
    RegisterProtocol,
    ProtocolRegistered(Result<(), String>),
    OpenHub,
    HubOpened(Result<(), String>),
    AutoUpdate(Result<Option<crate::updates::Release>, String>),
    AutoPatches(Result<Vec<Patch>, String>),
    CheckUpdate,
    UpdateChecked(Result<Option<crate::updates::Release>, String>),
    DownloadUpdate,
    ApplyUpdate,
    #[cfg(not(windows))]
    WineSetup,
    #[cfg(not(windows))]
    WineChoose,
    #[cfg(not(windows))]
    WineChosen(Option<PathBuf>),
    #[cfg(not(windows))]
    WineConfig,
    #[cfg(target_os = "macos")]
    Rosetta,
    Quit,
    Exit,
    OpenTasProject(PathBuf),
    OpenTas,
    TasLevel(u8),
    TasFps(String),
    TasPlay(bool),
    TasPrepared(Result<RunningGame, String>),
    TasHost(Option<iced::window::Id>),
    TasRaw(u64),
    TasEmbedded(Result<i64, String>),
    TasDetach,
    TasInteractive(bool),
    TasFrameLimit(Result<(), String>),
    TasDisabled(Option<PathBuf>, Result<(), String>),
    DisableAutoplay(Option<PathBuf>),
    TasScale(u64, f32),
    TasBounds(u64, f32, Option<iced::Rectangle>),
    Stats(Result<preferences::Stats, String>),
    Dismiss,
}
struct Notice {
    text: String,
    since: Instant,
    duration: Duration,
}
impl Notice {
    fn new(text: impl Into<String>, duration: Duration) -> Self {
        Self {
            text: text.into(),
            since: Instant::now(),
            duration,
        }
    }
    fn opacity(&self) -> f32 {
        (self.since.elapsed().as_secs_f32() / 0.16).min(1.0)
            * (self
                .duration
                .saturating_sub(self.since.elapsed())
                .as_secs_f32()
                / 0.3)
                .min(1.0)
    }
}
pub struct App {
    hub: crate::hub::State,
    home_active: bool,
    detached_tas: bool,
    tas_level: u8,
    tas_fps: String,
    tas_playing: bool,
    tas_embedded: bool,
    tas_interactive: bool,
    tas_host: Option<(u64, f32)>,
    update_release: Option<crate::updates::Release>,
    update_path: Option<PathBuf>,
    page: Page,
    folder_menu: bool,
    setup_problem: Option<String>,
    protocol_ready: bool,
    opening_hub: bool,
    config_sections: std::collections::HashMap<String, bool>,
    instance: Option<Instance>,
    preferences: Preferences,
    busy: bool,
    mutating: bool,
    revision: u64,
    preview_revision: u64,
    quitting: bool,
    picking_folder: bool,
    launching: bool,
    running: Option<RunningGame>,
    session: Option<Instant>,
    search: String,
    notice: Option<Notice>,
    version_menu: Option<usize>,
    entries: Vec<Entry>,
    directory: PathBuf,
    asset_kind: Kind,
    local_category: crate::hub::Category,
    document: Option<Document>,
    config_files: Vec<PathBuf>,
    config_name: String,
    group: String,
    profile_index: Value,
    patches: Vec<Patch>,
    versions: Vec<Version>,
    patch_index: Option<usize>,
    version: Option<Version>,
    jobs: Vec<Job>,
    finished_jobs: std::collections::HashSet<String>,
    import_url: String,
    sky: String,
    install_player: bool,
    install_bmlplus: bool,
    preview: Option<iced::widget::image::Handle>,
    skybox: Option<std::sync::Arc<crate::sky::Skybox>>,
    sky_yaw: f32,
    sky_pitch: f32,
    preview_path: Option<PathBuf>,
    sound: Option<PathBuf>,
    entry_menu: Option<PathBuf>,
    prompt: Option<Prompt>,
    deleted: Option<(PathBuf, PathBuf)>,
    launch_warning: Option<(Option<PathBuf>, Vec<String>)>,
    tas: tas::Project,
    tas_busy: bool,
    tas_start: String,
    tas_end: String,
    tas_count: String,
    tas_offset: usize,
}
async fn background<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tokio::task::spawn_blocking(work)
        .await
        .map_err(workspace::error)?
}
impl App {
    pub fn new() -> (Self, Task<Message>) {
        let state = Self {
            hub: crate::hub::State::default(),
            home_active: true,
            detached_tas: std::env::args().any(|a| a == "--tas-editor"),
            tas_level: 1,
            tas_fps: "240".into(),
            tas_playing: false,
            tas_embedded: false,
            tas_interactive: false,
            tas_host: None,
            update_release: None,
            update_path: None,
            page: Page::Home,
            folder_menu: std::env::args().any(|a| a == "--folder-menu"),
            setup_problem: None,
            protocol_ready: false,
            opening_hub: false,
            config_sections: std::collections::HashMap::new(),
            instance: None,
            preferences: Preferences::default(),
            busy: true,
            mutating: false,
            revision: 0,
            preview_revision: 0,
            quitting: false,
            picking_folder: false,
            launching: false,
            running: None,
            session: None,
            search: String::new(),
            notice: None,
            version_menu: None,
            entries: vec![],
            directory: PathBuf::new(),
            asset_kind: Kind::Skies,
            local_category: crate::hub::Category::Map,
            document: None,
            config_files: vec![],
            config_name: "游戏设置".into(),
            group: String::new(),
            profile_index: json!({"profiles":[],"currentId":""}),
            patches: vec![],
            versions: vec![],
            patch_index: None,
            version: None,
            jobs: vec![],
            finished_jobs: std::collections::HashSet::new(),
            import_url: String::new(),
            sky: "自制地图（X）".into(),
            install_player: true,
            install_bmlplus: true,
            skybox: None,
            sky_yaw: 0.0,
            sky_pitch: 0.0,
            preview: None,
            preview_path: None,
            sound: None,
            entry_menu: None,
            prompt: None,
            deleted: None,
            launch_warning: None,
            tas: tas::Project::default(),
            tas_busy: false,
            tas_start: "0".into(),
            tas_end: "0".into(),
            tas_count: "240".into(),
            tas_offset: 0,
        };
        let requested = std::env::args_os()
            .skip(1)
            .map(PathBuf::from)
            .find(|p| p.is_dir());
        (
            state,
            Task::perform(
                background(move || {
                    let mut prefs = preferences::load()?;
                    if let Ok(stats) = preferences::stats() {
                        prefs.playtime = stats.total;
                        prefs.last_played = stats.last;
                    }
                    let (instance, problem) = match requested
                        .or_else(|| prefs.instance_path.clone())
                        .map(game::discover)
                        .transpose()
                    {
                        Ok(instance) => (instance, None),
                        Err(error) => (None, Some(format!("之前的游戏目录无法使用：{error}"))),
                    };
                    Ok(Startup {
                        instance,
                        preferences: prefs,
                        problem,
                        protocol_ready: crate::system::protocol_registered(),
                    })
                }),
                Message::Startup,
            ),
        )
    }
    fn root(&self) -> Option<PathBuf> {
        self.instance.as_ref().map(|i| i.root.clone())
    }
    fn kind(&self) -> Option<Kind> {
        match self.page {
            Page::Maps => Some(Kind::Maps),
            Page::Mods => Some(Kind::Mods),
            Page::Assets => Some(self.asset_kind),
            _ => None,
        }
    }
    fn select_local_category(&mut self, category: crate::hub::Category) -> Task<Message> {
        self.local_category = category;
        self.directory = PathBuf::new();
        let page = match category.local_kind() {
            Some(Kind::Maps) => Page::Maps,
            Some(Kind::Mods) => Page::Mods,
            Some(kind) => {
                self.asset_kind = kind;
                Page::Assets
            }
            None => Page::Local,
        };
        if page == self.page {
            return self.reload();
        }
        self.update(Message::Page(page))
    }
    fn locked(&self) -> bool {
        self.busy
            || self.mutating
            || self.launching
            || self.running.is_some()
            || self.jobs.iter().any(|j| !j.progress().done)
    }
    fn lock_reason(&self) -> &'static str {
        if self.running.is_some() {
            "当前目录的游戏正在运行，请先退出该游戏"
        } else if self.launching {
            "当前目录的游戏正在启动，请稍候"
        } else {
            "请等待当前操作完成后重试"
        }
    }
    fn error(&mut self, message: impl Into<String>) {
        self.notice = Some(Notice::new(message, Duration::from_secs(8)));
    }
    fn notify(&mut self, message: impl Into<String>) {
        self.notice = Some(Notice::new(message, Duration::from_secs(5)));
    }
    fn close_menus(&mut self) {
        self.folder_menu = false;
        self.entry_menu = None;
        self.version_menu = None;
    }
    fn save_preferences(&self) -> Task<Message> {
        let prefs = self.preferences.clone();
        let sequence = preferences::next_save();
        Task::perform(
            background(move || preferences::save(&prefs, sequence)),
            Message::Saved,
        )
    }
    fn reload(&mut self) -> Task<Message> {
        self.revision += 1;
        let revision = self.revision;
        self.busy = false;
        self.preview_revision += 1;
        self.skybox = None;
        self.preview = None;
        self.entry_menu = None;
        if let Some(kind) = self.kind()
            && let Some(root) = self.root()
        {
            let base = root.join(kind.dir());
            if !self.directory.starts_with(&base) {
                self.directory = base;
            }
            let directory = self.directory.clone();
            self.busy = true;
            return Task::perform(
                background(move || workspace::list(&root, kind, &directory)),
                move |r| Message::Listed(revision, r),
            );
        }
        match self.page {
            Page::Local => {
                if let Some(root) = self.root() {
                    if self.local_category == crate::hub::Category::Patch {
                        return Task::none();
                    }
                    self.busy = true;
                    return Task::perform(
                        background(move || workspace::collection(&root)),
                        move |r| Message::Listed(revision, r),
                    );
                }
            }
            Page::Hub => return self.ensure_catalog(),
            Page::Config | Page::Data => {
                if let Some(root) = self.root() {
                    self.busy = true;
                    self.document = None;
                    let source = if self.page == Page::Data {
                        Source::Data
                    } else if self.config_name == "游戏设置" {
                        Source::Game
                    } else {
                        Source::File(
                            self.config_files
                                .iter()
                                .find(|p| {
                                    p.file_name()
                                        .is_some_and(|p| p.to_string_lossy() == self.config_name)
                                })
                                .cloned()
                                .unwrap_or_else(|| root.join("Bin/Player.ini")),
                        )
                    };
                    return Task::perform(
                        background(move || {
                            Ok((config::load(&root, source)?, config::files(&root)?))
                        }),
                        move |r| Message::DocumentLoaded(revision, r),
                    );
                }
            }
            Page::Profiles => {
                if let Some(root) = self.root() {
                    self.busy = true;
                    return Task::perform(background(move || profiles::load(&root)), move |r| {
                        Message::ProfilesLoaded(revision, r)
                    });
                }
            }
            Page::Settings => return Task::perform(background(preferences::stats), Message::Stats),
            Page::Install => {
                let root = self.root();
                if root.is_none() {
                    self.busy = false;
                    self.patches.clear();
                    return Task::none();
                }
                self.busy = true;
                return Task::perform(
                    background(move || jobs::for_instance(root.as_deref())),
                    move |r| Message::Patches(revision, r),
                );
            }
            _ => {}
        }
        Task::none()
    }
    fn change(
        &mut self,
        work: impl FnOnce() -> Result<String, String> + Send + 'static,
    ) -> Task<Message> {
        if self.locked() {
            self.error(self.lock_reason());
            return Task::none();
        }
        self.mutating = true;
        Task::perform(background(work), Message::Changed)
    }
    fn start_job(&mut self, request: Request) -> Task<Message> {
        if self
            .jobs
            .iter()
            .any(|job| job.request.key() == request.key() && !job.progress().done)
        {
            self.page = Page::Tasks;
            return Task::none();
        }
        if self.locked() {
            self.error(self.lock_reason());
            return Task::none();
        }
        let job = Job::new(request);
        let id = job.id.clone();
        self.jobs.push(job.clone());
        self.page = Page::Tasks;
        Task::perform(background(move || job.run()), move |result| {
            Message::JobFinished(id.clone(), result)
        })
    }
    fn show_window() -> Task<Message> {
        iced::window::latest().map(Message::Show)
    }
    fn launch(&mut self, map: Option<PathBuf>) -> Task<Message> {
        if self.locked() {
            return Task::none();
        }
        let Some(instance) = self.instance.clone() else {
            return Task::none();
        };
        let prefs = self.preferences.clone();
        self.launching = true;
        self.launch_warning = None;
        Task::perform(
            background(move || game::launch(&instance, map.as_deref(), &prefs)),
            Message::Started,
        )
    }
    pub fn update(&mut self, message: Message) -> Task<Message> {
        if let Message::Hub(action) = message {
            return self.update_catalog(action);
        }
        if matches!(
            &message,
            Message::Open(_)
                | Message::PickFolder
                | Message::Prompt(_)
                | Message::Move(_)
                | Message::FindConfig(_)
                | Message::Version(_)
                | Message::LatestVersion
                | Message::InstallPatch(_)
                | Message::Browse(_)
                | Message::Page(_)
        ) {
            self.close_menus();
        }
        match message {
            Message::Hub(_) => unreachable!(),
            Message::CloseMenus => self.close_menus(),
            Message::NoticeTick => {
                if self
                    .notice
                    .as_ref()
                    .is_some_and(|notice| notice.since.elapsed() >= notice.duration)
                {
                    self.notice = None;
                }
            }
            Message::Startup(result) => {
                self.busy = false;
                match result {
                    Ok(startup) => {
                        self.instance = startup.instance;
                        self.setup_problem = startup.problem;
                        self.protocol_ready = startup.protocol_ready;
                        crate::l10n::set(&startup.preferences.language);
                        self.preferences = startup.preferences;
                        let args: Vec<_> = std::env::args().skip(1).collect();
                        if self.detached_tas {
                            self.page = Page::Tas;
                            if let Some(path) = args.iter().map(PathBuf::from).find(|p| {
                                p.extension().is_some_and(|e| e.eq_ignore_ascii_case("tas"))
                                    && p.is_file()
                            }) {
                                return Task::perform(
                                    background(move || tas::Project::read(&path)),
                                    Message::TasOpened,
                                );
                            }
                            return self.reload();
                        }

                        if let Some(page) = args.iter().find_map(|s| s.strip_prefix("--page=")) {
                            let page = match page {
                                "hub" => Page::Hub,
                                "local" => Page::Local,
                                "maps" => Page::Maps,
                                "mods" => Page::Mods,
                                "assets" => Page::Assets,
                                "config" => Page::Config,
                                "data" => Page::Data,
                                "profiles" => Page::Profiles,
                                "install" => Page::Install,
                                "tasks" => Page::Tasks,
                                "tas" => Page::Tas,
                                "settings" => Page::Settings,
                                _ => Page::Home,
                            };
                            self.page = page;
                            if page == Page::Local {
                                self.local_category = crate::hub::Category::All;
                            } else if let Some(kind) = self.kind() {
                                self.local_category = crate::hub::Category::from_kind(kind);
                            }
                            return self.reload();
                        }
                        if let Some(url) = args.iter().find(|x| x.starts_with("rockoon://")) {
                            match jobs::deep_link(url) {
                                Ok(url) => {
                                    self.import_url = url;
                                    self.page = Page::Tasks;
                                }
                                Err(e) => self.setup_problem = Some(e),
                            }
                        } else if let Some(path) =
                            args.iter().map(PathBuf::from).find(|p| p.is_file())
                        {
                            return self.update(Message::FilesPicked(vec![path]));
                        }
                        let mut checks = vec![Task::perform(
                            background(crate::updates::check),
                            Message::AutoUpdate,
                        )];
                        if let Some(root) = self.root() {
                            checks.push(Task::perform(
                                background(move || jobs::patch_updates(&root)),
                                Message::AutoPatches,
                            ));
                        }
                        return Task::batch(checks);
                    }
                    Err(e) => self.setup_problem = Some(e),
                }
            }
            Message::PickFolder if !self.locked() && !self.picking_folder => {
                if self.document.as_ref().is_some_and(|d| d.dirty) {
                    return self.update(Message::Prompt(PromptKind::Discard(Box::new(
                        Message::PickFolder,
                    ))));
                }
                self.folder_menu = false;
                self.picking_folder = true;
                let initial = self.root();
                return Task::perform(
                    async move {
                        let mut picker =
                            rfd::AsyncFileDialog::new().set_title("选择 Ballance 游戏根目录");
                        if let Some(root) = initial {
                            picker = picker.set_directory(root);
                        }
                        picker.pick_folder().await.map(|f| f.path().to_owned())
                    },
                    Message::FolderPicked,
                );
            }
            Message::FolderPicked(path) => {
                self.picking_folder = false;
                if let Some(path) = path {
                    self.busy = true;
                    return Task::perform(
                        background(move || game::discover(path)),
                        Message::Loaded,
                    );
                }
            }
            Message::Loaded(result) => {
                self.busy = false;
                match result {
                    Ok(instance) => {
                        self.preferences.instance_path = Some(instance.root.clone());
                        self.instance = Some(instance);
                        self.setup_problem = None;
                        self.document = None;
                        self.directory = PathBuf::new();
                        self.deleted = None;
                        self.config_name = "游戏设置".into();
                        return Task::batch([self.save_preferences(), self.reload()]);
                    }
                    Err(e) => {
                        if self.instance.is_none() {
                            self.setup_problem = Some(e.clone());
                        }
                        self.error(e);
                    }
                }
            }
            Message::Rescanned(root, result) => {
                if self.root().as_ref() == Some(&root) {
                    match result {
                        Ok(instance) => {
                            self.instance = Some(instance);
                            if self.kind().is_some() || self.page == Page::Local {
                                return self.reload();
                            }
                            if self.page == Page::Hub {
                                return self.ensure_catalog();
                            }
                        }
                        Err(e) => self.error(e),
                    }
                }
            }
            Message::FolderMenu => {
                let open = !self.folder_menu;
                self.close_menus();
                self.folder_menu = open;
            }
            Message::Page(page) => {
                if page == self.page {
                    return Task::none();
                }
                if page == Page::Tas && !self.detached_tas {
                    return self.update(Message::OpenTas);
                }

                if self.document.as_ref().is_some_and(|d| d.dirty) && page != self.page {
                    self.prompt = Some(Prompt {
                        title: "放弃尚未保存的配置？".into(),
                        input: String::new(),
                        kind: PromptKind::Discard(Box::new(Message::Page(page))),
                    });
                    return Task::none();
                }
                self.page = page;
                if let Some(kind) = self.kind() {
                    self.local_category = crate::hub::Category::from_kind(kind);
                }
                self.folder_menu = false;
                self.search.clear();
                self.directory = PathBuf::new();
                self.document = None;
                self.preview = None;
                return self.reload();
            }
            Message::Discard(next) => {
                if let Some(doc) = &mut self.document {
                    doc.dirty = false;
                }
                return self.update(*next);
            }
            Message::Search(s) => self.search = s,
            Message::Refresh => {
                if self.document.as_ref().is_some_and(|d| d.dirty) {
                    return self.update(Message::Prompt(PromptKind::Discard(Box::new(
                        Message::Refresh,
                    ))));
                }
                return self.reload();
            }
            Message::Open(path) => {
                self.folder_menu = false;
                return Task::perform(
                    background(move || game::open_folder(&path)),
                    Message::Opened,
                );
            }
            Message::Opened(Err(e)) => self.error(e),
            Message::EntryMenu(path) => {
                self.entry_menu = if self.entry_menu.as_ref() == Some(&path) {
                    None
                } else {
                    Some(path)
                }
            }
            Message::Browse(path) => {
                self.directory = path;
                self.search.clear();
                return self.reload();
            }
            Message::Listed(revision, result) => {
                if revision != self.revision {
                    return Task::none();
                }
                self.busy = false;
                match result {
                    Ok(entries) => self.entries = entries,
                    Err(e) => self.error(e),
                }
            }
            Message::ResourceSource(online) => {
                if online {
                    let category = self.local_category;
                    if self.page != Page::Hub && category != self.hub.query.category {
                        self.hub.selected = None;
                        self.hub.detail_generation += 1;
                        self.hub.detail_loading = false;
                        self.hub.query.category = category;
                        self.hub.query.page = 0;
                        self.hub.loaded = false;
                        self.hub.generation += 1;
                        self.hub.loading = false;
                    }
                    return self.update(Message::Page(Page::Hub));
                }
                return self.select_local_category(self.hub.query.category);
            }
            Message::ResourceCategory(category) => {
                if self.page == Page::Hub {
                    return self.update_catalog(catalog::Action::Category(category));
                }
                return self.select_local_category(category);
            }
            Message::Import if !self.locked() => {
                let initial = self.directory.clone();
                return Task::perform(
                    async move {
                        rfd::AsyncFileDialog::new()
                            .set_title("导入资源或 BRP/ZIP 资源包")
                            .set_directory(initial)
                            .pick_files()
                            .await
                            .unwrap_or_default()
                            .into_iter()
                            .map(|f| f.path().to_owned())
                            .collect()
                    },
                    Message::FilesPicked,
                );
            }
            Message::FilesPicked(files) => {
                if files.len() == 1
                    && files[0]
                        .extension()
                        .is_some_and(|e| e.eq_ignore_ascii_case("tas"))
                {
                    let path = files[0].clone();
                    if self.detached_tas {
                        if self.tas.dirty {
                            self.error("请先导出当前 TAS，再打开其他项目");
                            return Task::none();
                        }
                        return Task::perform(
                            background(move || tas::Project::read(&path)),
                            Message::TasOpened,
                        );
                    }
                    return self.update(Message::OpenTasProject(path));
                }

                let Some(root) = self.root() else {
                    self.error("先选择游戏目录，再导入资源");
                    return Task::none();
                };
                if files.is_empty() {
                    return Task::none();
                }
                let dir = self.directory.clone();
                let kind = self.kind();
                let sky = self.sky_letter();
                self.busy = true;
                return Task::perform(background(move || workspace::classify(files)), move |r| {
                    Message::Classified(root.clone(), dir.clone(), kind, sky.clone(), r)
                });
            }
            Message::Classified(root, dir, kind, sky, result) => {
                self.busy = false;
                if self.root().as_ref() != Some(&root) {
                    self.error("目录已更换，请重新导入");
                    return Task::none();
                }
                let (packages, raw) = match result {
                    Ok(value) => value,
                    Err(e) => {
                        self.error(e);
                        return Task::none();
                    }
                };
                if !packages.is_empty() && !raw.is_empty() {
                    self.error("请分别导入资源包和单个资源文件");
                    return Task::none();
                }
                if !packages.is_empty() {
                    return self.start_job(Request::Local {
                        root,
                        files: packages,
                        sky,
                    });
                }
                if !raw.is_empty() {
                    let Some(kind) = kind else {
                        self.error("请在对应资源页导入单个文件");
                        return Task::none();
                    };
                    return self.change(move || {
                        workspace::import_files(&root, &dir, kind, &raw)?;
                        Ok("已导入".into())
                    });
                }
            }
            Message::ImportUrl(url) => self.import_url = url,
            Message::ImportRemote => {
                let Some(root) = self.root() else {
                    self.error("先选择游戏目录");
                    return Task::none();
                };
                let url = if self.import_url.starts_with("rockoon:") {
                    match jobs::deep_link(&self.import_url) {
                        Ok(url) => url,
                        Err(e) => {
                            self.error(e);
                            return Task::none();
                        }
                    }
                } else {
                    self.import_url.clone()
                };
                return self.start_job(Request::Brp {
                    root,
                    url,
                    sky: self.sky_letter(),
                });
            }
            Message::Move(path) if !self.locked() => {
                let directory = self.directory.clone();
                return Task::perform(
                    async move {
                        (
                            path,
                            rfd::AsyncFileDialog::new()
                                .set_title("移动到资源文件夹")
                                .set_directory(directory)
                                .pick_folder()
                                .await
                                .map(|f| f.path().to_owned()),
                        )
                    },
                    |(p, d)| Message::MovePicked(p, d),
                );
            }
            Message::MovePicked(path, Some(destination)) => {
                if let (Some(root), Some(kind)) = (self.root(), self.kind()) {
                    if !destination.starts_with(root.join(kind.dir())) {
                        self.error("请选择当前资源类别中的文件夹");
                        return Task::none();
                    }
                    let target = destination.join(path.file_name().unwrap_or_default());
                    return self.change(move || {
                        workspace::relocate(&root, &path, &target)?;
                        Ok("已移动".into())
                    });
                }
            }
            Message::Prompt(kind) => {
                if matches!(kind, PromptKind::Restore) {
                    self.launch_warning = None;
                }
                let (title, input) = match &kind {
                    PromptKind::NewFolder => ("新建文件夹", String::new()),
                    PromptKind::Delete(path) => (
                        "移到回收目录？",
                        path.file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                    ),
                    PromptKind::Rename(path) => (
                        "重命名",
                        path.file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                    ),
                    PromptKind::CreateProfile => ("新建配置预设", String::new()),
                    PromptKind::RenameProfile(_) => ("重命名配置预设", String::new()),
                    PromptKind::DeleteProfile(_) => ("删除配置预设？", String::new()),
                    PromptKind::Restore => {
                        ("恢复原版关卡文件？现有文件会保留备份。", String::new())
                    }
                    PromptKind::ResetData => ("重置当前关卡排行榜？保存后生效。", String::new()),
                    PromptKind::Discard(_) => ("放弃尚未保存的修改？", String::new()),
                };
                self.prompt = Some(Prompt {
                    title: title.into(),
                    input,
                    kind,
                });
            }
            Message::PromptText(value) => {
                if let Some(prompt) = &mut self.prompt {
                    prompt.input = value;
                }
            }
            Message::CancelPrompt => self.prompt = None,
            Message::Confirm => {
                if let Some(prompt) = self.prompt.take() {
                    if let PromptKind::Discard(next) = prompt.kind.clone() {
                        return self.update(Message::Discard(next));
                    }
                    let Some(root) = self.root() else {
                        return Task::none();
                    };
                    match prompt.kind {
                        PromptKind::Discard(next) => return self.update(Message::Discard(next)),
                        PromptKind::NewFolder => {
                            let dir = self.directory.clone();
                            return self.change(move || {
                                workspace::folder(&root, &dir, &prompt.input)?;
                                Ok("已创建文件夹".into())
                            });
                        }
                        PromptKind::Rename(path) => {
                            if prompt.input.contains(['/', '\\', ':'])
                                || prompt.input.trim().is_empty()
                            {
                                self.error("无效的文件名");
                                return Task::none();
                            }
                            let target = path.with_file_name(prompt.input.trim());
                            return self.change(move || {
                                workspace::relocate(&root, &path, &target)?;
                                Ok("已重命名".into())
                            });
                        }
                        PromptKind::Delete(path) => {
                            if self.locked() {
                                return Task::none();
                            }
                            let saved = root
                                .join(".rockoon/trash")
                                .join(workspace::id())
                                .join(path.file_name().unwrap_or_default());
                            self.deleted = Some((path.clone(), saved.clone()));
                            return self.change(move || {
                                workspace::idle(&root)?;
                                workspace::checked(&root, &path)?;
                                workspace::checked(&root, &saved)?;
                                std::fs::create_dir_all(saved.parent().ok_or("无效路径")?)
                                    .map_err(workspace::error)?;
                                std::fs::rename(path, saved).map_err(workspace::error)?;
                                Ok("已移到回收目录".into())
                            });
                        }
                        PromptKind::CreateProfile => {
                            return self
                                .update(Message::Profile(profiles::Action::Create(prompt.input)));
                        }
                        PromptKind::RenameProfile(id) => {
                            return self.update(Message::Profile(profiles::Action::Rename(
                                id,
                                prompt.input,
                            )));
                        }
                        PromptKind::DeleteProfile(id) => {
                            return self.update(Message::Profile(profiles::Action::Delete(id)));
                        }
                        PromptKind::Restore => return self.change(move || vanilla::restore(&root)),
                        PromptKind::ResetData => {
                            if let Some(doc) = &mut self.document {
                                for f in &mut doc.fields {
                                    if f.key.starts_with("/highscores/") && f.group == self.group {
                                        f.value = if f.number {
                                            "0".into()
                                        } else {
                                            "Mr. Default".into()
                                        };
                                    }
                                }
                                doc.dirty = true;
                            }
                        }
                    }
                }
            }
            Message::UndoDelete => {
                if let (Some(root), Some((original, saved))) = (self.root(), self.deleted.clone()) {
                    self.deleted = None;
                    return self.change(move || {
                        workspace::relocate(&root, &saved, &original)?;
                        Ok("已撤销删除".into())
                    });
                }
            }
            Message::Changed(result) => {
                self.mutating = false;
                match result {
                    Ok(message) => {
                        self.notify(message);
                        return self.reload();
                    }
                    Err(e) => self.error(e),
                }
            }
            Message::Toggle(entry) => {
                if let Some(instance) = self.instance.clone() {
                    return self.change(move || {
                        game::toggle_mod(
                            &instance,
                            &game::Resource {
                                path: entry.path,
                                #[cfg(test)]
                                name: entry.name,
                                disabled: entry.disabled,
                            },
                        )?;
                        Ok("已更新模组状态".into())
                    });
                }
            }
            Message::Preview(path) => {
                self.preview_revision += 1;
                let revision = self.preview_revision;
                self.preview_path = Some(path.clone());
                self.skybox = None;
                if self.kind() == Some(Kind::Skies) {
                    return Task::perform(
                        background(move || {
                            crate::sky::Skybox::load(&path).map(std::sync::Arc::new)
                        }),
                        move |r| Message::SkyLoaded(revision, r),
                    );
                }
                return Task::perform(
                    background(move || crate::assets::preview(&path)),
                    move |r| Message::Previewed(revision, r),
                );
            }
            Message::SkyLoaded(revision, result) => {
                if revision == self.preview_revision {
                    match result {
                        Ok(skybox) => {
                            self.skybox = Some(skybox);
                            self.sky_yaw = 0.0;
                            self.sky_pitch = 0.0;
                            return self.update(Message::SkyRender);
                        }
                        Err(e) => self.error(e),
                    }
                }
            }
            Message::SkyAngle(horizontal, value) => {
                if horizontal {
                    self.sky_yaw = value
                } else {
                    self.sky_pitch = value
                }
            }
            Message::SkyRender => {
                if let Some(skybox) = self.skybox.clone() {
                    self.preview_revision += 1;
                    let revision = self.preview_revision;
                    let (yaw, pitch) = (self.sky_yaw, self.sky_pitch);
                    return Task::perform(
                        background(move || Ok(skybox.render(yaw, pitch))),
                        move |r| Message::Previewed(revision, r),
                    );
                }
            }
            Message::Previewed(revision, result) => {
                if revision == self.preview_revision {
                    match result {
                        Ok(image) => self.preview = Some(image),
                        Err(e) => self.error(e),
                    }
                }
            }
            Message::ClosePreview => {
                self.preview_revision += 1;
                self.preview = None;
                self.preview_path = None;
                self.skybox = None;
            }
            Message::Sound(path) => {
                if self.sound.as_ref() == Some(&path) {
                    crate::media::stop();
                    self.sound = None;
                } else {
                    match crate::media::play(&path) {
                        Ok(()) => self.sound = Some(path),
                        Err(e) => self.error(e),
                    }
                }
            }
            Message::StopSound => {
                crate::media::stop();
                self.sound = None;
            }
            Message::FindConfig(path) => {
                if let Some(root) = self.root() {
                    return Task::perform(
                        background(move || {
                            let name = path
                                .file_name()
                                .ok_or("无效模组文件名")?
                                .to_string_lossy()
                                .to_string();
                            let name = name.strip_suffix(".disable").unwrap_or(&name);
                            let stem = std::path::Path::new(name)
                                .file_stem()
                                .unwrap_or_default()
                                .to_string_lossy();
                            config::files(&root)?
                                .into_iter()
                                .find(|p| {
                                    p.file_stem().is_some_and(|n| {
                                        n.to_string_lossy().eq_ignore_ascii_case(&stem)
                                    })
                                })
                                .ok_or("未找到对应模组配置，请先运行一次游戏".into())
                        }),
                        Message::FoundConfig,
                    );
                }
            }
            Message::FoundConfig(result) => match result {
                Ok(path) => {
                    self.page = Page::Config;
                    self.search.clear();
                    self.config_name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    self.config_files.push(path);
                    return self.reload();
                }
                Err(e) => self.error(e),
            },
            Message::UnlockAll => {
                if let Some(doc) = &mut self.document {
                    for field in &mut doc.fields {
                        if field.key.starts_with("/levelLock/") {
                            field.value = "true".into();
                        }
                    }
                    doc.dirty = true;
                }
            }
            Message::SelectConfig(name) => {
                if name == self.config_name {
                    return Task::none();
                }
                if self.document.as_ref().is_some_and(|d| d.dirty) {
                    self.prompt = Some(Prompt {
                        title: "放弃尚未保存的配置？".into(),
                        input: String::new(),
                        kind: PromptKind::Discard(Box::new(Message::SelectConfig(name))),
                    });
                    return Task::none();
                }
                self.config_name = name;
                self.search.clear();
                return self.reload();
            }
            Message::DocumentLoaded(revision, result) => {
                if revision != self.revision {
                    return Task::none();
                }
                self.busy = false;
                match result {
                    Ok((doc, files)) => {
                        self.group = doc.groups().first().cloned().unwrap_or_default();
                        self.document = Some(doc);
                        self.config_files = files;
                    }
                    Err(e) => self.error(e),
                }
            }
            Message::ResetLevel(level) => {
                self.group = format!("第 {} 关", level + 1);
                return self.update(Message::Prompt(PromptKind::ResetData));
            }
            Message::ConfigSection(group, open) => {
                self.config_sections
                    .insert(format!("{}:{group}", self.config_name), open);
            }
            Message::Field(i, value) => {
                if self.locked() {
                    return Task::none();
                }
                if let Some(doc) = &mut self.document
                    && let Some(field) = doc.fields.get_mut(i)
                {
                    field.value = value;
                    doc.dirty = true;
                }
            }
            Message::SaveDocument => {
                if let (Some(root), Some(doc)) = (self.root(), self.document.clone()) {
                    return self.change(move || {
                        doc.save(&root)?;
                        Ok("已保存".into())
                    });
                }
            }
            Message::ProfilesLoaded(revision, result) => {
                if revision != self.revision {
                    return Task::none();
                }
                self.busy = false;
                match result {
                    Ok(index) => self.profile_index = index,
                    Err(e) => self.error(e),
                }
            }
            Message::Profile(action) => {
                if let Some(root) = self.root() {
                    return self.change(move || {
                        profiles::change(&root, action)?;
                        Ok("已更新配置预设".into())
                    });
                }
            }
            Message::Patches(revision, result) => {
                if revision != self.revision {
                    return Task::none();
                }
                self.busy = false;
                match result {
                    Ok(patches) => self.patches = patches,
                    Err(e) => self.error(e),
                }
            }
            Message::Versions(index) => {
                if let Some(id) = self.patches.get(index).and_then(|p| p.package_id) {
                    if self.patch_index != Some(index) {
                        self.versions.clear();
                        self.version = None;
                    }
                    self.close_menus();
                    self.version_menu = Some(index);
                    self.patch_index = Some(index);
                    self.revision += 1;
                    let revision = self.revision;
                    return Task::perform(background(move || jobs::versions(id)), move |r| {
                        Message::VersionsLoaded(revision, r)
                    });
                }
            }
            Message::VersionsLoaded(revision, result) => {
                if revision == self.revision {
                    match result {
                        Ok(versions) => self.versions = versions,
                        Err(e) => self.error(e),
                    }
                }
            }
            Message::Version(version) => self.version = Some(version),
            Message::LatestVersion => self.version = None,
            Message::InstallPatch(index) => {
                if let (Some(root), Some(patch)) = (self.root(), self.patches.get(index).cloned()) {
                    let version = if self.patch_index == Some(index) {
                        self.version.clone()
                    } else {
                        None
                    };
                    return self.start_job(Request::Patch {
                        root,
                        patch,
                        version,
                    });
                }
            }
            Message::InstallGame if !self.locked() => {
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_title("选择空文件夹安装 Ballance")
                            .pick_folder()
                            .await
                            .map(|f| f.path().to_owned())
                    },
                    Message::InstallPicked,
                );
            }
            Message::InstallPicked(Some(root)) => {
                return self.start_job(Request::Game {
                    root,
                    player: self.install_player,
                    bmlplus: self.install_bmlplus,
                    token: workspace::id(),
                });
            }
            Message::InstallChoice(player, bmlplus) => {
                self.install_player = player || bmlplus;
                self.install_bmlplus = bmlplus;
            }
            Message::Sky(sky) => self.sky = sky,
            Message::JobFinished(id, result) => {
                self.finished_jobs.insert(id.clone());
                if result.is_ok()
                    && let Some(job) = self.jobs.iter().find(|j| j.id == id)
                    && let Request::Update { root, .. } = &job.request
                {
                    self.update_path = Some(root.join("native-update.bin"));
                }
                let root = self
                    .jobs
                    .iter()
                    .find(|j| j.id == id)
                    .map(|j| j.request.root().to_owned());
                match result {
                    Ok(message) => self.notify(message),
                    Err(e) => self.error(e),
                }
                if (self.root().is_none()
                    || self
                        .jobs
                        .iter()
                        .find(|j| j.id == id)
                        .is_some_and(|j| matches!(j.request, Request::Game { .. })))
                    && let Some(root) = root.clone()
                    && root.join("Bin/Player.exe").exists()
                {
                    return Task::perform(
                        background(move || game::discover(root)),
                        Message::Loaded,
                    );
                }
                if self.page == Page::Install {
                    return self.reload();
                }
                if let Some(root) = root.filter(|r| self.root().as_ref() == Some(r)) {
                    let requested = root.clone();
                    return Task::perform(background(move || game::discover(root)), move |r| {
                        Message::Rescanned(requested.clone(), r)
                    });
                }
            }
            Message::CancelJob(id) => {
                if let Some(job) = self.jobs.iter().find(|j| j.id == id)
                    && job.progress().cancellable
                {
                    job.cancel();
                }
            }
            Message::RetryJob(id) => {
                if let Some(request) = self
                    .jobs
                    .iter()
                    .find(|j| j.id == id)
                    .map(|j| j.request.clone())
                {
                    return self.start_job(request);
                }
            }
            Message::ClearJobs => self.jobs.retain(|j| !self.finished_jobs.contains(&j.id)),
            Message::Launch(map) if !self.locked() => {
                if let Some(root) = self.root() {
                    self.launching = true;
                    let requested = map.clone();
                    return Task::perform(
                        background(move || {
                            workspace::idle(&root)?;
                            tas::restore(&root)?;
                            let mut diff = vanilla::verify(&root)?;
                            if let Some(project) = tas::autoplay(&root)? {
                                diff.insert(0, format!("TAS 自动播放已开启：{project}"));
                            }
                            Ok(diff)
                        }),
                        move |r| Message::LaunchChecked(requested.clone(), r),
                    );
                }
            }
            Message::LaunchChecked(map, result) => {
                self.launching = false;
                match result {
                    Ok(diff) if diff.is_empty() => return self.launch(map),
                    Ok(diff) => self.launch_warning = Some((map, diff)),
                    Err(e) => self.error(e),
                }
            }
            Message::LaunchAnyway(map) => return self.launch(map),
            Message::Started(result) => {
                self.launching = false;
                match result {
                    Ok(process) => {
                        self.running = Some(process);
                        self.session = Some(Instant::now());
                        self.preferences.last_played = Some(
                            SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                        );
                        let last = self.preferences.last_played;
                        let save = Task::perform(
                            background(move || preferences::record(0, last)),
                            Message::Stats,
                        );
                        if self.preferences.hide_on_launch {
                            return Task::batch([save, iced::window::latest().map(Message::Hide)]);
                        }
                        return save;
                    }
                    Err(e) => self.error(e),
                }
            }
            Message::Stop => {
                if let Some(process) = self.running.clone() {
                    return Task::perform(
                        background(move || {
                            let mut child = process.lock().map_err(workspace::error)?;
                            child.kill().map_err(workspace::error)?;
                            child.wait().map_err(workspace::error)?;
                            Ok(())
                        }),
                        Message::Stopped,
                    );
                }
            }
            Message::Stopped(Err(e)) => self.error(e),
            Message::Incoming => {
                let incoming = crate::system::messages();
                if !incoming.is_empty() {
                    let mut requests = vec![Self::show_window()];
                    for args in incoming {
                        if let Some(path) = args.iter().map(PathBuf::from).find(|p| {
                            p.is_file()
                                && p.extension().is_some_and(|e| e.eq_ignore_ascii_case("tas"))
                        }) {
                            requests.push(self.update(Message::FilesPicked(vec![path])));
                            continue;
                        }
                        if let Some(value) = args.into_iter().find(|s| s.starts_with("rockoon://"))
                        {
                            match jobs::deep_link(&value) {
                                Ok(url) => {
                                    self.import_url = url;
                                    self.page = Page::Tasks;
                                }
                                Err(e) => self.error(e),
                            }
                        }
                    }
                    return Task::batch(requests.drain(..));
                }
            }
            Message::Tick => {
                if self.quitting && self.jobs.iter().all(|j| j.progress().done) && !self.mutating {
                    return self.update(Message::Exit);
                }
                let status = self.running.as_ref().map(|p| {
                    p.lock()
                        .map_err(workspace::error)
                        .and_then(|mut c| c.try_wait().map_err(workspace::error))
                });
                match status {
                    Some(Ok(Some(_))) => {
                        self.running = None;
                        let elapsed = self
                            .session
                            .take()
                            .map(|s| s.elapsed().as_secs())
                            .unwrap_or(0);
                        let last = self.preferences.last_played;
                        let record = Task::perform(
                            background(move || preferences::record(elapsed, last)),
                            Message::Stats,
                        );
                        if self.tas_playing {
                            self.tas_playing = false;
                            self.tas_embedded = false;
                            let _ = rockoon_platform::embed::detach_game_window();
                            if let Some(root) = self.root() {
                                return Task::batch([
                                    record,
                                    Task::perform(
                                        background(move || tas::restore(&root)),
                                        Message::Saved,
                                    ),
                                    Self::show_window(),
                                ]);
                            }
                        }
                        return Task::batch([record, Self::show_window()]);
                    }
                    Some(Err(e)) => {
                        self.running = None;
                        self.error(e);
                        return Self::show_window();
                    }
                    _ => {}
                }
            }
            Message::Hide(Some(id)) => {
                self.home_active = false;
                return iced::window::set_mode(id, iced::window::Mode::Hidden);
            }
            Message::Show(Some(id)) => {
                self.home_active = true;
                return iced::window::set_mode(id, iced::window::Mode::Windowed)
                    .chain(iced::window::gain_focus(id));
            }
            Message::ColorMode(mode) => {
                self.preferences.color_mode = mode;
                return self.save_preferences();
            }
            Message::PrefBool(key, value) => {
                match key.as_str() {
                    "hide" => self.preferences.hide_on_launch = value,
                    "map" => self.preferences.map_only = value,
                    "tas" => self.preferences.enable_tas = value,
                    _ => {}
                }
                return self.save_preferences();
            }
            Message::PrefText(key, value) => {
                match key.as_str() {
                    "motd" => self.preferences.motd = value,
                    "language" => {
                        self.preferences.language =
                            if value == "English" { "en" } else { "zh" }.into();
                        crate::l10n::set(&self.preferences.language);
                    }
                    "slot" => match value.parse::<u8>() {
                        Ok(slot) if slot <= 13 => self.preferences.level_slot = slot,
                        _ => {
                            self.error("关卡槽位需要 0–13；0 表示随机");
                            return Task::none();
                        }
                    },
                    _ => {}
                }
                return self.save_preferences();
            }
            Message::Saved(Err(e)) => self.error(e),
            Message::Tas(action) => return self.tas_action(action),
            Message::TasOpened(result) => {
                self.tas_busy = false;
                match result {
                    Ok(project) => {
                        self.tas = project;
                        self.tas_offset = 0;
                    }
                    Err(e) => self.error(e),
                }
            }
            Message::TasPath(Some(path)) => {
                let project = self.tas.clone();
                let target = path.clone();
                self.tas_busy = true;
                return Task::perform(background(move || project.save(&target)), move |r| {
                    Message::TasSaved(path.clone(), r)
                });
            }
            Message::TasPath(None) => self.tas_busy = false,
            Message::TasSaved(path, result) => {
                self.tas_busy = false;
                match result {
                    Ok(()) => {
                        self.tas.path = Some(path);
                        self.tas.dirty = false;
                    }
                    Err(e) => self.error(e),
                }
            }
            Message::TasStart(value) => self.tas_start = value,
            Message::TasEnd(value) => self.tas_end = value,
            Message::TasCount(value) => self.tas_count = value,
            Message::TasOffset(offset) => self.tas_offset = offset,
            Message::Event(iced::Event::Window(iced::window::Event::FileDropped(path))) => {
                return self.update(Message::FilesPicked(vec![path]));
            }
            Message::OpenTasProject(path) => {
                let root = self.root();
                return Task::perform(
                    background(move || {
                        let mut command = std::process::Command::new(
                            std::env::current_exe().map_err(workspace::error)?,
                        );
                        command.arg("--tas-editor");
                        if let Some(root) = root {
                            command.arg(root);
                        }
                        command.arg(path);
                        command.spawn().map_err(workspace::error)?;
                        Ok(())
                    }),
                    Message::Saved,
                );
            }
            Message::OpenTas => {
                let root = self.root();
                return Task::perform(
                    background(move || {
                        let executable = std::env::current_exe().map_err(workspace::error)?;
                        let mut command = std::process::Command::new(executable);
                        command.arg("--tas-editor");
                        if let Some(root) = root {
                            command.arg(root);
                        }
                        command.spawn().map_err(workspace::error)?;
                        Ok(())
                    }),
                    Message::Saved,
                );
            }
            Message::TasLevel(level) => self.tas_level = level,
            Message::TasFps(value) => {
                self.tas_fps = value.clone();
                if self.tas_playing
                    && let (Some(process), Ok(fps)) = (self.running.clone(), value.parse::<f32>())
                {
                    return Task::perform(
                        background(move || {
                            let pid = process.lock().map_err(workspace::error)?.id();
                            rockoon_platform::frame_limit::set_game_frame_limit(pid, fps)
                                .map_err(workspace::error)
                        }),
                        Message::TasFrameLimit,
                    );
                }
            }
            Message::TasPlay(from_current) => {
                if self.locked() {
                    return Task::none();
                }
                let Some(instance) = self.instance.clone() else {
                    self.error("请先选择游戏目录");
                    return Task::none();
                };
                if self.tas.frames.is_empty() {
                    self.error("TAS 项目没有帧");
                    return Task::none();
                }
                let project = self.tas.clone();
                let level = self.tas_level;
                let skip = if from_current {
                    self.tas_start.parse().unwrap_or(0)
                } else {
                    0
                };
                let fps = match self.tas_fps.parse::<f32>() {
                    Ok(f) if f.is_finite() && (1.0..=5000.0).contains(&f) => f,
                    _ => {
                        self.error("目标帧率需要 1–5000");
                        return Task::none();
                    }
                };
                let prefs = self.preferences.clone();
                self.launching = true;
                return Task::perform(
                    background(move || {
                        jobs::ensure_io(&instance.root)?;
                        tas::prepare(&instance.root, &project, level, skip)?;
                        match game::launch_tas(&instance, &prefs, fps) {
                            Ok(process) => Ok(process),
                            Err(e) => {
                                let _ = tas::restore(&instance.root);
                                Err(e)
                            }
                        }
                    }),
                    Message::TasPrepared,
                );
            }
            Message::TasPrepared(result) => {
                self.launching = false;
                match result {
                    Ok(process) => {
                        self.running = Some(process);
                        self.tas_playing = true;
                        self.session = Some(Instant::now());
                        self.preferences.last_played = Some(
                            SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs(),
                        );
                        let last = self.preferences.last_played;
                        return Task::batch([
                            iced::window::latest().map(Message::TasHost),
                            Task::perform(
                                background(move || preferences::record(0, last)),
                                Message::Stats,
                            ),
                        ]);
                    }
                    Err(e) => self.error(e),
                }
            }
            Message::TasHost(Some(id)) => {
                return iced::window::raw_id::<Message>(id).map(Message::TasRaw);
            }
            Message::TasRaw(host) => {
                return iced::window::latest().then(move |id| match id {
                    Some(id) => iced::window::scale_factor(id)
                        .map(move |scale| Message::TasScale(host, scale)),
                    None => Task::none(),
                });
            }
            Message::TasScale(host, scale) => {
                self.tas_host = Some((host, scale));
                return crate::bounds::viewport()
                    .map(move |rect| Message::TasBounds(host, scale, rect));
            }
            Message::TasBounds(host, scale, Some(rect)) => {
                if let Some(process) = self.running.clone() {
                    let attached = self.tas_embedded;
                    return Task::perform(
                        background(move || {
                            let rect = [
                                (rect.x * scale).round() as i32,
                                (rect.y * scale).round() as i32,
                                (rect.width * scale).round() as i32,
                                (rect.height * scale).round() as i32,
                            ];
                            if attached {
                                rockoon_platform::embed::move_game_window(
                                    rect[0], rect[1], rect[2], rect[3],
                                )
                                .map(|_| 0)
                                .map_err(workspace::error)
                            } else {
                                let pid = process.lock().map_err(workspace::error)?.id();
                                rockoon_platform::embed::attach_game_window(
                                    host as usize,
                                    pid,
                                    rect[0],
                                    rect[1],
                                    rect[2],
                                    rect[3],
                                )
                                .map_err(workspace::error)
                            }
                        }),
                        Message::TasEmbedded,
                    );
                }
            }
            Message::TasEmbedded(result) => match result {
                Ok(_) => self.tas_embedded = true,
                Err(e) => self.error(e),
            },
            Message::TasDetach => {
                self.tas_embedded = false;
                return Task::perform(
                    background(|| {
                        rockoon_platform::embed::detach_game_window().map_err(workspace::error)
                    }),
                    Message::Saved,
                );
            }
            Message::TasInteractive(value) => {
                self.tas_interactive = value;
                return Task::perform(
                    background(move || {
                        rockoon_platform::embed::set_game_window_interactive(value)
                            .map_err(workspace::error)
                    }),
                    Message::Saved,
                );
            }
            Message::TasFrameLimit(Err(e)) => self.error(e),
            Message::TasDisabled(map, result) => {
                self.mutating = false;
                match result {
                    Ok(()) => return self.launch(map),
                    Err(e) => self.error(e),
                }
            }
            Message::DisableAutoplay(map) => {
                if self.locked() {
                    return Task::none();
                }
                self.mutating = true;
                if let Some(root) = self.root() {
                    return Task::perform(
                        background(move || tas::disable_autoplay(&root)),
                        move |r| Message::TasDisabled(map.clone(), r),
                    );
                }
            }
            Message::RegisterProtocol => {
                return Task::perform(
                    background(crate::system::register_protocol),
                    Message::ProtocolRegistered,
                );
            }
            Message::ProtocolRegistered(result) => match result {
                Ok(()) => {
                    self.protocol_ready = true;
                    self.notify("已启用一键安装，可在资源站选择资源并点击“一键安装”");
                }
                Err(error) => self.error(error),
            },
            Message::OpenHub if !self.opening_hub => {
                self.opening_hub = true;
                return Task::perform(
                    background(|| {
                        jobs::hub("/packages?limit=1")?;
                        game::open_folder(&PathBuf::from(jobs::origin()))
                    }),
                    Message::HubOpened,
                );
            }
            Message::HubOpened(result) => {
                self.opening_hub = false;
                if let Err(error) = result {
                    self.error(error);
                }
            }
            Message::AutoUpdate(Ok(Some(release))) => {
                self.notify(format!("有新的原生版本：{}", release.version));
                self.update_release = Some(release);
            }
            Message::AutoPatches(Ok(updates)) => {
                if !updates.is_empty() {
                    self.notify(format!(
                        "补丁有更新：{}",
                        updates
                            .iter()
                            .map(|p| p.name.as_str())
                            .collect::<Vec<_>>()
                            .join("、")
                    ));
                }
            }
            Message::CheckUpdate => {
                self.notify("正在检查原生版更新…");
                return Task::perform(background(crate::updates::check), Message::UpdateChecked);
            }
            Message::UpdateChecked(result) => match result {
                Ok(Some(release)) => {
                    self.notify(format!("可更新到 {}", release.version));
                    self.update_release = Some(release);
                }
                Ok(None) => self.notify("暂无较新的原生版发布"),
                Err(e) => self.error(e),
            },
            Message::DownloadUpdate => {
                if let Some(release) = self.update_release.clone() {
                    match preferences::directory() {
                        Ok(root) => {
                            return self.start_job(Request::Update {
                                root: root.join("updates"),
                                release,
                            });
                        }
                        Err(e) => self.error(e),
                    }
                }
            }
            Message::ApplyUpdate => {
                if let Some(path) = &self.update_path {
                    match crate::system::install_update(path) {
                        Ok(()) => return self.update(Message::Exit),
                        Err(e) => self.error(e),
                    }
                }
            }
            #[cfg(not(windows))]
            Message::WineSetup => {
                if let Ok(root) = preferences::directory() {
                    return self.start_job(Request::Wine { root });
                }
            }
            #[cfg(not(windows))]
            Message::WineChoose => {
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_title("选择 Wine .app 或根目录")
                            .pick_folder()
                            .await
                            .map(|p| p.path().to_owned())
                    },
                    Message::WineChosen,
                );
            }
            #[cfg(not(windows))]
            Message::WineChosen(Some(path)) => {
                return Task::perform(
                    background(move || {
                        rockoon_platform::wine::set_custom_wine(
                            rockoon_platform::Context::default(),
                            Some(path.display().to_string()),
                        )
                        .map(|_| ())
                        .map_err(workspace::error)
                    }),
                    Message::Saved,
                );
            }
            #[cfg(not(windows))]
            Message::WineConfig => {
                return Task::perform(
                    background(|| {
                        rockoon_platform::wine::open_wine_config(
                            rockoon_platform::Context::default(),
                        )
                        .map_err(workspace::error)
                    }),
                    Message::Saved,
                );
            }
            #[cfg(target_os = "macos")]
            Message::Rosetta => {
                return Task::perform(
                    background(|| {
                        rockoon_platform::wine::install_rosetta().map_err(workspace::error)
                    }),
                    Message::Saved,
                );
            }
            Message::Quit => {
                if self.document.as_ref().is_some_and(|d| d.dirty) || self.tas.dirty {
                    self.prompt = Some(Prompt {
                        title: "放弃尚未保存或导出的修改并退出？".into(),
                        input: String::new(),
                        kind: PromptKind::Discard(Box::new(Message::Exit)),
                    });
                    return Task::none();
                }
                return self.update(Message::Exit);
            }
            Message::Exit => {
                if self.mutating || self.tas_busy || self.jobs.iter().any(|j| !j.progress().done) {
                    self.quitting = true;
                    for job in &self.jobs {
                        if !job.progress().done && job.progress().cancellable {
                            job.cancel();
                        }
                    }
                    self.notify("正在结束后台操作…");
                    return Task::none();
                }
                crate::media::stop();
                let _ = rockoon_platform::embed::detach_game_window();
                let elapsed = self
                    .session
                    .take()
                    .map(|s| s.elapsed().as_secs())
                    .unwrap_or(0);
                let last = self.preferences.last_played;
                return Task::batch([
                    self.save_preferences(),
                    Task::perform(
                        background(move || preferences::record(elapsed, last)),
                        Message::Stats,
                    ),
                ])
                .chain(iced::exit());
            }
            Message::Event(iced::Event::Window(iced::window::Event::Focused)) => {
                self.home_active = true;
            }
            Message::Event(iced::Event::Window(iced::window::Event::Unfocused)) => {
                self.home_active = false;
            }
            Message::Event(iced::Event::Window(iced::window::Event::Resized(_))) => {
                if self.tas_embedded
                    && let Some((host, scale)) = self.tas_host
                {
                    return crate::bounds::viewport()
                        .map(move |rect| Message::TasBounds(host, scale, rect));
                }
            }
            Message::Event(iced::Event::Window(iced::window::Event::CloseRequested)) => {
                return self.update(Message::Quit);
            }
            Message::Stats(result) => match result {
                Ok(stats) => {
                    self.preferences.playtime = stats.total;
                    self.preferences.last_played = stats.last;
                    return self.save_preferences();
                }
                Err(e) => self.error(e),
            },
            Message::Dismiss => {
                self.notice = None;
                self.launch_warning = None;
            }
            _ => {}
        }
        Task::none()
    }
    fn sky_letter(&self) -> String {
        let levels = [
            "L", "E", "A", "F", "C", "H", "D", "G", "K", "B", "J", "I", "F",
        ];
        self.sky
            .strip_prefix("第 ")
            .and_then(|s| s.split_whitespace().next())
            .and_then(|n| n.parse::<usize>().ok())
            .and_then(|n| levels.get(n.wrapping_sub(1)))
            .unwrap_or(&"X")
            .to_string()
    }
    fn tas_action(&mut self, action: TasAction) -> Task<Message> {
        if self.tas_busy {
            return Task::none();
        }
        let range = || -> Result<(usize, usize), String> {
            Ok((
                self.tas_start.parse().map_err(|_| "起始帧无效")?,
                self.tas_end.parse().map_err(|_| "结束帧无效")?,
            ))
        };
        if self.tas_playing
            && !matches!(
                action,
                TasAction::Save | TasAction::SaveAs | TasAction::Copy
            )
        {
            return Task::none();
        }
        let result = match action {
            TasAction::New => {
                if self.tas.dirty {
                    self.error("请先导出当前 TAS，再新建项目");
                    return Task::none();
                }
                self.tas = tas::Project::default();
                self.tas.insert(0, 240)
            }
            TasAction::Open => {
                if self.tas.dirty {
                    self.error("请先导出当前 TAS，再打开其他项目");
                    return Task::none();
                }
                self.tas_busy = true;
                return Task::perform(
                    async {
                        match rfd::AsyncFileDialog::new()
                            .add_filter("Ballance TAS", &["tas"])
                            .pick_file()
                            .await
                        {
                            Some(file) => background(move || tas::Project::read(file.path())).await,
                            None => Err("已取消打开".into()),
                        }
                    },
                    Message::TasOpened,
                );
            }
            TasAction::Save | TasAction::SaveAs => {
                if matches!(action, TasAction::Save)
                    && let Some(path) = self.tas.path.clone()
                {
                    return self.update(Message::TasPath(Some(path)));
                }
                self.tas_busy = true;
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_file_name("project.tas")
                            .add_filter("Ballance TAS", &["tas"])
                            .save_file()
                            .await
                            .map(|f| f.path().to_owned())
                    },
                    Message::TasPath,
                );
            }
            TasAction::Undo => {
                self.tas.undo();
                Ok(())
            }
            TasAction::Redo => {
                self.tas.redo();
                Ok(())
            }
            TasAction::Toggle(frame, key) => {
                self.tas.toggle(frame, key);
                Ok(())
            }
            TasAction::Copy => range().and_then(|(start, end)| self.tas.copy(start, end)),
            TasAction::Cut => range().and_then(|(start, end)| self.tas.cut(start, end)),
            TasAction::Paste => range().and_then(|(start, _)| self.tas.paste(start)),
            TasAction::Insert => range().and_then(|(start, _)| {
                self.tas_count
                    .parse::<usize>()
                    .map_err(|_| "插入帧数无效".into())
                    .and_then(|n| self.tas.insert(start, n))
            }),
            TasAction::Range(key, enabled) => {
                range().and_then(|(start, end)| self.tas.range(start, end, key, enabled))
            }
        };
        if let Err(e) = result {
            self.error(e);
        }
        Task::none()
    }
    pub fn theme(&self) -> iced::Theme {
        style::theme(self.preferences.color_mode)
    }
    pub fn subscription(&self) -> Subscription<Message> {
        let mut subscriptions = vec![
            iced::event::listen().map(Message::Event),
            Subscription::run(crate::system::events).map(|_| Message::Incoming),
        ];
        if self.running.is_some() || self.jobs.iter().any(|j| !j.progress().done) || self.quitting {
            subscriptions
                .push(iced::time::every(Duration::from_millis(500)).map(|_| Message::Tick));
        }
        if self.notice.is_some() {
            subscriptions
                .push(iced::time::every(Duration::from_millis(50)).map(|_| Message::NoticeTick));
        }
        Subscription::batch(subscriptions)
    }
}
