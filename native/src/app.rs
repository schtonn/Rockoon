use crate::{
    game::{self, Instance, Resource, RunningGame},
    preferences::{self, Preferences},
    style,
};
use iced::{
    Alignment, Element, Fill, Subscription, Task,
    widget::{
        button, column, container, image, row, rule, scrollable, space, stack, text, text_input,
        tooltip,
    },
};
use std::{path::PathBuf, time::Duration};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Home,
    Maps,
    Mods,
}

#[derive(Clone, Debug)]
pub enum Message {
    Startup(Result<Option<Instance>, String>),
    PickFolder,
    FolderPicked(Option<PathBuf>),
    Loaded(Result<(Instance, Option<String>), String>),
    Page(Page),
    Search(String),
    Refresh,
    Launch(Option<PathBuf>),
    Started(Result<RunningGame, String>),
    ToggleMod(Resource),
    ModChanged(Result<Instance, String>),
    OpenFolder(PathBuf),
    FolderOpened(Result<(), String>),
    Tick,
    Dismiss,
}

pub struct App {
    page: Page,
    instance: Option<Instance>,
    busy: bool,
    picking_folder: bool,
    launching: bool,
    running: Option<RunningGame>,
    search: String,
    notice: Option<String>,
}

async fn background<T: Send + 'static>(
    work: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tokio::task::spawn_blocking(work)
        .await
        .map_err(|error| error.to_string())?
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let state = Self {
            page: Page::Home,
            instance: None,
            busy: true,
            picking_folder: false,
            launching: false,
            running: None,
            search: String::new(),
            notice: None,
        };
        let requested = std::env::args_os().nth(1).map(PathBuf::from);
        let startup = Task::perform(
            background(move || {
                let preferences = preferences::load()?;
                let instance = requested
                    .or_else(|| preferences.instance_path.clone())
                    .map(game::discover)
                    .transpose()?;
                Ok(instance)
            }),
            Message::Startup,
        );
        (state, startup)
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Startup(result) => {
                self.busy = false;
                match result {
                    Ok(instance) => {
                        self.instance = instance;
                    }
                    Err(error) => self.notice = Some(error),
                }
            }
            Message::PickFolder if !self.busy && !self.picking_folder && !self.launching => {
                self.picking_folder = true;
                let initial = self.instance.as_ref().map(|game| game.root.clone());
                return Task::perform(
                    async move {
                        let mut picker =
                            rfd::AsyncFileDialog::new().set_title("选择 Ballance 游戏根目录");
                        if let Some(path) = initial {
                            picker = picker.set_directory(path);
                        }
                        picker
                            .pick_folder()
                            .await
                            .map(|folder| folder.path().to_owned())
                    },
                    Message::FolderPicked,
                );
            }
            Message::FolderPicked(path) => {
                self.picking_folder = false;
                if let Some(path) = path {
                    self.busy = true;
                    self.notice = None;
                    return Task::perform(
                        background(move || {
                            let instance = game::discover(path)?;
                            let preferences = Preferences {
                                instance_path: Some(instance.root.clone()),
                            };
                            let warning = preferences::save(&preferences)
                                .err()
                                .map(|error| format!("目录已打开，但无法保存偏好：{error}"));
                            Ok((instance, warning))
                        }),
                        Message::Loaded,
                    );
                }
            }
            Message::Loaded(result) => {
                self.busy = false;
                match result {
                    Ok((instance, warning)) => {
                        self.instance = Some(instance);
                        self.notice = warning;
                    }
                    Err(error) => self.notice = Some(error),
                }
            }
            Message::Page(page) => {
                self.page = page;
                self.search.clear();
            }
            Message::Search(search) => self.search = search,
            Message::Refresh if !self.busy && !self.launching => {
                if let Some(instance) = &self.instance {
                    let root = instance.root.clone();
                    self.busy = true;
                    return Task::perform(
                        background(move || game::discover(root).map(|instance| (instance, None))),
                        Message::Loaded,
                    );
                }
            }
            Message::Launch(map) if !self.busy && !self.launching && self.running.is_none() => {
                if let Some(instance) = self.instance.clone() {
                    self.launching = true;
                    self.notice = None;
                    return Task::perform(
                        background(move || game::launch(&instance, map.as_deref())),
                        Message::Started,
                    );
                }
            }
            Message::Started(result) => {
                self.launching = false;
                match result {
                    Ok(process) => self.running = Some(process),
                    Err(error) => self.notice = Some(error),
                }
            }
            Message::ToggleMod(resource)
                if !self.busy && !self.launching && self.running.is_none() =>
            {
                if let Some(instance) = self.instance.clone() {
                    self.busy = true;
                    self.notice = None;
                    return Task::perform(
                        background(move || {
                            game::toggle_mod(&instance, &resource)?;
                            game::discover(instance.root)
                        }),
                        Message::ModChanged,
                    );
                }
            }
            Message::ModChanged(result) => {
                self.busy = false;
                match result {
                    Ok(instance) => self.instance = Some(instance),
                    Err(error) => self.notice = Some(error),
                }
            }
            Message::OpenFolder(path) => {
                return Task::perform(
                    background(move || game::open_folder(&path)),
                    Message::FolderOpened,
                );
            }
            Message::FolderOpened(Err(error)) => self.notice = Some(error),
            Message::Tick => {
                let status = self.running.as_ref().map(|process| {
                    process
                        .lock()
                        .map_err(|error| error.to_string())?
                        .try_wait()
                        .map_err(|error| error.to_string())
                });
                match status {
                    Some(Ok(Some(exit))) => {
                        self.running = None;
                        if !exit.success() {
                            self.notice = Some(format!("游戏已退出：{exit}"));
                        }
                    }
                    Some(Err(error)) => {
                        self.notice = Some(format!("无法查询游戏状态：{error}"));
                        self.running = None;
                    }
                    _ => {}
                }
            }
            Message::Dismiss => self.notice = None,
            _ => {}
        }
        Task::none()
    }

    pub fn subscription(&self) -> Subscription<Message> {
        if self.running.is_some() {
            iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let sidebar = container(
            column![
                column![
                    text("ROCKOON").size(19).color(style::INK),
                    text("Ballance launcher").size(11).color(style::MUTED),
                ]
                .spacing(4),
                space::vertical().height(32),
                self.nav("开始", Page::Home),
                self.nav("地图", Page::Maps),
                self.nav("模组", Page::Mods),
                space::vertical(),
                text(if self.running.is_some() {
                    "●  游戏运行中"
                } else {
                    "○  就绪"
                })
                .size(12)
                .color(style::MUTED),
            ]
            .spacing(7)
            .height(Fill),
        )
        .padding([30, 18])
        .width(164)
        .height(Fill)
        .style(style::sidebar);

        let mut content = column![self.header()].spacing(22).width(Fill).height(Fill);
        if let Some(notice) = &self.notice {
            content = content.push(
                container(
                    row![
                        text(notice).size(13).width(Fill),
                        button(text("关闭").size(12))
                            .style(style::quiet)
                            .on_press(Message::Dismiss),
                    ]
                    .spacing(12)
                    .align_y(Alignment::Center),
                )
                .padding(12)
                .style(style::notice),
            );
        }
        content = content.push(match self.page {
            Page::Home => self.home(),
            Page::Maps => self.resources(false),
            Page::Mods => self.resources(true),
        });
        row![
            sidebar,
            container(content).padding(32).width(Fill).height(Fill)
        ]
        .into()
    }

    fn nav<'a>(&self, label: &'a str, page: Page) -> Element<'a, Message> {
        button(text(label).size(14))
            .on_press(Message::Page(page))
            .width(Fill)
            .padding([11, 14])
            .style(style::nav(self.page == page))
            .into()
    }

    fn header(&self) -> Element<'_, Message> {
        let title = match self.page {
            Page::Home => "开始",
            Page::Maps => "地图",
            Page::Mods => "模组",
        };
        let mut actions = row![].spacing(8).align_y(Alignment::Center);
        if let Some(instance) = &self.instance {
            actions = actions.push(tooltip(
                text(&instance.name).size(13).color(style::MUTED),
                container(text(instance.root.to_string_lossy().into_owned()).size(12)).padding(8),
                tooltip::Position::Bottom,
            ));
            actions = actions.push(
                button(text("打开目录").size(12))
                    .style(style::quiet)
                    .on_press(Message::OpenFolder(instance.root.clone())),
            );
        }
        actions = actions.push(
            button(
                text(if self.picking_folder {
                    "选择中…"
                } else {
                    "更换目录"
                })
                .size(12),
            )
            .style(style::quiet)
            .on_press_maybe(
                (!self.busy && !self.picking_folder && !self.launching)
                    .then_some(Message::PickFolder),
            ),
        );
        row![text(title).size(22), space::horizontal(), actions]
            .align_y(Alignment::Center)
            .into()
    }

    fn home(&self) -> Element<'_, Message> {
        let subtitle = if self.busy {
            "正在读取游戏目录…".to_owned()
        } else if let Some(instance) = &self.instance {
            format!("{}  ·  {}", instance.player, instance.loader)
        } else {
            "选择已安装的 Ballance 游戏目录".to_owned()
        };
        let enabled = !self.busy && !self.launching && self.running.is_none();
        let label = if self.launching {
            "正在启动…"
        } else if self.running.is_some() {
            "游戏运行中"
        } else {
            "启动游戏"
        };
        let action: Element<'_, Message> = if let Some(atlas) = self
            .instance
            .as_ref()
            .and_then(|instance| instance.visuals.launch_button())
        {
            let skin = image(atlas.clone())
                .width(256)
                .height(64)
                .opacity(if enabled { 1.0_f32 } else { 0.5_f32 });
            let caption = container(text(label).size(16).color(iced::Color::WHITE))
                .padding(iced::Padding {
                    left: 42.0,
                    ..iced::Padding::ZERO
                })
                .center_x(Fill)
                .center_y(Fill);
            button(stack![skin, caption].width(256).height(64).clip(true))
                .padding(0)
                .style(style::original_button)
                .on_press_maybe(enabled.then_some(Message::Launch(None)))
                .into()
        } else if self.instance.is_some() {
            button(text(label).size(15))
                .padding([14, 28])
                .style(style::primary)
                .on_press_maybe(enabled.then_some(Message::Launch(None)))
                .into()
        } else {
            button(text("选择游戏目录").size(15))
                .padding([14, 28])
                .style(style::primary)
                .on_press_maybe((!self.busy && !self.picking_folder).then_some(Message::PickFolder))
                .into()
        };

        let sky = self
            .instance
            .as_ref()
            .and_then(|instance| instance.visuals.sky.as_ref());
        let foreground = container(
            column![
                column![
                    text("Ballance").size(44).color(style::INK),
                    text(subtitle).size(13).color(if sky.is_some() {
                        iced::Color::WHITE
                    } else {
                        style::MUTED
                    }),
                ]
                .spacing(8),
                space::vertical(),
                container(action)
                    .align_x(iced::alignment::Horizontal::Right)
                    .width(Fill),
            ]
            .height(Fill),
        )
        .padding(28)
        .width(Fill)
        .height(Fill);

        if let Some(sky) = sky {
            // Read the level-1 skybox bitmap directly from this installation.
            // It is displayed as-is; no painted replacement scene or generated balls.
            stack![
                image(sky.clone())
                    .width(Fill)
                    .height(Fill)
                    .content_fit(iced::ContentFit::Fill),
                foreground,
            ]
            .width(Fill)
            .height(Fill)
            .clip(true)
            .into()
        } else {
            foreground.into()
        }
    }

    fn resources(&self, mods: bool) -> Element<'_, Message> {
        let Some(instance) = &self.instance else {
            return container(
                column![
                    text("先选择游戏目录").size(20),
                    text("在这里浏览你已安装的地图和模组。")
                        .size(13)
                        .color(style::MUTED),
                    button("选择目录").style(style::primary).on_press_maybe(
                        (!self.busy && !self.picking_folder).then_some(Message::PickFolder)
                    ),
                ]
                .spacing(14),
            )
            .center_x(Fill)
            .center_y(Fill)
            .into();
        };
        let resources = if mods { &instance.mods } else { &instance.maps };
        let query = self.search.to_lowercase();
        let matched: Vec<_> = resources
            .iter()
            .filter(|resource| resource.name.to_lowercase().contains(&query))
            .collect();
        let toolbar = row![
            text_input("搜索名称…", &self.search)
                .on_input(Message::Search)
                .padding(10)
                .width(Fill),
            button("刷新")
                .style(style::quiet)
                .padding([10, 16])
                .on_press_maybe((!self.busy && !self.launching).then_some(Message::Refresh)),
        ]
        .spacing(12)
        .align_y(Alignment::Center);
        let mut list = column![].spacing(0);
        for resource in &matched {
            let action = if mods {
                button(
                    text(if resource.disabled {
                        "启用"
                    } else {
                        "停用"
                    })
                    .size(12),
                )
                .style(style::quiet)
                .on_press_maybe(
                    (!self.busy && !self.launching && self.running.is_none())
                        .then(|| Message::ToggleMod((*resource).clone())),
                )
            } else {
                button(text("游玩  →").size(12))
                    .style(style::quiet)
                    .on_press_maybe(
                        (!self.busy && !self.launching && self.running.is_none())
                            .then(|| Message::Launch(Some(resource.path.clone()))),
                    )
            };
            let details = if mods {
                if resource.disabled {
                    "已停用"
                } else {
                    "已启用"
                }
            } else {
                resource
                    .path
                    .extension()
                    .and_then(|extension| extension.to_str())
                    .unwrap_or("")
            };
            list = list.push(
                container(
                    row![
                        text(&resource.name).size(14).width(Fill),
                        text(details).size(11).color(style::MUTED),
                        action,
                    ]
                    .spacing(20)
                    .align_y(Alignment::Center),
                )
                .padding([13, 0]),
            );
            list = list.push(rule::horizontal(1));
        }
        let body: Element<'_, Message> = if matched.is_empty() {
            container(
                text(if query.is_empty() {
                    "还没有本地资源，可将文件放入游戏的 ModLoader 目录后刷新。"
                } else {
                    "没有匹配的资源。"
                })
                .size(13)
                .color(style::MUTED),
            )
            .center_x(Fill)
            .center_y(Fill)
            .into()
        } else {
            scrollable(list).height(Fill).into()
        };
        column![
            toolbar,
            text(format!("{} 项", matched.len()))
                .size(12)
                .color(style::MUTED),
            body,
        ]
        .spacing(18)
        .height(Fill)
        .into()
    }
}
