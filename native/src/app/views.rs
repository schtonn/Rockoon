use super::*;
use iced::{
    Alignment, Fill,
    widget::{
        button, column, container, image, opaque, progress_bar, row, rule, space, stack, text,
    },
};
fn text_input<'a>(placeholder: &str, value: &str) -> iced::widget::TextInput<'a, Message> {
    iced::widget::text_input(placeholder, value)
        .size(14)
        .padding(9)
        .style(style::input)
}
fn checkbox<'a>(checked: bool) -> iced::widget::Checkbox<'a, Message> {
    iced::widget::checkbox(checked)
        .size(18)
        .spacing(9)
        .text_size(14)
        .style(style::check)
}
fn pick_list<'a, T, L, V>(
    options: L,
    selected: Option<V>,
    on_selected: impl Fn(T) -> Message + 'a,
) -> iced::widget::PickList<'a, T, L, V, Message>
where
    T: ToString + PartialEq + Clone + 'a,
    L: std::borrow::Borrow<[T]> + 'a,
    V: std::borrow::Borrow<T> + 'a,
{
    iced::widget::pick_list(options, selected, on_selected)
        .text_size(14)
        .padding(9)
        .style(style::select)
        .menu_style(style::select_menu)
}
fn scrollable<'a>(
    content: impl Into<Element<'a, Message>>,
) -> iced::widget::Scrollable<'a, Message> {
    iced::widget::scrollable(content).style(style::scroll)
}
fn action<'a>(label: &str, message: Message, enabled: bool) -> Element<'a, Message> {
    button(text(crate::l10n::text(label)).size(14))
        .padding([8, 12])
        .style(style::quiet)
        .on_press_maybe(enabled.then_some(message))
        .into()
}
fn menu_action<'a>(label: &str, message: Message, enabled: bool) -> Element<'a, Message> {
    button(text(crate::l10n::text(label)).size(14))
        .padding([8, 10])
        .width(Fill)
        .style(style::quiet)
        .on_press_maybe(enabled.then_some(message))
        .into()
}
fn menu_panel<'a>(
    content: impl Into<Element<'a, Message>>,
) -> iced::widget::Container<'a, Message> {
    container(content).padding(6).style(style::menu)
}
impl App {
    pub fn view(&self) -> Element<'_, Message> {
        let colors = style::colors(&self.theme());
        let mut nav = column![].spacing(6);
        for page in [Page::Home, Page::Maps, Page::Config, Page::Tasks] {
            nav = nav.push(self.nav(page));
        }
        nav = nav.push(space::vertical()).push(self.nav(Page::Settings));
        let sidebar = container(nav.height(Fill))
            .padding([24, 18])
            .width(164)
            .height(Fill)
            .clip(true)
            .style(style::sidebar);
        let header = self.header();
        let content: Element<'_, Message> = if self.page == Page::Home {
            stack![self.home(), container(header).padding(24).width(Fill)]
                .width(Fill)
                .height(Fill)
                .clip(true)
                .into()
        } else {
            let body = match self.page {
                Page::Maps | Page::Mods | Page::Assets => self.resources(),
                Page::Config | Page::Data => self.configuration(),
                Page::Profiles => self.profiles(),
                Page::Install => self.install(),
                Page::Tasks => self.tasks(),
                Page::Tas => self.timeline(),
                Page::Settings => self.settings(),
                Page::Home => unreachable!(),
            };
            let tabs = self.section_tabs();
            let body = if self.detached_tas
                || !matches!(self.page.section(), Page::Maps | Page::Config | Page::Tasks)
            {
                body
            } else {
                column![tabs, body].spacing(16).height(Fill).into()
            };
            container(column![header, body].spacing(20).height(Fill))
                .padding(24)
                .width(Fill)
                .height(Fill)
                .into()
        };
        let base: Element<'_, Message> = if self.detached_tas {
            content
        } else {
            row![sidebar, content].height(Fill).into()
        };
        let base = if let Some(notice) = &self.notice {
            let alpha = notice.opacity();
            let toast = opaque(
                container(
                    row![
                        text(&notice.text)
                            .size(14)
                            .color(colors.text.scale_alpha(alpha))
                            .width(Fill),
                        button(text("×").size(18).color(colors.text.scale_alpha(alpha)))
                            .padding([2, 7])
                            .style(style::quiet)
                            .on_press(Message::Dismiss)
                    ]
                    .spacing(12)
                    .align_y(Alignment::Center),
                )
                .padding(14)
                .width(440)
                .style(move |theme| style::toast(theme, alpha)),
            );
            stack![
                base,
                container(column![space::vertical(), row![space::horizontal(), toast]])
                    .padding(24)
                    .width(Fill)
                    .height(Fill)
            ]
            .into()
        } else {
            base
        };
        if let Some(prompt) = &self.prompt {
            let mut body = column![text(&prompt.title).size(19)].spacing(18).width(420);
            if matches!(
                prompt.kind,
                PromptKind::NewFolder
                    | PromptKind::Rename(_)
                    | PromptKind::CreateProfile
                    | PromptKind::RenameProfile(_)
            ) {
                body = body.push(
                    text_input("名称", &prompt.input)
                        .padding(10)
                        .on_input(Message::PromptText)
                        .on_submit(Message::Confirm),
                );
            } else if !prompt.input.is_empty() {
                body = body.push(text(&prompt.input).size(14));
            }
            body = body.push(
                row![
                    space::horizontal(),
                    action("取消", Message::CancelPrompt, true),
                    action("确定", Message::Confirm, !self.locked())
                ]
                .spacing(12),
            );
            return stack![
                base,
                opaque(
                    container(container(body).padding(24).style(style::sidebar))
                        .center_x(Fill)
                        .center_y(Fill)
                        .style(style::shade)
                )
            ]
            .into();
        }
        if let Some((map, diff)) = &self.launch_warning {
            let mut files = column![].spacing(7);
            for file in diff {
                files = files.push(text(file).size(14));
            }
            let mut dialog = column![
                text(crate::l10n::text("启动提醒")).size(19),
                scrollable(container(files).padding(iced::Padding {
                    right: 16.0,
                    ..iced::Padding::ZERO
                }))
                .height(240),
                row![
                    action("取消", Message::Dismiss, true),
                    action(
                        "恢复原版",
                        Message::Prompt(PromptKind::Restore),
                        !self.locked()
                    ),
                    action(
                        "仍然启动",
                        Message::LaunchAnyway(map.clone()),
                        !self.locked()
                    ),
                ]
                .spacing(8),
            ]
            .spacing(18)
            .width(520);
            if diff.iter().any(|s| s.starts_with("TAS 自动播放")) {
                dialog = dialog.push(action(
                    "关闭自动播放后启动",
                    Message::DisableAutoplay(map.clone()),
                    !self.locked(),
                ));
            }
            return stack![
                base,
                opaque(
                    container(container(dialog).padding(24).style(style::sidebar))
                        .center_x(Fill)
                        .center_y(Fill)
                        .style(style::shade)
                )
            ]
            .into();
        }
        if let Some(preview) = &self.preview {
            let mut panel = column![
                row![
                    text(
                        self.preview_path
                            .as_ref()
                            .and_then(|p| p.file_name())
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string()
                    )
                    .size(16)
                    .width(Fill),
                    action("关闭", Message::ClosePreview, true)
                ],
                image(preview.clone())
                    .width(Fill)
                    .height(Fill)
                    .content_fit(iced::ContentFit::Contain)
            ]
            .spacing(12);
            if self.skybox.is_some() {
                panel = panel.push(
                    row![
                        text(crate::l10n::text("方向")).size(14),
                        iced::widget::slider(-180.0..=180.0, self.sky_yaw, |v| Message::SkyAngle(
                            true, v
                        ))
                        .style(style::range)
                        .on_release(Message::SkyRender),
                        text(crate::l10n::text("俯仰")).size(14),
                        iced::widget::slider(-80.0..=80.0, self.sky_pitch, |v| Message::SkyAngle(
                            false, v
                        ))
                        .style(style::range)
                        .on_release(Message::SkyRender),
                    ]
                    .spacing(14)
                    .align_y(Alignment::Center),
                );
            }
            return stack![
                base,
                opaque(
                    container(panel)
                        .padding(24)
                        .width(Fill)
                        .height(Fill)
                        .style(style::sidebar)
                )
            ]
            .into();
        }
        base
    }
    fn nav(&self, page: Page) -> Element<'_, Message> {
        button(
            text(crate::l10n::text(page.section_label()))
                .size(15)
                .font(iced::Font {
                    weight: iced::font::Weight::Medium,
                    ..iced::Font::with_name("Source Han Sans CN")
                }),
        )
        .on_press(Message::Page(if self.page.section() == page.section() {
            self.page
        } else {
            page
        }))
        .width(Fill)
        .padding([10, 14])
        .style(style::nav(self.page.section() == page.section()))
        .into()
    }
    fn section_tabs(&self) -> Element<'_, Message> {
        let pages: &[Page] = match self.page.section() {
            Page::Maps => &[Page::Maps, Page::Mods, Page::Assets],
            Page::Config => &[Page::Config, Page::Data, Page::Profiles],
            Page::Tasks => &[Page::Tasks, Page::Install],
            _ => &[],
        };
        if pages.is_empty() {
            return space().height(0).into();
        }
        let mut tabs = row![].spacing(6);
        for &page in pages {
            let label = if page == Page::Config {
                "参数"
            } else {
                page.label()
            };
            tabs = tabs.push(
                button(text(crate::l10n::text(label)).size(14))
                    .padding([7, 12])
                    .style(style::tab(self.page == page))
                    .on_press(Message::Page(page)),
            );
        }
        if self.page.section() == Page::Maps {
            tabs = tabs.push(space::horizontal()).push(action(
                if self.opening_hub {
                    "正在连接…"
                } else {
                    "社区资源站 ↗"
                },
                Message::OpenHub,
                !self.opening_hub,
            ));
        }
        tabs.width(Fill).into()
    }
    fn header(&self) -> Element<'_, Message> {
        if self.instance.is_none() {
            return if self.page == Page::Home {
                space().into()
            } else {
                text(crate::l10n::text(self.page.section_label()))
                    .size(22)
                    .into()
            };
        }
        let full = self
            .instance
            .as_ref()
            .map(|i| i.root.display().to_string())
            .unwrap_or_else(|| "选择游戏目录".into());
        let path = if full.chars().count() > 56 {
            let first: String = full.chars().take(14).collect();
            let last: String = full
                .chars()
                .rev()
                .take(37)
                .collect::<String>()
                .chars()
                .rev()
                .collect();
            format!("{first}…{last}")
        } else {
            full
        };
        let trigger = button(
            iced::widget::canvas(FolderIcon {
                on_sky: self.page == Page::Home,
            })
            .width(22)
            .height(20),
        )
        .padding([7, 9])
        .style(if self.page == Page::Home {
            style::sky_button
        } else {
            style::quiet
        })
        .on_press(Message::FolderMenu);
        let panel = if self.folder_menu {
            let mut menu = column![].spacing(3).width(156);
            if let Some(root) = self.root() {
                menu = menu.push(menu_action("打开目录", Message::Open(root), true));
            }
            menu = menu.push(menu_action(
                "更换目录",
                Message::PickFolder,
                !self.locked() && !self.picking_folder,
            ));
            Some(menu_panel(menu).into())
        } else {
            None
        };
        let folder = crate::popup::menu(trigger, panel, false);
        let actions = row![
            folder,
            text(path).size(14).color(if self.page == Page::Home {
                iced::Color::WHITE
            } else {
                style::colors(&self.theme()).text
            })
        ]
        .spacing(8)
        .align_y(Alignment::Center);
        let mut header = row![].align_y(Alignment::Center);
        if self.page != Page::Home {
            header = header.push(text(crate::l10n::text(self.page.section_label())).size(22));
        }
        header.push(space::horizontal()).push(actions).into()
    }
    fn home(&self) -> Element<'_, Message> {
        if self.instance.is_none() {
            return self.setup();
        }
        let enabled =
            !self.busy && !self.launching && !self.jobs.iter().any(|j| !j.progress().done);
        let (label, message) = if self.launching {
            ("Starting...", Message::Launch(None))
        } else if self.running.is_some() {
            ("Stop", Message::Stop)
        } else if self.instance.is_some() {
            ("Start", Message::Launch(None))
        } else {
            ("Browse", Message::PickFolder)
        };
        let action: Element<'_, Message> = if let Some(visuals) = self
            .instance
            .as_ref()
            .map(|i| &i.visuals)
            .filter(|v| v.launch_button(false).is_some())
        {
            let caption = crate::launch::caption(visuals, label, enabled);
            crate::launch::button(
                visuals.launch_button(false).unwrap().clone(),
                visuals.launch_button(true).unwrap().clone(),
                caption,
                enabled,
                message,
            )
        } else {
            button(text(label.to_uppercase()).size(13))
                .padding([14, 28])
                .style(style::primary)
                .on_press_maybe(enabled.then_some(message))
                .into()
        };
        let foreground =
            container(column![space::vertical(), container(action).align_right(Fill)].height(Fill))
                .padding(28)
                .width(Fill)
                .height(Fill);
        if let Some(sky) = self.instance.as_ref().and_then(|i| i.visuals.sky.as_ref()) {
            stack![
                image(sky.clone())
                    .width(Fill)
                    .height(Fill)
                    .content_fit(iced::ContentFit::Fill),
                foreground
            ]
            .width(Fill)
            .height(Fill)
            .clip(true)
            .into()
        } else {
            foreground.into()
        }
    }
    fn setup(&self) -> Element<'_, Message> {
        if self.busy {
            return container(text(crate::l10n::text("读取中…")).size(14))
                .center_x(Fill)
                .center_y(Fill)
                .into();
        }
        let mut content = column![
            text(crate::l10n::text("选择 Ballance")).size(24),
            text(crate::l10n::text("选择已有的游戏文件夹，或安装一份游戏。")).size(14),
        ]
        .spacing(18);
        if let Some(problem) = &self.setup_problem {
            content = content.push(text(problem).size(14));
        }
        content = content.push(
            row![
                button(text(crate::l10n::text("选择已有游戏")).size(14))
                    .padding([10, 14])
                    .style(style::primary)
                    .on_press_maybe(
                        (!self.locked() && !self.picking_folder).then_some(Message::PickFolder)
                    ),
                action("下载并安装", Message::Page(Page::Install), !self.locked()),
            ]
            .spacing(12),
        );
        container(container(content).max_width(480))
            .padding(28)
            .center_x(Fill)
            .center_y(Fill)
            .into()
    }
    fn empty(&self) -> Element<'_, Message> {
        self.setup()
    }
    fn resources(&self) -> Element<'_, Message> {
        let Some(root) = self.root() else {
            return self.empty();
        };
        let Some(kind) = self.kind() else {
            return self.empty();
        };
        let mut toolbar = row![].spacing(8).align_y(Alignment::Center);
        if self.page == Page::Assets {
            toolbar = toolbar.push(
                pick_list(
                    vec![Kind::Skies, Kind::Textures, Kind::Sounds],
                    Some(kind),
                    Message::AssetKind,
                )
                .text_size(14),
            );
        }
        toolbar = toolbar
            .push(
                text_input("搜索", &self.search)
                    .on_input(Message::Search)
                    .padding(9)
                    .width(Fill),
            )
            .push(action("导入", Message::Import, !self.locked()))
            .push(action(
                "新建文件夹",
                Message::Prompt(PromptKind::NewFolder),
                !self.locked(),
            ))
            .push(action("刷新", Message::Refresh, !self.busy));
        let mut crumbs = row![].spacing(4);
        crumbs = crumbs.push(action(
            "根目录",
            Message::Browse(root.join(kind.dir())),
            !self.busy,
        ));
        let base = root.join(kind.dir());
        let mut path = base.clone();
        if let Ok(relative) = self.directory.strip_prefix(&base) {
            for part in relative.components() {
                path.push(part);
                crumbs = crumbs.push(text(crate::l10n::text("/")).size(14)).push(
                    button(text(part.as_os_str().to_string_lossy().to_string()).size(14))
                        .style(style::quiet)
                        .on_press(Message::Browse(path.clone())),
                );
            }
        }
        crumbs = crumbs.push(space::horizontal()).push(action(
            "打开文件夹",
            Message::Open(self.directory.clone()),
            true,
        ));
        if self.deleted.is_some() {
            crumbs = crumbs.push(action("撤销删除", Message::UndoDelete, !self.locked()));
        }
        if self.sound.is_some() {
            crumbs = crumbs.push(action("停止试听", Message::StopSound, true));
        }
        let mut list = column![].spacing(0);
        for entry in self
            .entries
            .iter()
            .filter(|e| workspace::matches(&e.name, &self.search))
        {
            let primary = if entry.directory {
                action("打开", Message::Browse(entry.path.clone()), !self.busy)
            } else {
                match kind {
                    Kind::Maps => action(
                        "游玩",
                        Message::Launch(Some(entry.path.clone())),
                        !self.locked() && !entry.disabled,
                    ),
                    Kind::Mods => action(
                        if entry.disabled { "启用" } else { "停用" },
                        Message::Toggle(entry.clone()),
                        !self.locked(),
                    ),
                    Kind::Skies | Kind::Textures => {
                        action("预览", Message::Preview(entry.path.clone()), true)
                    }
                    Kind::Sounds => action(
                        if self.sound.as_ref() == Some(&entry.path) {
                            "停止"
                        } else {
                            "试听"
                        },
                        Message::Sound(entry.path.clone()),
                        true,
                    ),
                }
            };
            let panel = if self.entry_menu.as_ref() == Some(&entry.path) {
                let mut menu = column![].spacing(3).width(156);
                if kind == Kind::Mods && !entry.directory {
                    menu = menu.push(menu_action(
                        "配置",
                        Message::FindConfig(entry.path.clone()),
                        true,
                    ));
                }
                menu = menu
                    .push(menu_action(
                        "重命名",
                        Message::Prompt(PromptKind::Rename(entry.path.clone())),
                        !self.locked(),
                    ))
                    .push(menu_action(
                        "移动",
                        Message::Move(entry.path.clone()),
                        !self.locked(),
                    ))
                    .push(menu_action(
                        "删除",
                        Message::Prompt(PromptKind::Delete(entry.path.clone())),
                        !self.locked(),
                    ));
                Some(menu_panel(menu).into())
            } else {
                None
            };
            let more = crate::popup::menu(
                button(iced::widget::canvas(MoreIcon).width(18).height(14))
                    .padding([7, 7])
                    .style(style::quiet)
                    .on_press(Message::EntryMenu(entry.path.clone())),
                panel,
                true,
            );
            let line = row![text(&entry.name).size(14).width(Fill), primary, more]
                .spacing(8)
                .align_y(Alignment::Center);
            list = list.push(container(line).padding([8, 0]));
            list = list.push(rule::horizontal(1));
        }
        let visible = self
            .entries
            .iter()
            .filter(|entry| workspace::matches(&entry.name, &self.search))
            .count();
        if visible == 0 {
            let mut empty = column![
                text(crate::l10n::text(if self.search.trim().is_empty() {
                    "这里还没有资源"
                } else {
                    "没有匹配的资源"
                }))
                .size(16),
            ]
            .spacing(12);
            if self.search.trim().is_empty() {
                empty = empty.push(
                    text(crate::l10n::text(if self.protocol_ready {
                        "在社区资源站点击“一键安装”，也可以导入本地文件。"
                    } else {
                        "可以导入本地文件，或打开社区资源站下载。"
                    }))
                    .size(14),
                );
                if !self.protocol_ready {
                    empty = empty.push(action("启用一键安装", Message::RegisterProtocol, true));
                }
            }
            list = list.push(container(empty).padding([24, 0]));
        }
        let mut body = column![toolbar, crumbs].spacing(12);
        if matches!(kind, Kind::Maps | Kind::Mods)
            && !root.join("BuildingBlocks/BML.dll").is_file()
            && !root.join("BuildingBlocks/BMLPlus.dll").is_file()
        {
            body = body.push(
                row![
                    text(crate::l10n::text("使用自制地图和模组需要 BML / BML+。"))
                        .size(14)
                        .width(Fill),
                    action("安装支持", Message::Page(Page::Install), !self.locked()),
                ]
                .align_y(Alignment::Center),
            );
        }
        if kind == Kind::Skies {
            body = body.push(
                row![
                    text(crate::l10n::text("导入到")).size(14),
                    self.sky_picker()
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            );
        }
        body.push(
            scrollable(container(list).padding(iced::Padding {
                right: 16.0,
                ..iced::Padding::ZERO
            }))
            .height(Fill),
        )
        .height(Fill)
        .into()
    }
    fn sky_picker(&self) -> Element<'_, Message> {
        let mut options = vec!["自制地图（X）".to_owned()];
        for i in 1..=13 {
            options.push(format!("第 {i} 关"));
        }
        pick_list(options, Some(self.sky.clone()), Message::Sky)
            .text_size(14)
            .into()
    }
    fn configuration_field<'a>(
        &'a self,
        i: usize,
        field: &'a config::Field,
    ) -> Element<'a, Message> {
        if field.key == "/volume" {
            let value = field.value.parse::<f32>().unwrap_or(1.0).clamp(0.0, 1.0);
            let control: Element<'_, Message> = if self.locked() {
                space::horizontal().width(180).into()
            } else {
                iced::widget::slider(0.0..=1.0, value, move |v| {
                    Message::Field(i, format!("{v:.2}"))
                })
                .step(0.01_f32)
                .width(180)
                .style(style::range)
                .into()
            };
            row![
                text(crate::l10n::text("音量")).size(14).width(Fill),
                control,
                text(format!("{:.0}%", value * 100.0)).size(14).width(44),
            ]
            .spacing(12)
            .align_y(Alignment::Center)
            .into()
        } else if field.boolean {
            let is_on = matches!(field.value.to_lowercase().as_str(), "1" | "true" | "yes");
            let numeric = field.value == "0" || field.value == "1";
            checkbox(is_on)
                .label(crate::l10n::text(&field.label))
                .on_toggle_maybe((!self.locked()).then_some(move |enabled| {
                    Message::Field(
                        i,
                        if numeric {
                            if enabled { "1" } else { "0" }
                        } else if enabled {
                            "true"
                        } else {
                            "false"
                        }
                        .into(),
                    )
                }))
                .into()
        } else if field.key.starts_with("/key") {
            let selected = field
                .value
                .parse::<i32>()
                .ok()
                .and_then(|id| crate::keys::ALL.iter().find(|key| key.id == id))
                .copied();
            row![
                text(crate::l10n::text(&field.label)).size(14).width(Fill),
                pick_list(crate::keys::ALL, selected, move |key| Message::Field(
                    i,
                    key.id.to_string()
                ))
                .width(130),
            ]
            .spacing(12)
            .align_y(Alignment::Center)
            .into()
        } else {
            row![
                text(crate::l10n::text(&field.label)).size(14).width(Fill),
                text_input("", &field.value)
                    .on_input(move |s| Message::Field(i, s))
                    .padding(8)
                    .width(180),
            ]
            .spacing(24)
            .align_y(Alignment::Center)
            .into()
        }
    }
    fn configuration(&self) -> Element<'_, Message> {
        let Some(root) = self.root() else {
            return self.empty();
        };
        let mut toolbar = row![].spacing(12).align_y(Alignment::Center);
        if self.page == Page::Config {
            let mut names = vec!["游戏设置".to_owned()];
            names.extend(
                self.config_files
                    .iter()
                    .filter(|path| **path != root.join("Bin/Player.ini"))
                    .filter_map(|p| p.file_name())
                    .map(|s| s.to_string_lossy().to_string()),
            );
            toolbar = toolbar.push(pick_list(
                names,
                Some(self.config_name.clone()),
                Message::SelectConfig,
            ));
        }
        let Some(doc) = &self.document else {
            return column![
                toolbar,
                text(crate::l10n::text(if self.busy {
                    "读取中…"
                } else {
                    "无法读取配置，请重新读取或选择其他配置。"
                }))
                .size(14),
                action("重新读取", Message::Refresh, !self.busy),
            ]
            .spacing(18)
            .into();
        };
        let groups = doc.groups();
        if self.page == Page::Data && groups.len() > 1 {
            toolbar = toolbar.push(pick_list(
                groups.clone(),
                Some(self.group.clone()),
                Message::Group,
            ));
        }
        toolbar = toolbar
            .push(
                text_input("搜索配置", &self.search)
                    .on_input(Message::Search)
                    .width(Fill),
            )
            .push(action(
                if doc.dirty { "保存修改" } else { "保存" },
                Message::SaveDocument,
                doc.dirty && !self.locked(),
            ))
            .push(action(
                "重新读取",
                Message::Refresh,
                !self.busy && !self.mutating,
            ));
        if self.page == Page::Data {
            toolbar = toolbar
                .push(action("全部解锁", Message::UnlockAll, !self.locked()))
                .push(action(
                    "重置排行",
                    Message::Prompt(PromptKind::ResetData),
                    !self.locked() && self.group != "关卡",
                ));
        }
        let searching = !self.search.trim().is_empty();
        let mut sections = column![].spacing(10);
        let mut shown = 0;
        for group in &groups {
            if self.page == Page::Data && *group != self.group {
                continue;
            }
            let fields: Vec<_> = doc
                .fields
                .iter()
                .enumerate()
                .filter(|(_, field)| {
                    field.group == *group
                        && (workspace::matches(&field.label, &self.search)
                            || workspace::matches(&field.key, &self.search))
                })
                .collect();
            if fields.is_empty() {
                continue;
            }
            shown += fields.len();
            let open = self.page == Page::Data
                || searching
                || self
                    .config_sections
                    .get(&format!("{}:{group}", self.config_name))
                    .copied()
                    .unwrap_or(!matches!(group.as_str(), "键位与相机" | "高级"));
            let mut section = column![].spacing(4);
            if self.page == Page::Config {
                section = section.push(
                    button(
                        row![
                            text(if open { "⌄" } else { "›" }).size(16).width(16),
                            text(crate::l10n::text(group)).size(14).width(Fill),
                        ]
                        .spacing(5)
                        .align_y(Alignment::Center),
                    )
                    .padding([8, 4])
                    .width(Fill)
                    .style(style::quiet)
                    .on_press(Message::ConfigSection(group.clone())),
                );
            }
            if open {
                let mut keys = Vec::new();
                let mut toggles = Vec::new();
                let resolution = (doc.source == Source::Game).then(|| {
                    (
                        fields.iter().find(|(_, f)| f.key == "Width").copied(),
                        fields.iter().find(|(_, f)| f.key == "Height").copied(),
                    )
                });
                for (i, field) in fields {
                    if field.key.starts_with("/key") {
                        keys.push((i, field));
                    } else if field.boolean && doc.source == Source::Game {
                        toggles.push((i, field));
                    } else if resolution.is_some_and(|(w, h)| w.is_some() && h.is_some())
                        && matches!(field.key.as_str(), "Width" | "Height")
                    {
                        if field.key == "Width" {
                            let (_, height) = resolution.unwrap();
                            let (hi, h) = height.unwrap();
                            section = section.push(
                                container(
                                    row![
                                        text(crate::l10n::text("分辨率")).size(14).width(Fill),
                                        text_input("", &field.value)
                                            .on_input(move |v| Message::Field(i, v))
                                            .padding(8)
                                            .width(80),
                                        text("×").size(14),
                                        text_input("", &h.value)
                                            .on_input(move |v| Message::Field(hi, v))
                                            .padding(8)
                                            .width(80),
                                    ]
                                    .spacing(8)
                                    .align_y(Alignment::Center),
                                )
                                .padding([8, 4]),
                            );
                        }
                    } else {
                        section = section
                            .push(container(self.configuration_field(i, field)).padding([8, 4]));
                    }
                }
                for pair in toggles.chunks(2) {
                    let mut line = row![].spacing(28);
                    for (i, field) in pair {
                        line =
                            line.push(container(self.configuration_field(*i, field)).width(Fill));
                    }
                    if pair.len() == 1 {
                        line = line.push(space::horizontal());
                    }
                    section = section.push(container(line).padding([8, 4]));
                }
                keys.sort_by_key(|(_, field)| {
                    [
                        "/keyForward",
                        "/keyBackward",
                        "/keyLeft",
                        "/keyRight",
                        "/keyRotateCam",
                        "/keyLiftCam",
                    ]
                    .iter()
                    .position(|key| *key == field.key)
                    .unwrap_or(usize::MAX)
                });
                for pair in keys.chunks(2) {
                    let mut line = row![].spacing(28);
                    for (i, field) in pair {
                        line =
                            line.push(container(self.configuration_field(*i, field)).width(Fill));
                    }
                    if pair.len() == 1 {
                        line = line.push(space::horizontal());
                    }
                    section = section.push(container(line).padding([6, 4]));
                }
            }
            sections = sections.push(section).push(rule::horizontal(1));
        }
        if shown == 0 {
            sections = sections.push(text(crate::l10n::text("没有匹配的配置项")).size(14));
        }
        column![
            toolbar,
            scrollable(container(sections).padding(iced::Padding {
                right: 16.0,
                ..iced::Padding::ZERO
            }))
            .height(Fill)
        ]
        .spacing(16)
        .height(Fill)
        .into()
    }
    fn profiles(&self) -> Element<'_, Message> {
        if self.instance.is_none() {
            return self.empty();
        }
        let mut list = column![action(
            "新建方案",
            Message::Prompt(PromptKind::CreateProfile),
            !self.locked()
        )]
        .spacing(14);
        if let Some(profiles) = self.profile_index["profiles"].as_array() {
            for p in profiles {
                let id = p["id"].as_str().unwrap_or("").to_owned();
                let current = self.profile_index["currentId"] == p["id"];
                list = list
                    .push(
                        row![
                            text(p["name"].as_str().unwrap_or("未命名"))
                                .size(15)
                                .width(Fill),
                            action(
                                if current { "当前" } else { "切换" },
                                Message::Profile(profiles::Action::Switch(id.clone())),
                                !current && !self.locked()
                            ),
                            action(
                                "重命名",
                                Message::Prompt(PromptKind::RenameProfile(id.clone())),
                                !self.locked()
                            ),
                            action(
                                "删除",
                                Message::Prompt(PromptKind::DeleteProfile(id)),
                                !current && !self.locked()
                            ),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                    )
                    .push(rule::horizontal(1));
            }
        }
        scrollable(container(list).padding(iced::Padding {
            right: 16.0,
            ..iced::Padding::ZERO
        }))
        .height(Fill)
        .into()
    }
    fn install(&self) -> Element<'_, Message> {
        let mut content = column![
            text(crate::l10n::text(
                "安装到新的空文件夹，完成后即可启动游戏。"
            ))
            .size(14),
            row![
                action("安装 Ballance", Message::InstallGame, !self.locked()),
                checkbox(self.install_player)
                    .label(crate::l10n::text("兼容增强（NewPlayer）"))
                    .text_size(14)
                    .on_toggle(|p| Message::InstallChoice(p, p && self.install_bmlplus)),
                checkbox(self.install_bmlplus)
                    .label(crate::l10n::text("模组与地图（BML+）"))
                    .text_size(14)
                    .on_toggle(|b| Message::InstallChoice(self.install_player || b, b)),
                space::horizontal(),
                action("刷新", Message::Refresh, !self.busy),
            ]
            .spacing(18)
            .align_y(Alignment::Center),
        ]
        .spacing(18);
        if self.instance.is_some() {
            for (index, patch) in self.patches.iter().enumerate() {
                content = content.push(
                    row![
                        text(&patch.name).size(15).width(Fill),
                        text(match &patch.installed {
                            Some(tag) if tag.is_empty() => format!("已安装 · {}", patch.latest),
                            Some(tag) if tag != &patch.latest =>
                                format!("{tag} → {}", patch.latest),
                            Some(tag) => tag.clone(),
                            None => patch.latest.clone(),
                        })
                        .size(14),
                        crate::popup::menu(
                            action(
                                &self
                                    .version
                                    .as_ref()
                                    .filter(|_| self.patch_index == Some(index))
                                    .map(|v| format!("#{}", v.version))
                                    .unwrap_or_else(|| "版本".into()),
                                Message::Versions(index),
                                !self.busy
                            ),
                            if self.version_menu == Some(index) {
                                let mut options =
                                    column![menu_action("最新版本", Message::LatestVersion, true)]
                                        .spacing(3)
                                        .width(224);
                                if self.versions.is_empty() {
                                    options = options.push(
                                        container(text(crate::l10n::text("读取中…")).size(14))
                                            .padding(10),
                                    );
                                }
                                for version in &self.versions {
                                    options = options.push(menu_action(
                                        &version.to_string(),
                                        Message::Version(version.clone()),
                                        true,
                                    ));
                                }
                                Some(menu_panel(scrollable(options).height(240)).into())
                            } else {
                                None
                            },
                            true
                        ),
                        action(
                            if patch.installed.is_some() {
                                "更新 / 重装"
                            } else {
                                "安装"
                            },
                            Message::InstallPatch(index),
                            !self.locked()
                        ),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center),
                );
                content = content.push(rule::horizontal(1));
            }
            content = content.push(action(
                "恢复原版关卡",
                Message::Prompt(PromptKind::Restore),
                !self.locked(),
            ));
        }
        scrollable(container(content).padding(iced::Padding {
            right: 16.0,
            ..iced::Padding::ZERO
        }))
        .height(Fill)
        .into()
    }
    fn tasks(&self) -> Element<'_, Message> {
        let mut list = column![
            row![
                text_input("资源包 HTTPS 地址或 rockoon:// 链接", &self.import_url)
                    .on_input(Message::ImportUrl)
                    .padding(9)
                    .width(Fill),
                action(
                    "导入",
                    Message::ImportRemote,
                    !self.locked() && !self.import_url.trim().is_empty()
                )
            ]
            .spacing(12),
            row![
                text(crate::l10n::text("天空盒目标")).size(14),
                self.sky_picker(),
                space::horizontal(),
                action("清理已完成", Message::ClearJobs, true)
            ]
            .spacing(12)
            .align_y(Alignment::Center),
        ]
        .spacing(20);
        for job in &self.jobs {
            let progress = job.progress();
            let mut controls = row![
                text(job.request.title()).size(15).width(Fill),
                text(progress.phase).size(14)
            ]
            .spacing(12)
            .align_y(Alignment::Center);
            if !progress.done {
                controls = controls.push(action(
                    "取消",
                    Message::CancelJob(job.id.clone()),
                    progress.cancellable,
                ));
            } else if progress.result.as_ref().is_some_and(Result::is_err) {
                controls = controls.push(action(
                    "重试",
                    Message::RetryJob(job.id.clone()),
                    !self.locked(),
                ));
            }
            if progress.done
                && progress.result.as_ref().is_some_and(Result::is_ok)
                && matches!(job.request, jobs::Request::Game { .. })
            {
                controls = controls.push(action(
                    "进入游戏",
                    Message::Page(Page::Home),
                    self.root().as_deref() == Some(job.request.root()),
                ));
            }
            list = list.push(controls);
            if !progress.done {
                list = list
                    .push(
                        progress_bar(
                            0.0..=progress.total.max(1) as f32,
                            progress.downloaded as f32,
                        )
                        .girth(3),
                    )
                    .push(
                        text(format!(
                            "{:.1} / {:.1} MiB · {:.1} MiB/s",
                            progress.downloaded as f64 / 1048576.0,
                            progress.total as f64 / 1048576.0,
                            progress.speed / 1048576.0
                        ))
                        .size(14),
                    );
            }
            if let Some(result) = progress.result {
                list = list.push(
                    text(match result {
                        Ok(s) => s,
                        Err(e) => e,
                    })
                    .size(14),
                );
            }
            list = list.push(rule::horizontal(1));
        }
        scrollable(container(list).padding(iced::Padding {
            right: 16.0,
            ..iced::Padding::ZERO
        }))
        .height(Fill)
        .into()
    }
    fn settings(&self) -> Element<'_, Message> {
        let prefs = &self.preferences;
        let mut content = column![
            row![
                text(crate::l10n::text("外观")).size(14).width(Fill),
                pick_list(
                    preferences::ColorMode::ALL,
                    Some(prefs.color_mode),
                    Message::ColorMode
                ),
            ]
            .align_y(Alignment::Center),
            row![
                text(crate::l10n::text("语言")).size(14).width(Fill),
                pick_list(
                    vec!["中文", "English"],
                    Some(if prefs.language == "en" {
                        "English"
                    } else {
                        "中文"
                    }),
                    |v| Message::PrefText("language".into(), v.into())
                )
                .text_size(14)
            ]
            .align_y(Alignment::Center),
            checkbox(prefs.hide_on_launch)
                .label(crate::l10n::text("启动游戏后隐藏窗口"))
                .text_size(14)
                .on_toggle(|v| Message::PrefBool("hide".into(), v)),
            checkbox(prefs.map_only)
                .label(crate::l10n::text("自制地图独立模式"))
                .text_size(14)
                .on_toggle(|v| Message::PrefBool("map".into(), v)),
            row![
                text(crate::l10n::text("地图槽位")).size(14).width(Fill),
                pick_list(
                    (0u8..=13).collect::<Vec<_>>(),
                    Some(prefs.level_slot),
                    |v| Message::PrefText("slot".into(), v.to_string())
                )
                .text_size(14)
            ]
            .align_y(Alignment::Center),
            row![
                text(crate::l10n::text("游戏内消息")).size(14).width(Fill),
                text_input(&crate::l10n::text("内容"), &prefs.motd)
                    .padding(9)
                    .width(320)
                    .on_input(|v| Message::PrefText("motd".into(), v))
            ]
            .align_y(Alignment::Center),
            row![
                checkbox(prefs.enable_tas)
                    .label(crate::l10n::text("启用 TAS 编辑器"))
                    .text_size(14)
                    .on_toggle(|v| Message::PrefBool("tas".into(), v)),
                space::horizontal(),
                action("打开 TAS 编辑器", Message::OpenTas, prefs.enable_tas)
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            rule::horizontal(1),
            text(if prefs.language == "en" {
                format!(
                    "Played {} hours {} minutes",
                    prefs.playtime / 3600,
                    prefs.playtime % 3600 / 60
                )
            } else {
                format!(
                    "累计游玩 {} 小时 {} 分钟",
                    prefs.playtime / 3600,
                    prefs.playtime % 3600 / 60
                )
            })
            .size(14),
        ]
        .spacing(22);
        if let Some(last) = prefs.last_played {
            let elapsed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .saturating_sub(last);
            let activity = if prefs.language == "en" {
                format!("Last played {} minutes ago", elapsed / 60)
            } else if elapsed < 3600 {
                format!("最近游玩：{} 分钟前", elapsed / 60)
            } else if elapsed < 86400 {
                format!("最近游玩：{} 小时前", elapsed / 3600)
            } else {
                format!("最近游玩：{} 天前", elapsed / 86400)
            };
            content = content.push(text(activity).size(14));
        }
        content = content
            .push(action("检查更新", Message::CheckUpdate, true))
            .push(action(
                if self.protocol_ready {
                    "重新关联一键安装"
                } else {
                    "启用一键安装"
                },
                Message::RegisterProtocol,
                true,
            ));
        if let Some(release) = &self.update_release {
            content = content
                .push(text(format!("Rockoon {}", release.version)).size(16))
                .push(text(&release.notes).size(14));
            content = content.push(action(
                if self.update_path.is_some() {
                    "安装更新并退出"
                } else {
                    "下载更新"
                },
                if self.update_path.is_some() {
                    Message::ApplyUpdate
                } else {
                    Message::DownloadUpdate
                },
                !self.locked(),
            ));
        }
        #[cfg(not(windows))]
        {
            #[cfg(target_os = "macos")]
            {
                content = content.push(action("安装 Rosetta 2", Message::Rosetta, !self.locked()));
            }
            content = content
                .push(action("准备 Wine 环境", Message::WineSetup, !self.locked()))
                .push(action("选择已有 Wine", Message::WineChoose, !self.locked()))
                .push(action("Wine 设置", Message::WineConfig, !self.locked()));
        }
        let hub_origin = jobs::origin();
        for (label, url) in [
            ("百科", "https://ballance.jxpxxzj.cn/wiki/"),
            (
                "制图手册",
                "https://ghomist.github.io/ballance-mapping-manual/",
            ),
            ("论坛", "https://forum.ballance.top/"),
            ("下载站", hub_origin.as_str()),
            ("GitHub", "https://github.com/schtonn/Rockoon"),
        ] {
            content = content.push(action(label, Message::Open(PathBuf::from(url)), true));
        }
        scrollable(container(content).padding(iced::Padding {
            right: 16.0,
            ..iced::Padding::ZERO
        }))
        .height(Fill)
        .into()
    }
    fn timeline(&self) -> Element<'_, Message> {
        let toolbar = row![
            action(
                "新建",
                Message::Tas(TasAction::New),
                !self.tas_playing && !self.tas_busy
            ),
            action(
                "打开",
                Message::Tas(TasAction::Open),
                !self.tas_playing && !self.tas_busy
            ),
            action("导出", Message::Tas(TasAction::Save), !self.tas_busy),
            action("另存为", Message::Tas(TasAction::SaveAs), !self.tas_busy),
            action(
                "撤销",
                Message::Tas(TasAction::Undo),
                !self.tas_playing && !self.tas_busy
            ),
            action(
                "重做",
                Message::Tas(TasAction::Redo),
                !self.tas_playing && !self.tas_busy
            ),
        ]
        .spacing(5);
        let range = row![
            text_input("起始帧", &self.tas_start)
                .padding(8)
                .width(90)
                .on_input(Message::TasStart),
            text("—").size(14),
            text_input("结束帧", &self.tas_end)
                .padding(8)
                .width(90)
                .on_input(Message::TasEnd),
            action("复制", Message::Tas(TasAction::Copy), true),
            action(
                "剪切",
                Message::Tas(TasAction::Cut),
                !self.tas_playing && !self.tas_busy
            ),
            action(
                "粘贴",
                Message::Tas(TasAction::Paste),
                !self.tas_playing && !self.tas_busy
            ),
            text_input("帧数", &self.tas_count)
                .padding(8)
                .width(90)
                .on_input(Message::TasCount),
            action(
                "插入",
                Message::Tas(TasAction::Insert),
                !self.tas_playing && !self.tas_busy
            ),
        ]
        .spacing(6)
        .align_y(Alignment::Center);
        let first = self.tas_offset.min(self.tas.frames.len().saturating_sub(1));
        let end = (first + 24).min(self.tas.frames.len());
        let mut headings = row![space::horizontal().width(80)].spacing(2);
        for i in first..end {
            headings = headings.push(text(i.to_string()).size(12).width(24));
        }
        let mut tracks = column![headings].spacing(3);
        for (key, label) in tas::KEYS.iter().enumerate() {
            let mut track = row![text(*label).size(14).width(80)]
                .spacing(2)
                .align_y(Alignment::Center);
            for i in first..end {
                let on = self.tas.frames[i].keys & (1 << key) != 0;
                track = track.push(
                    button(text(if on { "●" } else { "·" }).size(14))
                        .width(24)
                        .height(24)
                        .padding(0)
                        .style(style::nav(on))
                        .on_press_maybe(
                            (!self.tas_playing && !self.tas_busy)
                                .then_some(Message::Tas(TasAction::Toggle(i, key))),
                        ),
                );
            }
            track = track
                .push(
                    button(text("+").size(14))
                        .width(24)
                        .height(24)
                        .padding(0)
                        .style(style::quiet)
                        .on_press_maybe(
                            (!self.tas_playing && !self.tas_busy)
                                .then_some(Message::Tas(TasAction::Range(key, true))),
                        ),
                )
                .push(
                    button(text("−").size(14))
                        .width(24)
                        .height(24)
                        .padding(0)
                        .style(style::quiet)
                        .on_press_maybe(
                            (!self.tas_playing && !self.tas_busy)
                                .then_some(Message::Tas(TasAction::Range(key, false))),
                        ),
                );
            tracks = tracks.push(track);
        }
        let timeline = scrollable(tracks)
            .direction(iced::widget::scrollable::Direction::Horizontal(
                iced::widget::scrollable::Scrollbar::new(),
            ))
            .height(280);
        let stats = row![
            text(format!(
                "{} 帧{}",
                self.tas.frames.len(),
                if self.tas.dirty { " · 未导出" } else { "" }
            ))
            .size(14)
            .width(Fill),
            action(
                "上一段",
                Message::TasOffset(first.saturating_sub(24)),
                first > 0
            ),
            action(
                "下一段",
                Message::TasOffset(first + 24),
                first + 24 < self.tas.frames.len()
            ),
        ]
        .spacing(10)
        .align_y(Alignment::Center);
        let controls = column![
            row![
                text("关卡").size(14),
                pick_list(
                    (1u8..=13).collect::<Vec<_>>(),
                    Some(self.tas_level),
                    Message::TasLevel
                )
                .text_size(14)
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            row![
                text("目标帧率").size(14),
                text_input("240", &self.tas_fps)
                    .padding(8)
                    .width(90)
                    .on_input(Message::TasFps)
            ]
            .spacing(12)
            .align_y(Alignment::Center),
            action("从头回放", Message::TasPlay(false), !self.locked()),
            action("从起始帧回放", Message::TasPlay(true), !self.locked()),
            action("停止游戏", Message::Stop, self.running.is_some()),
            action("解除嵌入", Message::TasDetach, self.tas_embedded),
            checkbox(self.tas_interactive)
                .label("控制游戏")
                .text_size(14)
                .on_toggle_maybe(self.tas_embedded.then_some(Message::TasInteractive)),
        ]
        .spacing(14);
        let preview = container(space::horizontal())
            .id("tas-preview")
            .width(512)
            .height(384)
            .style(style::sidebar);
        column![
            toolbar,
            range,
            stats,
            timeline,
            row![preview, controls].spacing(24)
        ]
        .spacing(12)
        .height(Fill)
        .into()
    }
}

struct MoreIcon;
impl iced::widget::canvas::Program<Message> for MoreIcon {
    type State = ();
    fn draw(
        &self,
        _: &(),
        renderer: &iced::Renderer,
        theme: &iced::Theme,
        bounds: iced::Rectangle,
        _: iced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        use iced::widget::canvas::{Frame, Path};
        let mut frame = Frame::new(renderer, bounds.size());
        for x in [5.0, 9.0, 13.0] {
            frame.fill(
                &Path::circle(iced::Point::new(x, bounds.height / 2.0), 1.0),
                style::colors(theme).text,
            );
        }
        vec![frame.into_geometry()]
    }
}

struct FolderIcon {
    on_sky: bool,
}
impl iced::widget::canvas::Program<Message> for FolderIcon {
    type State = ();
    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        use iced::widget::canvas::{Frame, Path, Stroke};
        let mut frame = Frame::new(renderer, bounds.size());
        let path = Path::new(|p| {
            p.move_to(iced::Point::new(2.0, 4.0));
            p.line_to(iced::Point::new(8.0, 4.0));
            p.line_to(iced::Point::new(10.0, 7.0));
            p.line_to(iced::Point::new(20.0, 7.0));
            p.line_to(iced::Point::new(20.0, 17.0));
            p.line_to(iced::Point::new(2.0, 17.0));
            p.close();
        });
        frame.stroke(
            &path,
            Stroke::default()
                .with_color(if self.on_sky {
                    iced::Color::WHITE
                } else {
                    style::colors(theme).text
                })
                .with_width(1.5),
        );
        vec![frame.into_geometry()]
    }
}
