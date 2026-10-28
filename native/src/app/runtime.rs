//! Wine onboarding and settings share the same status and actions.
use super::views::{action, scrollable};
use super::*;
use iced::{
    Fill,
    widget::{column, container, text},
};
use rockoon_platform::wine::{self, WineStatus};

#[derive(Clone, Debug)]
pub enum Action {
    Refresh,
    Scanned(Result<WineStatus, String>),
    Prepare,
    Choose,
    Chosen(Option<PathBuf>),
    UseBundled,
    Configure,
    Rosetta,
    Completed(Result<WineStatus, String>),
}
pub(super) struct State {
    pub status: Option<WineStatus>,
    pub loading: bool,
    pub working: bool,
    pub error: Option<String>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            status: None,
            loading: true,
            working: false,
            error: None,
        }
    }
}
fn ready(status: &WineStatus) -> bool {
    status.rosetta && status.installed && status.prefix_ready
}
impl State {
    pub fn needs_setup(&self) -> bool {
        cfg!(not(windows)) && !self.status.as_ref().is_some_and(ready)
    }
}
pub(super) fn scan() -> Result<WineStatus, String> {
    wine::wine_status(rockoon_platform::Context::default()).map_err(workspace::error)
}
fn message(action: Action) -> Message {
    Message::Wine(action)
}
impl App {
    pub(super) fn refresh_wine(&mut self) -> Task<Message> {
        if self.wine.loading || self.wine.working {
            return Task::none();
        }
        self.wine.loading = true;
        Task::perform(background(scan), |r| message(Action::Scanned(r)))
    }
    pub(super) fn update_wine(&mut self, action: Action) -> Task<Message> {
        match action {
            Action::Scanned(result) => {
                self.wine.loading = false;
                match result {
                    Ok(status) => self.wine.status = Some(status),
                    Err(error) => self.wine.error = Some(error),
                }
                return Task::none();
            }
            Action::Completed(result) => {
                self.wine.working = false;
                self.wine.loading = false;
                match result {
                    Ok(status) => {
                        self.wine.status = Some(status);
                        self.wine.error = None;
                    }
                    Err(error) => self.wine.error = Some(error),
                }
                return Task::none();
            }
            Action::Chosen(path) => {
                self.wine.working = false;
                if let Some(path) = path {
                    self.wine.working = true;
                    return Task::perform(
                        background(move || {
                            wine::set_custom_wine(
                                rockoon_platform::Context::default(),
                                Some(path.display().to_string()),
                            )
                            .map_err(workspace::error)
                        }),
                        |r| message(Action::Completed(r)),
                    );
                }
                return Task::none();
            }
            _ => {}
        }
        if self.locked() || self.wine.loading {
            return Task::none();
        }
        self.wine.error = None;
        match action {
            Action::Refresh => return self.refresh_wine(),
            Action::Prepare =>
            {
                #[cfg(not(windows))]
                if let Ok(root) = preferences::directory() {
                    return self.start_job(Request::Wine { root });
                }
            }
            Action::Choose => {
                self.wine.working = true;
                return Task::perform(
                    async {
                        rfd::AsyncFileDialog::new()
                            .set_title(crate::l10n::text("选择 Wine .app 或根目录"))
                            .pick_folder()
                            .await
                            .map(|p| p.path().to_owned())
                    },
                    |p| message(Action::Chosen(p)),
                );
            }
            Action::UseBundled | Action::Configure | Action::Rosetta => {
                self.wine.working = true;
                return Task::perform(
                    background(move || {
                        let context = rockoon_platform::Context::default();
                        match action {
                            Action::UseBundled => {
                                wine::set_custom_wine(context, None).map_err(workspace::error)?;
                            }
                            Action::Configure => {
                                wine::open_wine_config(context).map_err(workspace::error)?
                            }
                            Action::Rosetta => wine::install_rosetta().map_err(workspace::error)?,
                            _ => unreachable!(),
                        }
                        scan()
                    }),
                    |r| message(Action::Completed(r)),
                );
            }
            _ => {}
        }
        Task::none()
    }
    pub(super) fn wine_guide(&self) -> Element<'_, Message> {
        let body = column![
            text(crate::l10n::text("准备运行环境")).size(24),
            text(crate::l10n::text(
                "Ballance 需要通过 Wine 运行，完成下面的准备后即可启动游戏。"
            ))
            .size(14),
            self.wine_panel(false),
        ]
        .spacing(18);
        scrollable(
            container(container(body).max_width(560))
                .padding(28)
                .center_x(Fill)
                .center_y(Fill),
        )
        .height(Fill)
        .into()
    }
    pub(super) fn wine_panel(&self, advanced: bool) -> Element<'_, Message> {
        let mut body = column![].spacing(0);
        if advanced {
            body = body
                .push(container(text(crate::l10n::text("游戏运行环境")).size(16)).padding([8, 0]));
        }
        let enabled = !self.locked() && !self.wine.loading;
        if self.wine.loading {
            body = body.push(text(crate::l10n::text("正在检查运行环境…")).size(14));
        }
        if let Some(status) = &self.wine.status {
            if !status.rosetta || cfg!(all(target_os = "macos", target_arch = "aarch64")) {
                body = body.push(self.preference_row(
                    "Rosetta 2",
                    "让 Apple 芯片运行 Wine 所需的 Intel 程序，安装时会出现系统授权窗口。",
                    if status.rosetta {
                        text(crate::l10n::text("已就绪")).size(14).into()
                    } else {
                        action("安装 Rosetta 2", message(Action::Rosetta), enabled)
                    },
                ));
            }
            let source = if status.custom_path.is_some() {
                "自选 Wine"
            } else {
                "内置 Wine"
            };
            let version = status.version.as_ref().or(status
                .bundled_version
                .as_ref()
                .filter(|_| status.custom_path.is_none()));
            let summary = format!(
                "{} {}",
                crate::l10n::text(source),
                version.map(String::as_str).unwrap_or("")
            );
            body = body.push(self.preference_row(
                "Wine",
                "用于运行 Windows 版 Ballance。",
                text(summary).size(14).into(),
            ));
            if ready(status) {
                body = body.push(self.preference_row(
                    "游戏运行环境",
                    "",
                    text(crate::l10n::text("已就绪")).size(14).into(),
                ));
            } else {
                let available = status.installed || status.bundled_version.is_some();
                let label = if status.custom_path.is_some() && !status.installed {
                    "自选 Wine 无法运行"
                } else if !status.rosetta {
                    "先安装 Rosetta 2"
                } else if available {
                    "初始化游戏环境"
                } else {
                    "请选择已有 Wine"
                };
                body = body.push(self.preference_row(
                    "游戏运行环境",
                    "使用随软件附带的 Wine 或已选择的 Wine 准备游戏环境，初始化可能需要几十秒。",
                    action(
                        label,
                        message(Action::Prepare),
                        enabled
                            && status.rosetta
                            && available
                            && !(status.custom_path.is_some() && !status.installed),
                    ),
                ));
            }
            body = body.push(
                self.preference_row(
                    "Wine 路径",
                    status
                        .path
                        .as_deref()
                        .unwrap_or("可选择已安装的 Wine 应用或包含 bin/wine 的目录。"),
                    action("选择已有 Wine", message(Action::Choose), enabled),
                ),
            );
            if status.custom_path.is_some() {
                body = body.push(self.preference_row(
                    "内置 Wine",
                    "",
                    action(
                        "使用内置 Wine",
                        message(Action::UseBundled),
                        enabled && status.bundled_version.is_some(),
                    ),
                ));
            }
            if advanced && ready(status) {
                body = body.push(self.preference_row(
                    "高级设置",
                    "打开 Wine 自带的声音和显示设置。",
                    action("打开 Wine 设置", message(Action::Configure), enabled),
                ));
            }
        }
        if let Some(error) = &self.wine.error {
            body = body.push(container(text(error).size(14)).padding([8, 0]));
        }
        if self.wine.working {
            body = body.push(
                container(text(crate::l10n::text("正在处理，请完成可能出现的系统窗口…")).size(14))
                    .padding([8, 0]),
            );
        }
        body = body.push(self.preference_row(
            "环境检测",
            "",
            action("重新检测", message(Action::Refresh), enabled),
        ));
        body.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn readiness_requires_rosetta_runtime_and_initialized_prefix() {
        let mut status = WineStatus {
            rosetta: true,
            installed: true,
            prefix_ready: true,
            ..Default::default()
        };
        assert!(ready(&status));
        status.rosetta = false;
        assert!(!ready(&status));
        status.rosetta = true;
        status.installed = false;
        assert!(!ready(&status));
        status.installed = true;
        status.prefix_ready = false;
        assert!(!ready(&status));
    }
}
