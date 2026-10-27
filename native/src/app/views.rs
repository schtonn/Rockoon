use super::*;
use iced::{
    Alignment, Fill,
    widget::{
        button, checkbox, column, container, image, opaque, pick_list, progress_bar, row, rule,
        scrollable, space, stack, text, text_input,
    },
};
fn action<'a>(label: &'a str, message: Message, enabled: bool) -> Element<'a, Message> {
    button(text(crate::l10n::text(label)).size(14))
        .padding([8, 12])
        .style(style::quiet)
        .on_press_maybe(enabled.then_some(message))
        .into()
}
impl App {
    pub fn view(&self) -> Element<'_, Message> {
        let mut nav = column![].spacing(6);
        for page in [
            Page::Home,
            Page::Maps,
            Page::Mods,
            Page::Assets,
            Page::Config,
            Page::Data,
            Page::Profiles,
            Page::Install,
            Page::Tasks,
        ] {
            nav = nav.push(self.nav(page));
        }
        if self.preferences.enable_tas {
            nav = nav.push(self.nav(Page::Tas));
        }
        nav = nav.push(space::vertical()).push(self.nav(Page::Settings));
        let sidebar = container(nav.height(Fill))
            .padding([24, 18])
            .width(164)
            .height(Fill)
            .clip(true)
            .style(style::sidebar);
        let mut header = column![self.header()].spacing(12);
        if let Some(notice) = &self.notice {
            header = header.push(
                container(
                    row![
                        text(notice).size(14).width(Fill),
                        action("关闭", Message::Dismiss, true)
                    ]
                    .spacing(12)
                    .align_y(Alignment::Center),
                )
                .padding(12)
                .style(style::notice),
            );
        }
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
                        .on_release(Message::SkyRender),
                        text(crate::l10n::text("俯仰")).size(14),
                        iced::widget::slider(-80.0..=80.0, self.sky_pitch, |v| Message::SkyAngle(
                            false, v
                        ))
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
            text(crate::l10n::text(page.label()))
                .size(15)
                .font(iced::Font {
                    weight: iced::font::Weight::Medium,
                    ..iced::Font::with_name("Source Han Sans CN")
                }),
        )
        .on_press(Message::Page(page))
        .width(Fill)
        .padding([10, 14])
        .style(style::nav(self.page == page))
        .into()
    }
    fn header(&self) -> Element<'_, Message> {
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
        let folder = button(iced::widget::canvas(FolderIcon).width(22).height(20))
            .padding([7, 9])
            .style(style::quiet)
            .on_press(Message::FolderMenu);
        let actions = row![folder, text(path).size(14)]
            .spacing(8)
            .align_y(Alignment::Center);
        let mut header = row![].align_y(Alignment::Center);
        if self.page != Page::Home {
            header = header.push(text(crate::l10n::text(self.page.label())).size(22));
        }
        let mut result = column![header.push(space::horizontal()).push(actions)]
            .spacing(8)
            .width(Fill);
        if self.folder_menu {
            let mut menu = column![].spacing(4).width(156);
            if let Some(root) = self.root() {
                menu = menu.push(action("打开目录", Message::Open(root), true));
            }
            menu = menu.push(action(
                "更换目录",
                Message::PickFolder,
                !self.locked() && !self.picking_folder,
            ));
            result = result.push(
                container(container(menu).padding(8).style(style::sidebar)).align_right(Fill),
            );
        }
        result.into()
    }
    fn home(&self) -> Element<'_, Message> {
        let enabled =
            !self.busy && !self.launching && !self.jobs.iter().any(|j| !j.progress().done);
        let (label, message) = if self.launching {
            ("正在启动…", Message::Launch(None))
        } else if self.running.is_some() {
            ("结束游戏", Message::Stop)
        } else if self.instance.is_some() {
            ("启动游戏", Message::Launch(None))
        } else {
            ("选择游戏目录", Message::PickFolder)
        };
        let action: Element<'_, Message> = if let Some(atlas) = self
            .instance
            .as_ref()
            .and_then(|i| i.visuals.launch_button())
        {
            let skin = image(atlas.clone())
                .width(256)
                .height(64)
                .opacity(if enabled { 1.0_f32 } else { 0.5_f32 });
            let caption = container(
                text(crate::l10n::text(label))
                    .size(16)
                    .color(iced::Color::WHITE),
            )
            .padding(iced::Padding {
                left: 42.0,
                ..iced::Padding::ZERO
            })
            .center_x(Fill)
            .center_y(Fill);
            button(stack![skin, caption].width(256).height(64).clip(true))
                .padding(0)
                .style(style::original_button)
                .on_press_maybe(enabled.then_some(message))
                .into()
        } else {
            button(text(crate::l10n::text(label)).size(15))
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
    fn empty(&self) -> Element<'_, Message> {
        container(action("选择游戏目录", Message::PickFolder, !self.locked()))
            .center_x(Fill)
            .center_y(Fill)
            .into()
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
            let line = row![
                text(&entry.name).size(14).width(Fill),
                primary,
                action("···", Message::EntryMenu(entry.path.clone()), true)
            ]
            .spacing(8)
            .align_y(Alignment::Center);
            list = list.push(container(line).padding([8, 0]));
            if self.entry_menu.as_ref() == Some(&entry.path) {
                if kind == Kind::Mods && !entry.directory {
                    list = list.push(
                        container(action(
                            "配置",
                            Message::FindConfig(entry.path.clone()),
                            true,
                        ))
                        .align_right(Fill),
                    );
                }
                list = list.push(
                    row![
                        space::horizontal(),
                        action(
                            "重命名",
                            Message::Prompt(PromptKind::Rename(entry.path.clone())),
                            !self.locked()
                        ),
                        action("移动", Message::Move(entry.path.clone()), !self.locked()),
                        action(
                            "删除",
                            Message::Prompt(PromptKind::Delete(entry.path.clone())),
                            !self.locked()
                        ),
                    ]
                    .spacing(8),
                );
            }
            list = list.push(rule::horizontal(1));
        }
        if self.entries.is_empty() {
            list = list.push(text(crate::l10n::text("暂无资源")).size(14));
        }
        let mut body = column![toolbar, crumbs].spacing(12);
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
    fn configuration(&self) -> Element<'_, Message> {
        if self.instance.is_none() {
            return self.empty();
        }
        let mut toolbar = row![].spacing(12).align_y(Alignment::Center);
        if self.page == Page::Config {
            let mut names = vec!["游戏选项".to_owned()];
            names.extend(
                self.config_files
                    .iter()
                    .filter_map(|p| p.file_name())
                    .map(|s| s.to_string_lossy().to_string()),
            );
            toolbar = toolbar.push(
                pick_list(names, Some(self.config_name.clone()), Message::SelectConfig)
                    .text_size(14),
            );
        }
        let Some(doc) = &self.document else {
            return column![
                toolbar,
                text(if self.busy {
                    "读取中…"
                } else {
                    "无法读取配置"
                })
                .size(14)
            ]
            .spacing(18)
            .into();
        };
        let groups = doc.groups();
        if groups.len() > 1 {
            toolbar = toolbar
                .push(pick_list(groups, Some(self.group.clone()), Message::Group).text_size(14));
        }
        toolbar = toolbar
            .push(
                text_input("搜索配置", &self.search)
                    .on_input(Message::Search)
                    .padding(9)
                    .width(Fill),
            )
            .push(action(
                if doc.dirty { "保存修改" } else { "保存" },
                Message::SaveDocument,
                doc.dirty && !self.locked(),
            ));
        toolbar = toolbar.push(action(
            "重新读取",
            Message::Refresh,
            !self.busy && !self.mutating,
        ));
        if self.page == Page::Data {
            toolbar = toolbar.push(action("全部解锁", Message::UnlockAll, !self.locked()));
            toolbar = toolbar.push(action(
                "重置排行",
                Message::Prompt(PromptKind::ResetData),
                !self.locked() && self.group != "关卡",
            ));
        }
        let mut fields = column![].spacing(0);
        for (i, field) in doc.fields.iter().enumerate().filter(|(_, f)| {
            f.group == self.group
                && (workspace::matches(&f.label, &self.search)
                    || workspace::matches(&f.key, &self.search))
        }) {
            let control: Element<'_, Message> = if field.boolean {
                let is_on = matches!(field.value.to_lowercase().as_str(), "1" | "true" | "yes");
                let numeric = field.value == "0" || field.value == "1";
                checkbox(is_on)
                    .label(crate::l10n::text(&field.label))
                    .text_size(14)
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
                    .and_then(|id| crate::keys::ALL.iter().find(|k| k.id == id))
                    .copied();
                row![
                    text(crate::l10n::text(&field.label)).size(14).width(Fill),
                    pick_list(crate::keys::ALL, selected, move |key| Message::Field(
                        i,
                        key.id.to_string()
                    ))
                    .text_size(14)
                ]
                .align_y(Alignment::Center)
                .into()
            } else {
                row![
                    text(crate::l10n::text(&field.label)).size(14).width(Fill),
                    text_input("", &field.value)
                        .on_input(move |s| Message::Field(i, s))
                        .padding(8)
                        .width(220)
                ]
                .spacing(24)
                .align_y(Alignment::Center)
                .into()
            };
            fields = fields
                .push(container(control).padding([11, 0]))
                .push(rule::horizontal(1));
        }
        column![
            toolbar,
            scrollable(container(fields).padding(iced::Padding {
                right: 16.0,
                ..iced::Padding::ZERO
            }))
            .height(Fill)
        ]
        .spacing(18)
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
            row![
                action("安装 Ballance", Message::InstallGame, !self.locked()),
                checkbox(self.install_player)
                    .label(crate::l10n::text("NewPlayer"))
                    .text_size(14)
                    .on_toggle(|p| Message::InstallChoice(p, self.install_bmlplus)),
                checkbox(self.install_bmlplus)
                    .label(crate::l10n::text("BML+"))
                    .text_size(14)
                    .on_toggle(|b| Message::InstallChoice(self.install_player, b)),
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
                        action("版本", Message::Versions(index), !self.busy),
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
                if self.patch_index == Some(index) {
                    content = content.push(
                        pick_list(
                            self.versions.clone(),
                            self.version.clone(),
                            Message::Version,
                        )
                        .placeholder("最新版本")
                        .text_size(14),
                    );
                }
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
            checkbox(prefs.enable_tas)
                .label(crate::l10n::text("显示 TAS 编辑器"))
                .text_size(14)
                .on_toggle(|v| Message::PrefBool("tas".into(), v)),
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
                "注册下载站一键安装",
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
            .direction(scrollable::Direction::Horizontal(
                scrollable::Scrollbar::new(),
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

struct FolderIcon;
impl iced::widget::canvas::Program<Message> for FolderIcon {
    type State = ();
    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
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
            Stroke::default().with_color(style::INK).with_width(1.5),
        );
        vec![frame.into_geometry()]
    }
}
