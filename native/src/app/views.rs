use super::*;
use iced::{
    Alignment, Fill,
    widget::{
        button, column, container, image, opaque, progress_bar, row, rule, space, stack, text,
    },
};
const CONTROL_WIDTH: f32 = 184.0;
pub(super) const ROW_HEIGHT: f32 = 36.0;
fn text_input<'a>(placeholder: &str, value: &str) -> iced::widget::TextInput<'a, Message> {
    iced::widget::text_input(placeholder, value)
        .size(14)
        .padding(6)
        .style(style::input)
}
fn search_input<'a>(
    placeholder: &str,
    value: &str,
    changed: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    row![
        crate::icons::view(crate::icons::Icon::Search, 16.0),
        text_input(placeholder, value).on_input(changed).width(Fill)
    ]
    .spacing(9)
    .align_y(Alignment::Center)
    .width(Fill)
    .into()
}
fn checkbox<'a>(checked: bool) -> iced::widget::Checkbox<'a, Message> {
    iced::widget::checkbox(checked)
        .size(18)
        .spacing(9)
        .text_size(14)
        .style(style::check)
}
pub(super) fn pick_list<'a, T, L, V>(
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
        .padding(6)
        .width(CONTROL_WIDTH)
        .style(style::select)
        .menu_style(style::select_menu)
}
pub(super) fn scrollable<'a>(
    content: impl Into<Element<'a, Message>>,
) -> iced::widget::Scrollable<'a, Message> {
    iced::widget::scrollable(content).style(style::scroll)
}
pub(super) fn action<'a>(label: &str, message: Message, enabled: bool) -> Element<'a, Message> {
    let content: Element<'a, Message> = if let Some(icon) = crate::icons::for_label(label) {
        crate::icons::label(icon, crate::l10n::text(label))
    } else {
        text(crate::l10n::text(label)).size(14).into()
    };
    button(container(content).center_y(32))
        .height(32)
        .padding([0, 12])
        .style(style::secondary)
        .on_press_maybe(enabled.then_some(message))
        .into()
}
fn more_button<'a>(message: Message) -> Element<'a, Message> {
    button(crate::icons::view(crate::icons::Icon::More, 16.0))
        .padding(6)
        .width(28)
        .height(28)
        .style(style::quiet)
        .on_press(message)
        .into()
}
pub(super) fn separator<'a>() -> Element<'a, Message> {
    container(rule::horizontal(1)).padding([8, 0]).into()
}
fn resource_action<'a>(label: &str, message: Message, enabled: bool) -> Element<'a, Message> {
    button(container(text(crate::l10n::text(label)).size(14)).center_y(26))
        .height(26)
        .padding([0, 8])
        .style(style::secondary)
        .on_press_maybe(enabled.then_some(message))
        .into()
}
fn menu_action<'a>(label: &str, message: Message, enabled: bool) -> Element<'a, Message> {
    let content: Element<'a, Message> = if let Some(icon) = crate::icons::for_label(label) {
        crate::icons::label(icon, crate::l10n::text(label))
    } else {
        text(crate::l10n::text(label)).size(14).into()
    };
    button(content)
        .padding([8, 10])
        .width(Fill)
        .style(style::quiet)
        .on_press_maybe(enabled.then_some(message))
        .into()
}
fn config_branch<'a>(content: Element<'a, Message>) -> Element<'a, Message> {
    container(
        row![
            rule::vertical(1),
            container(content)
                .padding(iced::Padding::ZERO.left(14))
                .width(Fill)
        ]
        .height(iced::Shrink),
    )
    .padding(iced::Padding::ZERO.left(9).bottom(4))
    .into()
}
fn menu_panel<'a>(
    content: impl Into<Element<'a, Message>>,
) -> iced::widget::Container<'a, Message> {
    container(content).padding(6).style(style::menu)
}
impl App {
    pub fn view(&self) -> Element<'_, Message> {
        crate::keys::recorder(self.view_content(), self.key_capture.is_some())
    }
    fn view_content(&self) -> Element<'_, Message> {
        let colors = style::colors(&self.theme());
        let mut nav = column![].spacing(6);
        for page in [Page::Home, Page::Maps, Page::Config, Page::Tasks] {
            nav = nav.push(self.nav(page));
        }
        nav = nav.push(space::vertical()).push(self.nav(Page::Settings));
        let sidebar = container(
            row![
                container(nav.height(Fill))
                    .padding([24, 18])
                    .width(Fill)
                    .height(Fill),
                rule::vertical(1).style(|theme| rule::Style {
                    color: style::colors(theme).line,
                    ..rule::default(theme)
                }),
            ]
            .width(Fill)
            .height(Fill),
        )
        .width(164)
        .height(Fill)
        .clip(true)
        .style(style::navigation_surface);
        let header = self.header();
        let content: Element<'_, Message> = if self.page == Page::Home {
            stack![self.home(), container(header).padding(24).width(Fill)]
                .width(Fill)
                .height(Fill)
                .clip(true)
                .into()
        } else {
            let body = match self.page {
                Page::Hub => self.online_resources(),
                Page::Mapping => self.mapping_view(),
                Page::Local | Page::Maps | Page::Mods | Page::Assets => self.resources(),
                Page::Config | Page::Data => self.configuration(),
                Page::Profiles => self.profiles(),
                Page::Install => self.install(),
                Page::Tasks => self.tasks(),
                Page::Tas => self.timeline(),
                Page::Settings => self.settings(),
                Page::Home => unreachable!(),
            };
            let tabs = self.section_tabs();
            let body =
                if self.detached_tas || !matches!(self.page.section(), Page::Maps | Page::Config) {
                    body
                } else {
                    column![tabs, body].spacing(16).height(Fill).into()
                };
            let foreground = container(column![header, body].spacing(20).height(Fill))
                .padding(24)
                .width(Fill)
                .height(Fill)
                .into();
            self.frosted_page(foreground)
        };
        let base: Element<'_, Message> = if self.detached_tas {
            content
        } else {
            row![sidebar, content].height(Fill).into()
        };
        let base = if let Some(notice) = &self.notice {
            let alpha = notice.opacity();
            let toast = opaque(
                iced::widget::mouse_area(
                    container(
                        row![
                            text(&notice.text)
                                .size(14)
                                .color(colors.text.scale_alpha(alpha))
                                .width(Fill),
                            button(crate::icons::view(crate::icons::Icon::Close, 16.0))
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
                )
                .on_press(Message::Dismiss),
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
        if let Some((job, name)) = self
            .jobs
            .iter()
            .find_map(|job| job.progress().sky_prompt.map(|name| (job, name)))
        {
            let mut body = column![
                text("导入天空盒").size(19),
                text(name).size(14),
                row![
                    text("替换哪一关的天空？").size(14),
                    space::horizontal(),
                    self.sky_picker()
                ]
                .align_y(Alignment::Center),
            ]
            .spacing(16)
            .width(430);
            if matches!(self.sky.as_str(), "第 4 关" | "第 13 关") {
                body = body.push(text("第 4 关和第 13 关共用同一片天空，会一起更换。").size(14));
            }
            body = body.push(
                row![
                    space::horizontal(),
                    action("取消", Message::SkyDecision(job.id.clone(), false), true),
                    action("导入", Message::SkyDecision(job.id.clone(), true), true),
                ]
                .spacing(8),
            );
            return stack![
                base,
                opaque(
                    container(container(body).padding(24).style(style::menu))
                        .center_x(Fill)
                        .center_y(Fill)
                        .style(style::shade)
                )
            ]
            .into();
        }
        if let Some(conflict) = &self.save_conflict {
            let body = column![
                text("配置发生冲突").size(19),
                text(format!("{conflict}，要用当前修改覆盖吗？")).size(14),
                row![
                    space::horizontal(),
                    action("取消", Message::CancelSaveConflict, self.saving.is_none()),
                    action("覆盖", Message::OverwriteSave, self.saving.is_none()),
                ]
                .spacing(12)
            ]
            .spacing(18)
            .width(430);
            return stack![
                base,
                opaque(
                    container(container(body).padding(24).style(style::menu))
                        .center_x(Fill)
                        .center_y(Fill)
                        .style(style::shade)
                )
            ]
            .into();
        }
        if self.component_editor {
            return stack![
                base,
                opaque(
                    container(self.component_dialog())
                        .center_x(Fill)
                        .center_y(Fill)
                        .style(style::shade)
                )
            ]
            .into();
        }
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
                if self.skybox.is_some() {
                    crate::sky::view(preview.clone())
                } else {
                    image(preview.clone())
                        .width(Fill)
                        .height(Fill)
                        .content_fit(iced::ContentFit::Contain)
                        .into()
                }
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
        .on_press(Message::Page(if page == Page::Home {
            Page::Home
        } else if self.page.section() == page.section() {
            self.page
        } else if page == Page::Maps {
            Page::Hub
        } else if page == Page::Config {
            Page::Install
        } else {
            page
        }))
        .width(Fill)
        .padding([10, 14])
        .style(style::nav(self.page.section() == page.section()))
        .into()
    }
    fn section_tabs(&self) -> Element<'_, Message> {
        if self.page.section() == Page::Maps {
            let mapping = self.page == Page::Mapping;
            let online = if mapping {
                self.mapping.online
            } else {
                self.page == Page::Hub
            };
            let source = row![
                button(container(text("在线").size(14)).center_y(Fill))
                    .padding([0, 12])
                    .height(32)
                    .style(style::tab(online))
                    .on_press_maybe((!mapping).then_some(Message::ResourceSource(true))),
                button(container(text("本地").size(14)).center_y(Fill))
                    .padding([0, 12])
                    .height(32)
                    .style(style::tab(!online))
                    .on_press_maybe((!mapping).then_some(Message::ResourceSource(false))),
            ]
            .spacing(2)
            .align_y(Alignment::Center);
            let tabs = row![
                container(source).padding(2).style(style::tabs),
                pick_list(
                    ResourceFilter::all(),
                    Some(if mapping {
                        ResourceFilter::Mapping
                    } else {
                        ResourceFilter::Category(if online {
                            self.hub.query.category
                        } else {
                            self.local_category
                        })
                    }),
                    Message::ResourceCategory,
                ),
            ]
            .spacing(16)
            .align_y(Alignment::Center);
            return tabs
                .push(space::horizontal())
                .push(action(
                    if self.opening_hub {
                        "正在连接…"
                    } else {
                        "社区资源站"
                    },
                    Message::OpenHub,
                    !self.opening_hub,
                ))
                .width(Fill)
                .into();
        }
        let pages: &[Page] = match self.page.section() {
            Page::Config => &[Page::Install, Page::Config, Page::Data, Page::Profiles],
            _ => &[],
        };
        let mut tabs = row![].spacing(2);
        for &page in pages {
            tabs = tabs.push(
                button(
                    container(
                        text(crate::l10n::text(if page == Page::Config {
                            "选项"
                        } else if page == Page::Profiles {
                            "配置集"
                        } else {
                            page.label()
                        }))
                        .size(14),
                    )
                    .center_y(Fill),
                )
                .padding([0, 12])
                .height(32)
                .style(style::tab(self.page == page))
                .on_press(Message::Page(page)),
            );
        }
        if pages.is_empty() {
            space().into()
        } else {
            container(tabs).padding(2).style(style::tabs).into()
        }
    }
    fn frosted_page<'a>(&self, foreground: Element<'a, Message>) -> Element<'a, Message> {
        let Some(sky) = self.instance.as_ref().and_then(|i| {
            i.visuals
                .frosted_background(&self.preferences.home_background)
        }) else {
            return foreground;
        };
        let opacity: f32 = if style::colors(&self.theme()).background.r > 0.5 {
            0.20
        } else {
            0.14
        };
        stack![
            image(sky.clone())
                .width(Fill)
                .height(Fill)
                .content_fit(iced::ContentFit::Fill)
                .opacity(opacity),
            foreground
        ]
        .width(Fill)
        .height(Fill)
        .clip(true)
        .into()
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
        .on_press_maybe((!self.locked() && !self.picking_folder).then_some(Message::FolderMenu));
        let panel = self.folder_menu.then(|| {
            container(
                column![
                    button(text("打开目录").size(14))
                        .padding([8, 12])
                        .width(Fill)
                        .style(style::quiet)
                        .on_press_maybe(self.root().map(Message::Open)),
                    button(text("更改目录").size(14))
                        .padding([8, 12])
                        .width(Fill)
                        .style(style::quiet)
                        .on_press(Message::PickFolder),
                ]
                .spacing(2),
            )
            .padding(4)
            .width(150)
            .style(style::menu)
            .into()
        });
        let trigger = crate::popup::menu(trigger, panel);
        let path = container(text(path).size(14).color(if self.page == Page::Home {
            iced::Color::WHITE
        } else {
            style::colors(&self.theme()).text
        }))
        .padding([7, 0]);
        let folder_row = row![trigger, path].spacing(8).align_y(Alignment::Center);
        let actions = column![folder_row].spacing(4).align_x(Alignment::Start);
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
        if self.wine.needs_setup() && self.running.is_none() {
            return self.wine_guide();
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
        let start_button: Element<'_, Message> = if let Some(visuals) = self
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
        let foreground = container(
            column![space::vertical(), container(start_button).align_right(Fill)].height(Fill),
        )
        .padding(28)
        .width(Fill)
        .height(Fill);
        if let Some(sky) = self
            .instance
            .as_ref()
            .and_then(|i| i.visuals.background(&self.preferences.home_background))
        {
            stack![
                self.instance
                    .as_ref()
                    .and_then(|i| i.visuals.balls.clone())
                    .map(|balls| crate::home_balls::view(
                        balls,
                        sky.clone(),
                        self.home_active && !self.launching && self.running.is_none()
                    ))
                    .unwrap_or_else(|| image(sky.clone())
                        .width(Fill)
                        .height(Fill)
                        .content_fit(iced::ContentFit::Fill)
                        .into()),
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
        if cfg!(not(windows)) {
            content = content.push(
                text(crate::l10n::text(
                    "选好游戏后，会继续引导你准备 Wine 运行环境。",
                ))
                .size(14),
            );
        }
        content = content.push(
            row![
                button(container(text(crate::l10n::text("选择已有游戏")).size(14)).center_y(Fill))
                    .height(40)
                    .padding([0, 14])
                    .style(style::primary)
                    .on_press_maybe(
                        (!self.locked() && !self.picking_folder).then_some(Message::PickFolder)
                    ),
                button(container(text(crate::l10n::text("下载并安装")).size(14)).center_y(Fill))
                    .height(40)
                    .padding([0, 14])
                    .style(style::secondary)
                    .on_press_maybe((!self.locked()).then_some(Message::Page(Page::Install))),
            ]
            .spacing(12)
            .align_y(Alignment::Center),
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
        if self.page == Page::Local && self.local_category == crate::hub::Category::Patch {
            return self.local_components();
        }
        let scope = self.kind();
        let toolbar = row![
            search_input("搜索", &self.search, Message::Search),
            action("导入", Message::Import, !self.locked()),
            action("刷新", Message::Refresh, !self.busy)
        ]
        .spacing(8)
        .align_y(Alignment::Center);
        let base = scope
            .map(|kind| root.join(kind.dir()))
            .unwrap_or_else(|| root.clone());
        let mut crumbs = row![].spacing(4).align_y(Alignment::Center);
        let nested = scope.is_some() && self.directory != base;
        if nested {
            crumbs = crumbs.push(action("根目录", Message::Browse(base.clone()), !self.busy));
            let mut path = base.clone();
            if let Ok(relative) = self.directory.strip_prefix(&base) {
                for part in relative.components() {
                    path.push(part);
                    crumbs = crumbs.push(text("/").size(14)).push(
                        button(text(part.as_os_str().to_string_lossy().to_string()).size(14))
                            .style(style::quiet)
                            .on_press(Message::Browse(path.clone())),
                    );
                }
            }
        }
        if self.deleted.is_some() {
            crumbs = crumbs.push(action("撤销删除", Message::UndoDelete, !self.locked()));
        }
        if self.sound.is_some() {
            crumbs = crumbs.push(action("停止试听", Message::StopSound, true));
        }
        let mut list = column![].spacing(0);
        let rows = crate::local_resources::rows(&self.entries, &self.search);
        let visible = rows.len();
        for item in rows {
            match item {
                crate::local_resources::LocalRow::File(entry) => {
                    list = list.push(self.local_entry(entry, &root, false))
                }
                crate::local_resources::LocalRow::Sky(group) => {
                    list = list.push(self.local_sky_group(&group));
                    if self.expanded_skies.contains(&group.key) {
                        for entry in group.faces {
                            list = list.push(self.local_entry(entry, &root, true));
                        }
                    }
                }
            }
        }
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
                    text(crate::l10n::text(
                        "可以在“在线”浏览安装资源，也可以导入本地文件。",
                    ))
                    .size(14),
                );
                empty = empty.push(action("浏览在线资源", Message::Page(Page::Hub), true));
            }
            list = list.push(container(empty).padding([24, 0]));
        }
        let mut body = column![toolbar].spacing(10);
        if nested || self.deleted.is_some() || self.sound.is_some() {
            body = body.push(crumbs);
        }
        if matches!(scope, Some(Kind::Maps | Kind::Mods))
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
        let panel = if self.entry_menu.as_ref() == Some(&self.directory) {
            Some(
                menu_panel(
                    column![
                        menu_action("导入", Message::Import, !self.locked()),
                        menu_action(
                            "新建文件夹",
                            Message::Prompt(PromptKind::NewFolder),
                            scope.is_some() && !self.locked()
                        ),
                        menu_action(
                            "打开文件夹",
                            Message::Open(if scope.is_some() {
                                self.directory.clone()
                            } else {
                                root.clone()
                            }),
                            true
                        ),
                        menu_action("刷新", Message::Refresh, !self.busy),
                    ]
                    .width(170),
                )
                .into(),
            )
        } else {
            None
        };
        body.push(crate::popup::context(
            scrollable(
                container(list)
                    .padding(iced::Padding::ZERO.right(12))
                    .width(Fill),
            )
            .height(Fill),
            panel,
            Message::EntryMenu(self.directory.clone()),
        ))
        .height(Fill)
        .into()
    }
    fn local_entry<'a>(
        &'a self,
        entry: &'a Entry,
        root: &std::path::Path,
        sky_child: bool,
    ) -> Element<'a, Message> {
        let kind = entry.kind;
        let primary: Element<'_, Message> = if entry.directory {
            resource_action("打开", Message::Browse(entry.path.clone()), !self.busy)
        } else {
            match kind {
                Kind::Maps => resource_action(
                    "游玩",
                    Message::Launch(Some(entry.path.clone())),
                    !self.locked() && !entry.disabled,
                ),
                Kind::Mods => iced::widget::toggler(!entry.disabled)
                    .size(18)
                    .style(style::toggle)
                    .on_toggle_maybe((!self.locked()).then_some({
                        let entry = entry.clone();
                        move |_| Message::Toggle(entry.clone())
                    }))
                    .into(),
                Kind::Skies if sky_child => space::horizontal().into(),
                Kind::Skies | Kind::Textures => {
                    resource_action("预览", Message::Preview(entry.path.clone()), true)
                }
                Kind::Sounds => resource_action(
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
            if kind == Kind::Maps && !entry.directory && !entry.disabled {
                menu = menu.push(menu_action(
                    "在 Blender 中编辑",
                    Message::Mapping(mapping::Action::OpenMap(entry.path.clone())),
                    !self.locked(),
                ));
            }
            if kind == Kind::Mods && !entry.directory {
                menu = menu.push(menu_action(
                    "配置",
                    Message::FindConfig(entry.path.clone()),
                    true,
                ));
            }
            menu = menu
                .push(menu_action(
                    "打开文件夹",
                    Message::Open(if entry.directory {
                        entry.path.clone()
                    } else {
                        entry.path.parent().unwrap_or(root).to_owned()
                    }),
                    true,
                ))
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
        let glyph = if entry.directory {
            crate::icons::Icon::Folder
        } else {
            match kind {
                Kind::Maps => crate::icons::Icon::Map,
                Kind::Mods => crate::icons::Icon::Puzzle,
                Kind::Sounds => crate::icons::Icon::Volume,
                _ => crate::icons::Icon::Image,
            }
        };
        let line = row![
            crate::icons::view(glyph, 18.0),
            iced::widget::mouse_area(
                container(
                    text(if sky_child {
                        entry.path.file_name().unwrap_or_default().to_string_lossy()
                    } else {
                        std::borrow::Cow::Borrowed(entry.name.as_str())
                    })
                    .size(14)
                )
                .width(Fill)
            )
            .on_double_click(if entry.directory {
                Message::Browse(entry.path.clone())
            } else {
                match kind {
                    Kind::Maps => Message::Launch(Some(entry.path.clone())),
                    Kind::Mods => Message::FindConfig(entry.path.clone()),
                    Kind::Skies if sky_child => Message::CloseMenus,
                    Kind::Skies | Kind::Textures => Message::Preview(entry.path.clone()),
                    Kind::Sounds => Message::Sound(entry.path.clone()),
                }
            }),
            text(if entry.directory {
                String::new()
            } else {
                crate::hub::size(entry.size)
            })
            .size(14)
            .width(88)
            .align_x(Alignment::End),
            more_button(Message::EntryMenu(entry.path.clone())),
            container(primary).width(76).align_x(Alignment::End)
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .width(Fill);
        crate::popup::context(
            container(line)
                .padding(iced::Padding::ZERO.left(if sky_child { 26.0 } else { 0.0 }))
                .height(34)
                .center_y(34)
                .width(Fill),
            panel,
            Message::EntryMenu(entry.path.clone()),
        )
    }
    fn local_sky_group<'a>(
        &'a self,
        group: &crate::local_resources::SkyGroup<'a>,
    ) -> Element<'a, Message> {
        let open = self.expanded_skies.contains(&group.key);
        let panel = if self.entry_menu.as_ref() == Some(&group.key) {
            Some(
                menu_panel(
                    column![
                        menu_action("预览", Message::Preview(group.preview().path.clone()), true),
                        menu_action(
                            "打开文件夹",
                            Message::Open(group.preview().path.parent().unwrap().to_owned()),
                            true
                        ),
                    ]
                    .width(156),
                )
                .into(),
            )
        } else {
            None
        };
        let line = row![
            button(
                container(
                    row![
                        crate::icons::view(
                            if open {
                                crate::icons::Icon::Down
                            } else {
                                crate::icons::Icon::Right
                            },
                            16.0
                        ),
                        text(group.name.clone()).size(14),
                    ]
                    .spacing(10)
                    .align_y(Alignment::Center)
                )
                .center_y(34)
            )
            .style(style::quiet)
            .padding(0)
            .width(Fill)
            .on_press(Message::ExpandSky(group.key.clone())),
            text(crate::hub::size(group.size()))
                .size(14)
                .width(88)
                .align_x(Alignment::End),
            more_button(Message::EntryMenu(group.key.clone())),
            container(resource_action(
                "预览",
                Message::Preview(group.preview().path.clone()),
                true
            ))
            .width(76)
            .align_x(Alignment::End),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .width(Fill);
        crate::popup::context(
            container(line).height(34).center_y(34).width(Fill),
            panel,
            Message::EntryMenu(group.key.clone()),
        )
    }
    fn local_components(&self) -> Element<'_, Message> {
        let Some(instance) = &self.instance else {
            return self.empty();
        };
        let mut components = column![text("Ballance").size(16)].spacing(8);
        for component in &instance.components {
            components = components.push(
                container(
                    row![
                        crate::hint::label(
                            component.name.clone(),
                            crate::components::help(&component.key)
                        ),
                        space::horizontal(),
                        text(&component.version).size(14)
                    ]
                    .align_y(Alignment::Center),
                )
                .height(ROW_HEIGHT)
                .center_y(ROW_HEIGHT),
            );
        }
        components = components.push(action("配置游戏", Message::Page(Page::Install), true));
        let mut list = column![components, separator()].spacing(0);
        for patch in self
            .hub
            .receipts
            .iter()
            .filter(|r| r.category == "x-patch" && workspace::matches(&r.name, &self.search))
        {
            let name = if patch.name.is_empty() {
                format!("补丁 #{}", patch.id)
            } else {
                patch.name.clone()
            };
            let version = patch
                .version_note
                .clone()
                .or_else(|| patch.version.map(|v| format!("版本 #{v}")))
                .unwrap_or_else(|| "已安装".into());
            let line = container(
                row![
                    text(name).size(14).width(Fill),
                    text(version).size(14),
                    resource_action(
                        "详情",
                        Message::Hub(super::catalog::Action::Open(patch.id)),
                        true
                    )
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            )
            .height(34)
            .center_y(34);
            list = list.push(line);
        }
        scrollable(list).height(Fill).into()
    }
    pub(super) fn sky_picker(&self) -> Element<'_, Message> {
        let mut options = vec!["自制地图（X）".to_owned()];
        for i in 1..=13 {
            options.push(format!("第 {i} 关"));
        }
        pick_list(options, Some(self.sky.clone()), Message::Sky)
            .text_size(14)
            .into()
    }
    fn restore_field(&self, i: usize, field: &config::Field) -> Element<'_, Message> {
        let changed = self
            .undo_values
            .get(&(field.key.clone(), field.group.clone()))
            .is_some_and(|value| *value != field.value);
        if !changed {
            return space().width(24).into();
        }
        button(crate::icons::view(crate::icons::Icon::Undo, 15.0))
            .padding(4)
            .style(style::quiet)
            .on_press_maybe((!self.locked()).then_some(Message::RestoreField(i)))
            .into()
    }
    fn configuration_label<'a>(
        &'a self,
        i: usize,
        field: &'a config::Field,
    ) -> Element<'a, Message> {
        row![
            container(crate::hint::label(
                crate::l10n::text(&field.label),
                self.document
                    .as_ref()
                    .and_then(|d| d.field_help(field))
                    .unwrap_or_default(),
            ))
            .width(Fill),
            self.restore_field(i, field)
        ]
        .spacing(6)
        .align_y(Alignment::Center)
        .width(Fill)
        .into()
    }
    fn configuration_field<'a>(
        &'a self,
        i: usize,
        field: &'a config::Field,
    ) -> Element<'a, Message> {
        if let Some(encoding) = field.key_encoding() {
            let capture = self.key_capture.as_ref().filter(|c| c.field == i);
            let caption = match capture {
                Some(c) if c.unsupported => "不支持此键，请重试".into(),
                Some(_) => "按下按键，Esc 取消".into(),
                None => crate::keys::name(encoding, &field.value),
            };
            return row![
                self.configuration_label(i, field),
                button(text(caption).size(14))
                    .padding([5, 8])
                    .width(CONTROL_WIDTH)
                    .style(style::secondary)
                    .on_press_maybe((!self.locked()).then_some(Message::CaptureKey(i)))
            ]
            .spacing(12)
            .align_y(Alignment::Center)
            .into();
        }
        if (field.key == "LevelNumber" && field.group == "CustomMap") || field.key == "BitsPerPixel"
        {
            let options: Vec<String> = if field.key == "LevelNumber" {
                std::iter::once("自动（第 2–11 关）".to_owned())
                    .chain((1..=13).map(|n| format!("第 {n} 关")))
                    .collect()
            } else {
                vec!["16 位".into(), "32 位".into()]
            };
            let selected = if field.key == "LevelNumber" {
                field
                    .value
                    .parse::<usize>()
                    .ok()
                    .and_then(|i| options.get(i))
                    .cloned()
            } else {
                match field.value.as_str() {
                    "16" => options.first().cloned(),
                    "32" => options.get(1).cloned(),
                    _ => None,
                }
            };
            let values = options.clone();
            let level = field.key == "LevelNumber";
            return row![
                self.configuration_label(i, field),
                pick_list(options, selected, move |label| {
                    let index = values.iter().position(|v| *v == label).unwrap_or(0);
                    Message::Field(
                        i,
                        if level {
                            index.to_string()
                        } else if index == 0 {
                            "16".into()
                        } else {
                            "32".into()
                        },
                    )
                })
            ]
            .spacing(12)
            .align_y(Alignment::Center)
            .into();
        }
        if field.key == "Language" {
            const LANGUAGES: [&str; 5] = ["德语", "英语", "西班牙语", "意大利语", "法语"];
            let selected = field
                .value
                .parse::<usize>()
                .ok()
                .and_then(|i| LANGUAGES.get(i))
                .copied();
            return row![
                self.configuration_label(i, field),
                pick_list(LANGUAGES.to_vec(), selected, move |label| Message::Field(
                    i,
                    LANGUAGES
                        .iter()
                        .position(|name| *name == label)
                        .unwrap_or(1)
                        .to_string()
                ))
            ]
            .spacing(12)
            .align_y(Alignment::Center)
            .into();
        }
        if field.key == "/volume" {
            let value = field.value.parse::<f32>().unwrap_or(1.0).clamp(0.0, 1.0);
            let control: Element<'_, Message> = if self.locked() {
                space::horizontal().width(CONTROL_WIDTH).into()
            } else {
                iced::widget::slider(0.0..=1.0, value, move |v| {
                    Message::Field(i, format!("{v:.2}"))
                })
                .step(0.01_f32)
                .width(CONTROL_WIDTH)
                .style(style::range)
                .into()
            };
            row![
                self.configuration_label(i, field),
                control,
                text(format!("{:.0}%", value * 100.0)).size(14).width(44),
            ]
            .spacing(12)
            .align_y(Alignment::Center)
            .into()
        } else if field.boolean {
            row![
                self.configuration_label(i, field),
                iced::widget::toggler(field.enabled())
                    .size(18)
                    .style(style::toggle)
                    .on_toggle_maybe(
                        (!self.locked())
                            .then_some(move |enabled| Message::Field(i, field.toggled(enabled)))
                    )
            ]
            .spacing(12)
            .align_y(Alignment::Center)
            .into()
        } else {
            row![
                self.configuration_label(i, field),
                text_input("", &field.value)
                    .on_input(move |s| Message::Field(i, s))
                    .padding(5)
                    .width(CONTROL_WIDTH),
            ]
            .spacing(24)
            .align_y(Alignment::Center)
            .into()
        }
    }
    pub(super) fn preference_row<'a>(
        &'a self,
        label: &str,
        description: &str,
        control: Element<'a, Message>,
    ) -> Element<'a, Message> {
        container(
            row![
                crate::hint::label(label.to_owned(), description.to_owned()),
                space::horizontal(),
                container(control)
                    .width(CONTROL_WIDTH)
                    .align_x(Alignment::End)
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .height(ROW_HEIGHT)
        .center_y(ROW_HEIGHT)
        .into()
    }
    fn section_open(&self, id: &str, default: bool) -> bool {
        self.config_sections
            .get(&format!("{}:{id}", self.config_name))
            .copied()
            .unwrap_or(default)
    }
    fn config_heading<'a>(&self, label: &str, id: &str, open: bool) -> Element<'a, Message> {
        button(
            row![
                crate::icons::view(
                    if open {
                        crate::icons::Icon::Down
                    } else {
                        crate::icons::Icon::Right
                    },
                    12.0
                ),
                text(crate::l10n::text(label).to_owned())
                    .size(14)
                    .font(iced::Font {
                        weight: iced::font::Weight::Medium,
                        ..iced::Font::with_name("Source Han Sans CN")
                    })
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .padding([6, 3])
        .width(Fill)
        .style(style::quiet)
        .on_press(Message::ConfigSection(id.to_owned(), !open))
        .into()
    }
    fn configuration_rows<'a>(
        &'a self,
        fields: &[(usize, &'a config::Field)],
    ) -> Element<'a, Message> {
        let game = self
            .document
            .as_ref()
            .is_some_and(|d| d.source == Source::Game);
        let mut content = column![].spacing(0);
        let mut paired = Vec::new();
        let width = fields.iter().find(|(_, f)| f.key == "Width").copied();
        let height = fields.iter().find(|(_, f)| f.key == "Height").copied();
        for &(i, field) in fields {
            if game && (field.boolean || field.key.starts_with("/key")) {
                paired.push((i, field));
            } else if game
                && width.is_some()
                && height.is_some()
                && matches!(field.key.as_str(), "Width" | "Height")
            {
                if field.key == "Width" {
                    let (hi, h) = height.unwrap();
                    content = content.push(
                        container(
                            row![
                                text("分辨率").size(14).width(Fill),
                                self.restore_field(i, field),
                                text_input("", &field.value)
                                    .on_input(move |v| Message::Field(i, v))
                                    .padding(5)
                                    .width(80),
                                text("×").size(14),
                                self.restore_field(hi, h),
                                text_input("", &h.value)
                                    .on_input(move |v| Message::Field(hi, v))
                                    .padding(5)
                                    .width(80)
                            ]
                            .spacing(8)
                            .align_y(Alignment::Center),
                        )
                        .height(ROW_HEIGHT)
                        .center_y(ROW_HEIGHT),
                    );
                }
            } else {
                content = content.push(
                    container(self.configuration_field(i, field))
                        .height(ROW_HEIGHT)
                        .center_y(ROW_HEIGHT),
                );
            }
        }
        paired.sort_by_key(|(_, field)| {
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
        for pair in paired.chunks(2) {
            let mut line = row![].spacing(24);
            for &(i, field) in pair {
                line = line.push(container(self.configuration_field(i, field)).width(Fill));
            }
            if pair.len() == 1 {
                line = line.push(space::horizontal());
            }
            content = content.push(container(line).height(ROW_HEIGHT).center_y(ROW_HEIGHT));
        }
        content.into()
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
                names.into_iter().map(config::Choice).collect::<Vec<_>>(),
                Some(config::Choice(self.config_name.clone())),
                |choice| Message::SelectConfig(choice.0),
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
        if self.page == Page::Data {
            return self.save_data(doc);
        }
        let groups = doc.groups();
        toolbar = toolbar
            .push(search_input("搜索配置", &self.search, Message::Search))
            .push(action(
                "重新读取",
                Message::Refresh,
                !self.busy && !self.mutating,
            ));
        let searching = !self.search.trim().is_empty();
        let mut sections = column![].spacing(4);
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
                        && (searching
                            && (workspace::matches(group, &self.search)
                                || workspace::matches(config::group_label(group), &self.search))
                            || workspace::matches(&field.label, &self.search)
                            || workspace::matches(&field.key, &self.search))
                })
                .collect();
            if fields.is_empty() {
                continue;
            }
            shown += fields.len();
            let open = self.page == Page::Data
                || searching
                || self.section_open(group, !matches!(group.as_str(), "键位与相机" | "高级"));
            let mut section = column![].spacing(3);
            if self.page == Page::Config {
                section =
                    section.push(self.config_heading(config::group_label(group), group, open));
            }
            if open {
                let content = self.configuration_rows(&fields);
                section = section.push(if self.page == Page::Config {
                    config_branch(content)
                } else {
                    content
                });
            }
            sections = sections.push(section);
        }
        if shown == 0 {
            sections = sections.push(text(crate::l10n::text("没有匹配的配置项")).size(14));
        }
        let mut content = column![toolbar].spacing(16);
        if let Some(error) = &self.save_error {
            content = content.push(text(error).size(14));
        }
        content
            .push(
                scrollable(container(sections).padding(iced::Padding {
                    right: 16.0,
                    ..iced::Padding::ZERO
                }))
                .height(Fill),
            )
            .height(Fill)
            .into()
    }
    fn save_data(&self, doc: &Document) -> Element<'_, Message> {
        let toolbar = row![
            search_input("搜索关卡或玩家", &self.search, Message::Search),
            action("全部解锁", Message::UnlockAll, !self.locked()),
            action("重新读取", Message::Refresh, !self.busy && !self.mutating)
        ]
        .spacing(8)
        .align_y(Alignment::Center);
        let mut levels = column![].spacing(0);
        for level in 0..13 {
            let name = format!("第 {} 关", level + 1);
            let prefix = format!("/highscores/{level}/");
            if !self.search.trim().is_empty()
                && !workspace::matches(&name, &self.search)
                && !doc.fields.iter().any(|f| {
                    f.key.starts_with(&prefix) && workspace::matches(&f.value, &self.search)
                })
            {
                continue;
            }
            let id = format!("data-level-{level}");
            let open = self.section_open(&id, false);
            let unlock = doc
                .fields
                .iter()
                .enumerate()
                .find(|(_, f)| f.key == format!("/levelLock/{level}"));
            let control: Element<'_, Message> = if let Some((i, f)) = unlock {
                row![
                    text("解锁").size(14),
                    self.restore_field(i, f),
                    iced::widget::toggler(f.enabled())
                        .size(18)
                        .style(style::toggle)
                        .on_toggle_maybe((!self.locked()).then_some(move |on| Message::Field(
                            i,
                            if on { "true" } else { "false" }.into()
                        )))
                ]
                .spacing(10)
                .align_y(Alignment::Center)
                .into()
            } else {
                crate::hint::label("由游戏决定", "这份存档没有第 13 关的独立解锁标记。")
            };
            levels = levels.push(
                container(
                    row![self.config_heading(&name, &id, open), control]
                        .spacing(12)
                        .align_y(Alignment::Center),
                )
                .height(ROW_HEIGHT)
                .center_y(ROW_HEIGHT),
            );
            if open {
                let mut ranks = column![
                    row![
                        text("排名").size(14).width(48),
                        row![text("玩家").size(14).width(Fill), space().width(24)]
                            .spacing(6)
                            .width(Fill),
                        row![
                            text("分数").size(14).width(CONTROL_WIDTH),
                            space().width(24)
                        ]
                        .spacing(6)
                    ]
                    .spacing(10)
                ]
                .spacing(0);
                for rank in 0..10 {
                    let player = doc
                        .fields
                        .iter()
                        .enumerate()
                        .find(|(_, f)| f.key == format!("{prefix}{rank}/player"));
                    let score = doc
                        .fields
                        .iter()
                        .enumerate()
                        .find(|(_, f)| f.key == format!("{prefix}{rank}/score"));
                    if let (Some((pi, p)), Some((si, v))) = (player, score) {
                        ranks = ranks.push(
                            container(
                                row![
                                    text((rank + 1).to_string()).size(14).width(48),
                                    row![
                                        text_input("玩家", &p.value)
                                            .on_input(move |s| Message::Field(pi, s))
                                            .width(Fill),
                                        self.restore_field(pi, p)
                                    ]
                                    .spacing(6)
                                    .align_y(Alignment::Center)
                                    .width(Fill),
                                    row![
                                        text_input("分数", &v.value)
                                            .on_input(move |s| Message::Field(si, s))
                                            .width(CONTROL_WIDTH),
                                        self.restore_field(si, v)
                                    ]
                                    .spacing(6)
                                    .align_y(Alignment::Center)
                                ]
                                .spacing(10)
                                .align_y(Alignment::Center),
                            )
                            .height(ROW_HEIGHT)
                            .center_y(ROW_HEIGHT),
                        );
                    }
                }
                ranks = ranks.push(action(
                    "重置本关排名",
                    Message::ResetLevel(level),
                    !self.locked(),
                ));
                levels = levels.push(config_branch(ranks.into()));
            }
        }
        let mut content = column![toolbar].spacing(12);
        if let Some(error) = &self.save_error {
            content = content.push(text(error).size(14));
        }
        content
            .push(scrollable(container(levels).padding(iced::Padding::ZERO.right(12))).height(Fill))
            .height(Fill)
            .into()
    }
    fn profiles(&self) -> Element<'_, Message> {
        if self.instance.is_none() {
            return self.empty();
        }
        let mut list = column![action(
            "新配置集",
            Message::Prompt(PromptKind::CreateProfile),
            !self.locked()
        )]
        .spacing(8);
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
                    .push(separator());
            }
        }
        scrollable(container(list).padding(iced::Padding {
            right: 16.0,
            ..iced::Padding::ZERO
        }))
        .height(Fill)
        .into()
    }
    fn component_dialog(&self) -> Element<'_, Message> {
        use crate::components::{self, ReleaseChoice};
        let targets = components::targets(&self.patches);
        let selected = targets
            .iter()
            .find(|t| Some(t.index) == self.patch_index)
            .cloned();
        let mut releases = vec![ReleaseChoice::Latest];
        releases.extend(self.versions.iter().cloned().map(ReleaseChoice::Version));
        let release = self
            .version
            .clone()
            .map(ReleaseChoice::Version)
            .unwrap_or(ReleaseChoice::Latest);
        let mut body = column![text("配置当前游戏").size(19)]
            .spacing(14)
            .width(480);
        if self.busy {
            body = body.push(text("读取可安装组件…").size(14));
        } else if self.patch_index.is_some() {
            body = body.push(
                row![
                    text("组件").size(14).width(Fill),
                    pick_list(targets, selected, |t| Message::Versions(t.index)).width(240)
                ]
                .align_y(Alignment::Center),
            );
            body = body.push(
                row![
                    text("版本").size(14).width(Fill),
                    pick_list(releases, Some(release), Message::Version).width(240)
                ]
                .align_y(Alignment::Center),
            );
            if self.versions_loading {
                body = body.push(text("读取历史版本…").size(14));
            }
        }
        if let Some(error) = &self.component_error {
            body = body.push(text(error).size(14));
            body = body.push(action(
                "重试",
                Message::EditComponents(self.component_target.clone()),
                !self.busy,
            ));
        }
        body = body.push(
            row![
                space::horizontal(),
                action("取消", Message::CancelComponents, true),
                action(
                    "安装所选版本",
                    Message::InstallPatch(self.patch_index.unwrap_or(usize::MAX)),
                    !self.locked()
                        && self
                            .patch_index
                            .and_then(|i| self.patches.get(i))
                            .is_some_and(|p| p.package_id.is_some())
                )
            ]
            .spacing(8),
        );
        container(body).padding(24).style(style::menu).into()
    }
    fn install(&self) -> Element<'_, Message> {
        let mut content = column![].spacing(10);
        if let Some(instance) = &self.instance {
            let mut game = column![
                row![
                    text("Ballance").size(18).width(Fill),
                    action(
                        "编辑",
                        Message::EditComponents("player-vc6".into()),
                        !self.locked()
                    )
                ]
                .align_y(Alignment::Center)
            ]
            .spacing(8);
            for component in &instance.components {
                game = game.push(
                    container(
                        row![
                            crate::hint::label(
                                component.name.clone(),
                                crate::components::help(&component.key)
                            ),
                            text(&component.version).size(14).width(Fill),
                        ]
                        .spacing(12)
                        .align_y(Alignment::Center),
                    )
                    .height(ROW_HEIGHT)
                    .center_y(ROW_HEIGHT),
                );
            }
            content = content.push(container(game).padding(16).style(style::menu));
            content = content.push(action(
                "浏览地图和模组",
                Message::BrowseLocalResources,
                true,
            ));
            content = content.push(separator()).push(action(
                "恢复原版关卡",
                Message::Prompt(PromptKind::Restore),
                !self.locked(),
            ));
        }
        let fresh = self.section_open("安装新游戏", self.instance.is_none());
        if self.instance.is_some() {
            content = content.push(separator());
        }
        content = content.push(self.config_heading("安装新游戏", "安装新游戏", fresh));
        if fresh {
            content = content.push(config_branch(
                column![
                    row![
                        checkbox(self.install_player)
                            .on_toggle(|p| Message::InstallChoice(p, p && self.install_bmlplus)),
                        iced::widget::mouse_area(crate::hint::label(
                            "NewPlayer",
                            crate::components::help("player-vc6")
                        ))
                        .on_press(Message::InstallChoice(
                            !self.install_player,
                            !self.install_player && self.install_bmlplus
                        ))
                    ]
                    .spacing(9)
                    .align_y(Alignment::Center),
                    row![
                        checkbox(self.install_bmlplus)
                            .on_toggle(|b| Message::InstallChoice(self.install_player || b, b)),
                        iced::widget::mouse_area(crate::hint::label(
                            "BML+",
                            crate::components::help("bmlplus")
                        ))
                        .on_press(Message::InstallChoice(
                            self.install_player || !self.install_bmlplus,
                            !self.install_bmlplus
                        ))
                    ]
                    .spacing(9)
                    .align_y(Alignment::Center),
                    action("选择目录并安装", Message::InstallGame, !self.locked())
                ]
                .spacing(10)
                .into(),
            ));
        }
        scrollable(container(content).padding(iced::Padding::ZERO.right(12)))
            .height(Fill)
            .into()
    }
    fn tasks(&self) -> Element<'_, Message> {
        let mut list = column![
            row![
                text_input("资源包 HTTPS 地址或 rockoon:// 链接", &self.import_url)
                    .on_input(Message::ImportUrl)
                    .width(Fill),
                action(
                    "导入资源包",
                    Message::ImportRemote,
                    !self.locked() && !self.import_url.trim().is_empty()
                )
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        ]
        .spacing(8);
        if self.jobs.is_empty() {
            list = list.push(
                text("在线资源安装和文件下载的进度会显示在这里，也可以在上方粘贴资源包链接。")
                    .size(14),
            );
        }
        let mut ordered: Vec<_> = self.jobs.iter().collect();
        ordered.sort_by_cached_key(|job| job.progress().done);
        for job in ordered {
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
                            "{:.1} / {:.1} MiB  {:.1} MiB/s",
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
            list = list.push(separator());
        }
        if self
            .jobs
            .iter()
            .any(|job| self.finished_jobs.contains(&job.id))
        {
            list = list.push(
                container(
                    iced::widget::tooltip(
                        button(crate::icons::view(crate::icons::Icon::Close, 16.0))
                            .padding(7)
                            .style(style::quiet)
                            .on_press(Message::ClearJobs),
                        container(text("清理已完成任务").size(14)).padding(8),
                        iced::widget::tooltip::Position::FollowCursor,
                    )
                    .delay(std::time::Duration::ZERO)
                    .gap(12)
                    .style(style::menu),
                )
                .center_x(Fill)
                .padding([6, 0]),
            );
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
        let toggle = |value, key: &'static str| -> Element<'_, Message> {
            iced::widget::toggler(value)
                .size(18)
                .style(style::toggle)
                .on_toggle(move |v| Message::PrefBool(key.into(), v))
                .into()
        };
        let mut backgrounds = column![text("首页背景").size(14)].spacing(10);
        if let Some(instance) = &self.instance {
            for chunk in instance.visuals.backgrounds.chunks(4) {
                let mut choices = row![].spacing(10);
                for background in chunk {
                    choices = choices.push(
                        button(
                            column![crate::cover::view(
                                background.thumbnail.clone(),
                                72.0,
                                iced::ContentFit::Fill
                            ),]
                            .spacing(0),
                        )
                        .padding(2)
                        .style(style::background_choice(
                            self.preferences.home_background == background.letter,
                        ))
                        .on_press(Message::HomeBackground(background.letter.clone()))
                        .width(iced::Length::FillPortion(1)),
                    );
                }
                for _ in chunk.len()..4 {
                    choices = choices.push(space::horizontal().width(iced::Length::FillPortion(1)));
                }
                backgrounds = backgrounds.push(choices);
            }
        }
        let mut content = column![
            self.preference_row(
                "配色",
                "",
                pick_list(
                    preferences::ColorMode::ALL,
                    Some(prefs.color_mode),
                    Message::ColorMode
                )
                .into()
            ),
            self.preference_row(
                "界面语言",
                "",
                pick_list(
                    vec!["中文", "English"],
                    Some(if prefs.language == "en" {
                        "English"
                    } else {
                        "中文"
                    }),
                    |v| Message::PrefText("language".into(), v.into())
                )
                .into()
            ),
            separator(),
            self.preference_row("启动后隐藏窗口", "", toggle(prefs.hide_on_launch, "hide")),
            self.preference_row(
                "退出自制地图后关闭游戏",
                "从启动器游玩自制地图时，离开地图就会关闭游戏。",
                toggle(prefs.map_only, "map")
            ),
            self.preference_row(
                "游戏启动提示",
                "",
                text_input("留空不显示", &prefs.motd)
                    .width(CONTROL_WIDTH)
                    .on_input(|v| Message::PrefText("motd".into(), v))
                    .into()
            ),
            self.preference_row(
                "TAS 编辑器",
                "逐帧编辑和回放游戏输入。",
                action("打开编辑器", Message::OpenTas, true)
            ),
        ]
        .spacing(0);
        content = content
            .push(separator())
            .push(backgrounds)
            .push(separator())
            .push(
                self.preference_row(
                    "累计游玩",
                    "",
                    text(format!(
                        "{} 小时 {} 分钟",
                        prefs.playtime / 3600,
                        prefs.playtime % 3600 / 60
                    ))
                    .size(14)
                    .into(),
                ),
            );
        if let Some(last) = prefs.last_played {
            let elapsed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
                .saturating_sub(last);
            let activity = if prefs.language == "en" {
                format!("Last played {} minutes ago", elapsed / 60)
            } else if elapsed < 3600 {
                format!("{} 分钟前", elapsed / 60)
            } else if elapsed < 86400 {
                format!("{} 小时前", elapsed / 3600)
            } else {
                format!("{} 天前", elapsed / 86400)
            };
            content =
                content.push(self.preference_row("最近游玩", "", text(activity).size(14).into()));
        }
        content = content
            .push(separator())
            .push(self.preference_row(
                "启动器更新",
                "",
                action("检查更新", Message::CheckUpdate, true),
            ))
            .push(self.preference_row(
                "资源站安装链接",
                "让浏览器中的资源站安装链接打开 Rockoon。",
                action(
                    if self.protocol_ready {
                        "重新关联"
                    } else {
                        "关联到 Rockoon"
                    },
                    Message::RegisterProtocol,
                    true,
                ),
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
        if cfg!(not(windows)) {
            content = content.push(separator()).push(self.wine_panel(true));
        }
        content = content.push(separator());
        let hub_origin = jobs::origin();
        let mut links = row![].spacing(4);
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
            links = links.push(action(label, Message::Open(PathBuf::from(url)), true));
        }
        content = content.push(links);
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
                if self.tas.dirty { "  未导出" } else { "" }
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
