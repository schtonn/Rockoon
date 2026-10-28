use super::views::{action, scrollable, separator};
use super::*;
use crate::blender;
use crate::l10n::ui_text as text;
use iced::{
    Alignment, Fill,
    widget::{column, container, row, space},
};

#[derive(Clone, Debug)]
pub enum Action {
    Refresh,
    Select(Option<String>),
    Choose,
    Picked(Option<String>),
    Scanned(u64, Result<Scan, String>),
    Release(u64, Result<blender::BbpRelease, String>),
    Install,
    Configure,
    Configured(Result<(), String>),
    OpenMap(PathBuf),
    MapOpened(Result<(), String>),
}
#[derive(Clone, Debug)]
pub struct Scan {
    candidates: Vec<String>,
    selected: Option<String>,
    probe: Result<Option<blender::BlenderProbe>, String>,
}
#[derive(Default)]
pub(super) struct State {
    pub(super) online: bool,
    candidates: Vec<String>,
    selected: Option<String>,
    probe: Option<blender::BlenderProbe>,
    latest: Option<blender::BbpRelease>,
    loading: bool,
    release_loading: bool,
    error: Option<String>,
    release_error: Option<String>,
    generation: u64,
    scanned: bool,
    pending_map: Option<PathBuf>,
}
fn message(action: Action) -> Message {
    Message::Mapping(action)
}
fn link(label: &str, url: &str) -> Element<'static, Message> {
    action(
        label,
        Message::Hub(super::catalog::Action::Link(url.into())),
        true,
    )
}
impl App {
    pub(super) fn refresh_mapping(&mut self, release: bool) -> Task<Message> {
        self.mapping.generation += 1;
        self.mapping.loading = true;
        self.mapping.error = None;
        let generation = self.mapping.generation;
        let selected = self.preferences.blender_path.clone();
        let scan = Task::perform(
            background(move || {
                let candidates = blender::blender_find();
                let selected = selected
                    .filter(|p| p.exists())
                    .map(|p| p.to_string_lossy().into_owned())
                    .or_else(|| candidates.first().cloned());
                let probe = selected
                    .as_ref()
                    .map(|exe| blender::blender_probe(exe.clone()).map_err(workspace::error))
                    .transpose();
                Ok(Scan {
                    candidates,
                    selected,
                    probe,
                })
            }),
            move |result| message(Action::Scanned(generation, result)),
        );
        if release || self.mapping.latest.is_none() {
            self.mapping.release_loading = true;
            self.mapping.release_error = None;
            Task::batch([
                scan,
                Task::perform(background(blender::latest_release), move |result| {
                    message(Action::Release(generation, result))
                }),
            ])
        } else {
            scan
        }
    }
    fn mapping_installing(&self) -> bool {
        self.jobs
            .iter()
            .any(|j| matches!(j.request, Request::Bbp { .. }) && !j.progress().done)
    }
    pub(super) fn update_mapping(&mut self, action: Action) -> Task<Message> {
        match action {
            Action::Refresh if !self.mapping_installing() => return self.refresh_mapping(true),
            Action::Choose => {
                self.close_menus();
                return Task::perform(
                    async {
                        #[cfg(target_os = "macos")]
                        let picked = rfd::AsyncFileDialog::new()
                            .set_title(crate::l10n::text("选择 Blender.app"))
                            .pick_folder()
                            .await;
                        #[cfg(not(target_os = "macos"))]
                        let picked = rfd::AsyncFileDialog::new()
                            .set_title(crate::l10n::text("选择 Blender 可执行文件"))
                            .pick_file()
                            .await;
                        picked.map(|p| p.path().to_string_lossy().into_owned())
                    },
                    |p| message(Action::Picked(p)),
                );
            }
            Action::Picked(Some(exe)) | Action::Select(Some(exe)) => {
                self.preferences.blender_path = Some(exe.into());
                return Task::batch([self.save_preferences(), self.refresh_mapping(false)]);
            }
            Action::Select(None) => {
                self.preferences.blender_path = None;
                return Task::batch([self.save_preferences(), self.refresh_mapping(false)]);
            }
            Action::Scanned(generation, result) if generation == self.mapping.generation => {
                self.mapping.loading = false;
                self.mapping.scanned = true;
                match result {
                    Ok(scan) => {
                        self.mapping.candidates = scan.candidates;
                        self.mapping.selected = scan.selected;
                        match scan.probe {
                            Ok(probe) => self.mapping.probe = probe,
                            Err(error) => {
                                self.mapping.probe = None;
                                self.mapping.error = Some(error);
                            }
                        }
                    }
                    Err(error) => self.mapping.error = Some(error),
                }
                if let Some(map) = self.mapping.pending_map.take() {
                    return self.update_mapping(Action::OpenMap(map));
                }
            }
            Action::Release(generation, result) if generation == self.mapping.generation => {
                self.mapping.release_loading = false;
                match result {
                    Ok(release) => {
                        self.mapping.latest = Some(release);
                        self.mapping.release_error = None;
                    }
                    Err(error) => self.mapping.release_error = Some(error),
                }
            }
            Action::Install if !self.locked() && !self.mapping.loading => {
                let (Some(probe), Some(exe), Some(release)) = (
                    &self.mapping.probe,
                    &self.mapping.selected,
                    &self.mapping.latest,
                ) else {
                    return Task::none();
                };
                if probe.extensions_dir.is_empty()
                    || !blender::version_at_least(&probe.version, "4.5")
                {
                    self.mapping.error = Some("安装最新版 BBP 需要 Blender 4.5 或更新版本".into());
                    return Task::none();
                }
                if probe.bbp.iter().any(|b| b.module != blender::MODULE) {
                    self.mapping.error = Some("请先在 Blender 中卸载旧版 BBP".into());
                    return Task::none();
                }
                let request = Request::Bbp {
                    root: probe.extensions_dir.clone().into(),
                    exe: exe.clone(),
                    release: release.clone(),
                    textures: self
                        .root()
                        .map(|r| r.join("Textures").to_string_lossy().into_owned())
                        .unwrap_or_default(),
                };
                return self.start_job(request);
            }
            Action::Configure if !self.locked() && !self.mapping.loading => {
                let Some(exe) = self.mapping.selected.clone() else {
                    return Task::none();
                };
                let textures = self
                    .root()
                    .map(|r| r.join("Textures").to_string_lossy().into_owned())
                    .unwrap_or_default();
                self.mapping.loading = true;
                return Task::perform(
                    background(move || {
                        blender::blender_configure_bbp(exe, textures).map_err(workspace::error)
                    }),
                    |r| message(Action::Configured(r)),
                );
            }
            Action::Configured(result) => {
                self.mapping.loading = false;
                match result {
                    Ok(()) => {
                        self.notify("BBP 已启用并使用当前游戏的材质");
                        return self.refresh_mapping(false);
                    }
                    Err(error) => self.mapping.error = Some(error),
                }
            }
            Action::OpenMap(map) => {
                self.close_menus();
                if !self.mapping.scanned || self.mapping.loading {
                    self.mapping.pending_map = Some(map);
                    if !self.mapping.loading {
                        return self.refresh_mapping(false);
                    }
                    return Task::none();
                }
                let ready = self
                    .mapping
                    .probe
                    .as_ref()
                    .is_some_and(|p| p.bbp.iter().any(|b| b.module == blender::MODULE));
                if !ready {
                    if self.page != Page::Mapping {
                        self.mapping.online = self.page == Page::Hub;
                    }
                    self.page = Page::Mapping;
                    self.notify("请先在制图页配置 Blender 和 BBP");
                    return Task::none();
                }
                let Some(exe) = self.mapping.selected.clone() else {
                    return Task::none();
                };
                let textures = self
                    .root()
                    .map(|r| r.join("Textures").to_string_lossy().into_owned())
                    .unwrap_or_default();
                return Task::perform(
                    background(move || {
                        if !map.is_file() {
                            return Err("地图文件已不存在".into());
                        }
                        blender::blender_open_map(exe, map.to_string_lossy().into_owned(), textures)
                            .map_err(workspace::error)
                    }),
                    |r| message(Action::MapOpened(r)),
                );
            }
            Action::MapOpened(Ok(())) => self.notify("已打开 Blender，正在导入地图"),
            Action::MapOpened(Err(error)) => self.error(error),
            _ => {}
        }
        Task::none()
    }
    pub(super) fn mapping_view(&self) -> Element<'_, Message> {
        let state = &self.mapping;
        let enabled = !state.loading && !self.locked();
        let status = if state.loading {
            "检测中…".into()
        } else {
            state
                .probe
                .as_ref()
                .map(|p| p.version.clone())
                .unwrap_or_else(|| "未找到 Blender".into())
        };
        let mut content = column![
            row![
                text("Blender").size(18).width(Fill),
                text(status).size(14),
                action(
                    "重新检测",
                    message(Action::Refresh),
                    !state.loading && !self.mapping_installing()
                )
            ]
            .spacing(12)
            .align_y(Alignment::Center)
        ]
        .spacing(12);
        if let Some(selected) = &state.selected {
            let mut choices = state.candidates.clone();
            if !choices.contains(selected) {
                choices.push(selected.clone());
            }
            content = content.push(
                iced::widget::pick_list(choices, Some(selected.clone()), |v| {
                    message(Action::Select(Some(v)))
                })
                .text_size(14)
                .padding(6)
                .width(Fill)
                .style(style::select)
                .menu_style(style::select_menu),
            );
        }
        let mut controls = row![
            link("下载 Blender", "https://www.blender.org/download/lts/"),
            action("选择 Blender", message(Action::Choose), enabled)
        ]
        .spacing(8);
        if self.preferences.blender_path.is_some() {
            controls = controls.push(action("自动检测", message(Action::Select(None)), enabled));
        }
        content = content.push(controls).push(separator());
        let bbp = state
            .probe
            .as_ref()
            .and_then(|p| p.bbp.iter().find(|b| b.module == blender::MODULE));
        let status = bbp
            .map(|b| format!("{}{}", b.version, if b.enabled { "" } else { "（未启用）" }))
            .unwrap_or_else(|| "未安装".into());
        content = content.push(
            row![
                crate::hint::label(
                    "Ballance Blender Plugin",
                    "用于在 Blender 中导入和制作 Ballance 地图。"
                ),
                space::horizontal(),
                text(status).size(14)
            ]
            .align_y(Alignment::Center),
        );
        if let Some(release) = &state.latest {
            content = content.push(text(format!("可安装版本 {}", release.tag)).size(14));
        }
        if let Some(probe) = &state.probe {
            if !blender::version_at_least(&probe.version, "4.5") {
                content = content.push(text("安装最新版 BBP 需要 Blender 4.5 或更新版本").size(14));
            }
            for legacy in probe.bbp.iter().filter(|b| b.module != blender::MODULE) {
                content = content.push(text(format!("请先卸载旧版 BBP：{}", legacy.path)).size(14));
            }
        }
        if let Some(bbp) = bbp
            && !bbp.texture_folder.is_empty()
        {
            content = content.push(text(format!("材质目录  {}", bbp.texture_folder)).size(14));
        }
        let installing = self.mapping_installing();
        let compatible = state.probe.as_ref().is_some_and(|p| {
            !p.extensions_dir.is_empty()
                && blender::version_at_least(&p.version, "4.5")
                && p.bbp.iter().all(|b| b.module == blender::MODULE)
        });
        content = content.push(row![
            action(if installing { "安装中…" } else if bbp.is_some() { "更新或重装 BBP" } else { "安装 BBP" },
                message(Action::Install), enabled && compatible && !installing && state.latest.is_some()),
            action("使用当前游戏材质", message(Action::Configure), enabled && bbp.is_some() && self.root().is_some()),
            link("制图手册", "https://ghomist.github.io/ballance-mapping-manual/mapping/intro/installations.html"),
            link("GitHub", "https://github.com/yyc12345/BallanceBlenderHelper"),
        ].spacing(8).align_y(Alignment::Center));
        if let Some(error) = &state.error {
            content = content.push(text(error).size(14));
        }
        if let Some(error) = &state.release_error {
            content = content.push(text(format!("无法读取 BBP 安装版本：{error}")).size(14));
        }
        content =
            content
                .push(separator())
                .push(action("浏览本地地图", Message::MappingMaps, true));
        scrollable(
            container(content)
                .max_width(880)
                .padding(iced::Padding::ZERO.right(12)),
        )
        .height(Fill)
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stale_blender_detection_cannot_replace_a_newer_selection() {
        let (mut app, _) = App::new();
        app.mapping.generation = 2;
        app.mapping.selected = Some("new Blender".into());
        let scan = Scan {
            candidates: vec![],
            selected: Some("old Blender".into()),
            probe: Ok(None),
        };
        let _ = app.update_mapping(Action::Scanned(1, Ok(scan)));
        assert_eq!(app.mapping.selected.as_deref(), Some("new Blender"));
    }
}
