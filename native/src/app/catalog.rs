use super::*;
use crate::{
    hub,
    icons::{self, Icon},
};
use iced::{
    Alignment, Fill,
    widget::{self, button, column, container, image, markdown, row, space, text},
};
#[derive(Clone, Debug)]
pub enum Action {
    Search(String),
    SearchReady(u64),
    Submit,
    Category(hub::Category),
    Sort(hub::Sort),
    Page(usize),
    Refresh,
    Listed(u64, Result<hub::Listing, String>),
    Open(u64),
    Back,
    Detail(u64, u64, Result<Box<hub::Detail>, String>),
    RetryDetail,
    Choice(hub::Choice),
    Preview(usize),
    Image(hub::ImageKey, Result<image::Handle, String>),
    RetryImage(hub::ImageKey),
    Install(u64, bool),
    Receipts(PathBuf, Result<Vec<hub::Receipt>, String>),
    Link(String),
}
fn message(action: Action) -> Message {
    Message::Hub(action)
}
impl App {
    pub(super) fn ensure_catalog(&mut self) -> Task<Message> {
        let receipts = self.catalog_receipts();
        let request = if !self.hub.loaded && !self.hub.loading {
            self.fetch_catalog(true)
        } else {
            self.catalog_images()
        };
        Task::batch([receipts, request])
    }
    pub(super) fn catalog_receipts(&mut self) -> Task<Message> {
        let Some(root) = self.root() else {
            self.hub.receipts.clear();
            self.hub.receipt_root = None;
            return Task::none();
        };
        if self.hub.receipt_root.as_ref() != Some(&root) {
            self.hub.receipts.clear();
            self.hub.receipt_root = Some(root.clone());
        }
        Task::perform(
            background(move || {
                let result = hub::receipts(&root);
                Ok((root, result))
            }),
            |result| match result {
                Ok((root, r)) => message(Action::Receipts(root, r)),
                Err(e) => message(Action::Receipts(PathBuf::new(), Err(e))),
            },
        )
    }
    fn fetch_catalog(&mut self, increment: bool) -> Task<Message> {
        if increment {
            self.hub.generation += 1;
        }
        self.hub.loading = true;
        self.hub.error = None;
        let revision = self.hub.generation;
        let query = self.hub.query.clone();
        Task::perform(background(move || hub::list(&query)), move |r| {
            message(Action::Listed(revision, r))
        })
    }
    fn catalog_images(&mut self) -> Task<Message> {
        if self.page != Page::Hub {
            return Task::none();
        }
        let mut keys: Vec<_> = if let Some(package) = &self.hub.selected {
            package
                .preview_images
                .iter()
                .take(12)
                .map(|p| hub::ImageKey::thumb(p))
                .collect()
        } else {
            self.hub
                .items
                .iter()
                .filter_map(|p| p.cover().map(hub::ImageKey::thumb))
                .collect()
        };
        if let Some(path) = self
            .hub
            .selected
            .as_ref()
            .and_then(|p| p.preview_images.get(self.hub.preview))
        {
            keys.push(hub::ImageKey::large(path));
        }
        let tasks = keys
            .into_iter()
            .filter_map(|key| {
                if self.hub.images.contains_key(&key)
                    || self.hub.failed_images.contains(&key)
                    || !self.hub.pending_images.insert(key.clone())
                {
                    return None;
                }
                let requested = key.clone();
                Some(Task::perform(hub::fetch_image(key), move |r| {
                    message(Action::Image(requested.clone(), r))
                }))
            })
            .collect::<Vec<_>>();
        Task::batch(tasks)
    }
    fn catalog_open(&mut self, id: u64) -> Task<Message> {
        self.close_menus();
        self.hub.detail_generation += 1;
        self.hub.detail_loading = true;
        self.hub.detail_error = None;
        self.hub.detail = None;
        self.hub.markdown.clear();
        self.hub.choice = hub::Choice::Latest;
        self.hub.preview = 0;
        self.hub.selected = Some(
            self.hub
                .items
                .iter()
                .find(|p| p.id == id)
                .cloned()
                .unwrap_or(hub::Package {
                    id,
                    ..hub::Package::default()
                }),
        );
        let revision = self.hub.detail_generation;
        Task::batch([
            Task::perform(
                background(move || hub::detail(id).map(Box::new)),
                move |r| message(Action::Detail(revision, id, r)),
            ),
            self.catalog_images(),
        ])
    }
    pub(super) fn update_catalog(&mut self, action: Action) -> Task<Message> {
        match action {
            Action::Search(value) => {
                self.hub.query.text = value;
                self.hub.query.page = 0;
                self.hub.generation += 1;
                self.hub.loading = true;
                self.hub.error = None;
                let revision = self.hub.generation;
                return Task::perform(
                    async move {
                        tokio::time::sleep(Duration::from_millis(320)).await;
                        revision
                    },
                    |r| message(Action::SearchReady(r)),
                );
            }
            Action::SearchReady(revision) if revision == self.hub.generation => {
                return self.fetch_catalog(false);
            }
            Action::Submit => return self.fetch_catalog(true),
            Action::Category(category) => {
                self.hub.selected = None;
                self.hub.detail_generation += 1;
                self.hub.detail_loading = false;
                self.hub.query.category = category;
                self.hub.query.page = 0;
                return self.fetch_catalog(true);
            }
            Action::Sort(sort) => {
                self.hub.query.sort = sort;
                self.hub.query.page = 0;
                return self.fetch_catalog(true);
            }
            Action::Page(page) => {
                self.hub.query.page = page;
                return self.fetch_catalog(true);
            }
            Action::Refresh => {
                self.hub.failed_images.clear();
                return self.fetch_catalog(true);
            }
            Action::Listed(revision, result) if revision == self.hub.generation => {
                self.hub.loading = false;
                self.hub.loaded = true;
                match result {
                    Ok(list) => {
                        let pages = list.total.div_ceil(hub::PAGE_SIZE);
                        if list.items.is_empty() && pages > 0 && self.hub.query.page >= pages {
                            self.hub.query.page = pages - 1;
                            return self.fetch_catalog(true);
                        }
                        self.hub.total = list.total;
                        self.hub.items = list.items;
                        return self.catalog_images();
                    }
                    Err(e) => self.hub.error = Some(e),
                }
            }
            Action::Open(id) => {
                if self.page != Page::Hub {
                    self.hub.query.category = self.local_category;
                    self.hub.query.page = 0;
                    self.hub.loaded = false;
                }
                self.page = Page::Hub;
                return self.catalog_open(id);
            }
            Action::Back => {
                self.hub.selected = None;
                self.hub.detail_generation += 1;
                self.hub.detail_loading = false;
                self.hub.detail_error = None;
                if !self.hub.loaded {
                    return self.fetch_catalog(true);
                }
                return self.catalog_images();
            }
            Action::RetryDetail => {
                if let Some(p) = &self.hub.selected {
                    return self.catalog_open(p.id);
                }
            }
            Action::Detail(revision, id, result)
                if revision == self.hub.detail_generation
                    && self.hub.selected.as_ref().is_some_and(|p| p.id == id) =>
            {
                self.hub.detail_loading = false;
                match result {
                    Ok(detail) => {
                        self.hub.markdown =
                            markdown::parse(detail.package.description.as_deref().unwrap_or(""))
                                .collect();
                        self.hub.selected = Some(detail.package.clone());
                        self.hub.detail = Some(*detail);
                        return self.catalog_images();
                    }
                    Err(e) => self.hub.detail_error = Some(e),
                }
            }
            Action::Choice(choice) => self.hub.choice = choice,
            Action::Preview(index) => {
                if self
                    .hub
                    .selected
                    .as_ref()
                    .is_some_and(|p| index < p.preview_images.len())
                {
                    self.hub.preview = index;
                    return self.catalog_images();
                }
            }
            Action::Image(key, result) => self.hub.put_image(key, result),
            Action::RetryImage(key) => {
                self.hub.failed_images.remove(&key);
                return self.catalog_images();
            }
            Action::Receipts(root, result) if self.root().as_ref() == Some(&root) => match result {
                Ok(receipts) => self.hub.receipts = receipts,
                Err(e) => self.error(format!("安装记录读取失败：{e}")),
            },
            Action::Install(id, from_detail) => {
                let package = self
                    .hub
                    .selected
                    .as_ref()
                    .filter(|p| p.id == id)
                    .or_else(|| self.hub.items.iter().find(|p| p.id == id))
                    .cloned();
                let Some(package) = package else {
                    return Task::none();
                };
                if package.source_key.starts_with("bbp-") {
                    return self.update(Message::Page(Page::Mapping));
                }
                if package.managed_component().is_some() {
                    return self.update(Message::Page(Page::Install));
                }
                let Some(root) = self.root() else {
                    self.notify("选择游戏目录后即可安装资源");
                    return self.update(Message::PickFolder);
                };
                if self.locked() {
                    self.error(self.lock_reason());
                    return Task::none();
                }
                let choice = if from_detail {
                    self.hub.choice.clone()
                } else {
                    hub::Choice::Latest
                };
                let task = self.start_job(Request::Resource {
                    root,
                    package,
                    choice,
                    sky: self.sky_letter(),
                });
                self.page = Page::Hub;
                return task;
            }
            Action::Link(link) => {
                let Ok(url) = url::Url::parse(&link) else {
                    self.error("链接无效");
                    return Task::none();
                };
                if !matches!(url.scheme(), "https" | "http") {
                    self.error("仅支持网页链接");
                    return Task::none();
                }
                return Task::perform(
                    background(move || game::open_folder(&PathBuf::from(url.as_str()))),
                    Message::Opened,
                );
            }
            _ => {}
        }
        Task::none()
    }
    fn hub_busy(&self, id: u64) -> bool {
        let Some(root) = self.root() else {
            return false;
        };
        self.jobs.iter().any(|j| {
            !j.progress().done
                && j.request.root() == root
                && matches!(&j.request,Request::Resource{package,..}if package.id==id)
        })
    }
    pub(super) fn online_resources(&self) -> Element<'_, Message> {
        if let Some(package) = &self.hub.selected {
            return self.resource_detail(package);
        }
        let search = widget::text_input("搜索名称、作者或简介", &self.hub.query.text)
            .size(14)
            .padding(6)
            .style(style::input)
            .on_input(|s| message(Action::Search(s)))
            .on_submit(message(Action::Submit));
        let toolbar = row![
            icons::view(Icon::Search, 16.0),
            search,
            picker(hub::Sort::ALL.to_vec(), self.hub.query.sort, |v| message(
                Action::Sort(v)
            )),
            icon_button(
                Icon::Refresh,
                "刷新",
                message(Action::Refresh),
                !self.hub.loading
            )
        ]
        .spacing(10)
        .align_y(Alignment::Center);
        let body: Element<'_, Message> = if self.hub.loading {
            container(text("正在搜索…").size(16))
                .center_x(Fill)
                .center_y(Fill)
                .into()
        } else if let Some(error) = &self.hub.error {
            container(
                column![
                    icons::view(Icon::Warning, 24.0),
                    text("暂时无法连接资源站").size(18),
                    text(error).size(14),
                    icon_button(Icon::Refresh, "重试", message(Action::Refresh), true)
                ]
                .spacing(14),
            )
            .center_x(Fill)
            .center_y(Fill)
            .into()
        } else if self.hub.items.is_empty() {
            container(
                column![
                    icons::view(Icon::Search, 28.0),
                    text("没有找到匹配的资源").size(17),
                    icon_button(
                        Icon::Refresh,
                        "清除搜索",
                        message(Action::Search(String::new())),
                        true
                    )
                ]
                .spacing(14),
            )
            .center_x(Fill)
            .center_y(Fill)
            .into()
        } else {
            widget::responsive(move |size| {
                let columns = if size.width >= 760.0 { 3 } else { 2 };
                let mut grid = column![].spacing(16);
                for chunk in self.hub.items.chunks(columns) {
                    let mut line = row![].spacing(16);
                    for package in chunk {
                        line = line.push(
                            container(self.resource_card(package))
                                .width(iced::Length::FillPortion(1)),
                        );
                    }
                    for _ in chunk.len()..columns {
                        line = line.push(space::horizontal().width(iced::Length::FillPortion(1)));
                    }
                    grid = grid.push(line);
                }
                widget::scrollable(container(grid).padding(iced::Padding::ZERO.right(12)))
                    .height(Fill)
                    .style(style::scroll)
                    .into()
            })
            .into()
        };
        let pages = self.hub.total.div_ceil(hub::PAGE_SIZE).max(1);
        let footer = row![
            text(format!("{} 项资源", self.hub.total)).size(14),
            space::horizontal(),
            icon_button(
                Icon::Left,
                "上一页",
                message(Action::Page(self.hub.query.page.saturating_sub(1))),
                !self.hub.loading && self.hub.query.page > 0
            ),
            text(format!("{} / {pages}", self.hub.query.page + 1)).size(14),
            icon_button(
                Icon::Right,
                "下一页",
                message(Action::Page(self.hub.query.page + 1)),
                !self.hub.loading && self.hub.query.page + 1 < pages
            )
        ]
        .spacing(12)
        .align_y(Alignment::Center);
        column![toolbar, body, footer]
            .spacing(16)
            .height(Fill)
            .into()
    }
    fn resource_card<'a>(&'a self, package: &'a hub::Package) -> Element<'a, Message> {
        let cover = self.resource_cover(package.cover(), false, 128.0);
        let title = button(text(short(package.title(), 28)).size(16).height(42))
            .padding(0)
            .width(Fill)
            .style(style::quiet)
            .on_press(message(Action::Open(package.id)));
        let authors = package.authors();
        let metadata = row![
            text(short(&authors, 25)).size(13).width(Fill),
            text(hub::size(package.file_size)).size(13)
        ]
        .spacing(8);
        let busy = self.hub_busy(package.id);
        let installed = self.hub.installed(package);
        let label = if busy {
            "安装中…"
        } else if package.source_key.starts_with("bbp-") {
            "制图工具"
        } else if package.managed_component().is_some() {
            "管理组件"
        } else if installed {
            "重新安装"
        } else {
            "安装"
        };
        let actions = row![
            icon_button(Icon::Eye, "详情", message(Action::Open(package.id)), true),
            space::horizontal(),
            icon_button(
                if installed {
                    Icon::Check
                } else {
                    Icon::Download
                },
                label,
                message(Action::Install(package.id, false)),
                !self.locked()
            )
        ]
        .spacing(6)
        .align_y(Alignment::Center);
        let content = column![
            button(cover)
                .padding(0)
                .style(style::quiet)
                .on_press(message(Action::Open(package.id))),
            container(column![title, metadata, actions].spacing(10)).padding(12)
        ]
        .spacing(0);
        container(content)
            .width(Fill)
            .style(style::catalog_card)
            .into()
    }
    fn resource_cover<'a>(
        &'a self,
        path: Option<&str>,
        large: bool,
        height: f32,
    ) -> Element<'a, Message> {
        let key = path.map(|p| {
            if large {
                hub::ImageKey::large(p)
            } else {
                hub::ImageKey::thumb(p)
            }
        });
        let handle = key
            .as_ref()
            .and_then(|k| self.hub.image(k))
            .or_else(|| path.and_then(|p| self.hub.image(&hub::ImageKey::thumb(p))));
        if let Some(handle) = handle {
            return crate::cover::view(
                handle.clone(),
                height,
                if large {
                    iced::ContentFit::Contain
                } else {
                    iced::ContentFit::Cover
                },
            );
        }
        let content: Element<'_, Message> =
            if let Some(key) = key.filter(|k| self.hub.failed_images.contains(k)) {
                icon_button(
                    Icon::Refresh,
                    "重试封面",
                    message(Action::RetryImage(key)),
                    true,
                )
            } else {
                icons::view(Icon::Image, 28.0)
            };
        container(content)
            .width(Fill)
            .height(height)
            .center_x(Fill)
            .center_y(height)
            .style(style::catalog_placeholder)
            .into()
    }
    fn resource_hero<'a>(&'a self, package: &'a hub::Package, wide: bool) -> Element<'a, Message> {
        let mut info = column![text(package.title()).size(24)].spacing(14);
        let authors = package.authors();
        if !authors.is_empty() {
            info = info.push(text(authors).size(14));
        }
        info = info.push(
            row![
                text(category_label(&package.category)).size(14).width(Fill),
                text(hub::size(package.file_size)).size(14)
            ]
            .spacing(8),
        );
        if !package.tags.is_empty() {
            info = info.push(
                text(
                    package
                        .tags
                        .iter()
                        .take(12)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join("  "),
                )
                .size(14),
            );
        }
        if !package.mod_formats.is_empty() {
            info = info.push(
                text(
                    package
                        .mod_formats
                        .iter()
                        .map(|m| if m == "bmodp" { "BML+" } else { "BML" })
                        .collect::<Vec<_>>()
                        .join(" / "),
                )
                .size(14),
            );
        }
        if self.hub.detail_loading {
            info = info.push(text("读取详情…").size(14));
        }
        if self
            .hub
            .detail
            .as_ref()
            .is_some_and(|d| d.versions.len() > 1)
        {
            info = info.push(picker(self.hub.choices(), self.hub.choice.clone(), |v| {
                message(Action::Choice(v))
            }));
        }
        if self
            .hub
            .detail
            .as_ref()
            .is_some_and(|d| d.version_error.is_some())
        {
            info = info.push(icon_button(
                Icon::Refresh,
                "重试版本列表",
                message(Action::RetryDetail),
                true,
            ));
        }
        if self.root().is_none() {
            info = info.push(text("选择游戏目录后即可安装").size(14));
        }
        if let Some(root) = self.root()
            && matches!(package.category.as_str(), "map" | "mod" | "bmod" | "bmodp")
            && !root.join("BuildingBlocks/BMLPlus.dll").is_file()
            && !root.join("BuildingBlocks/BML.dll").is_file()
        {
            info = info.push(
                row![
                    text("需要 BML / BML+").size(14),
                    icon_button(
                        Icon::Puzzle,
                        "安装支持",
                        Message::Page(Page::Install),
                        !self.locked()
                    )
                ]
                .spacing(10),
            );
        }
        let label = if self.hub_busy(package.id) {
            "安装中…"
        } else if package.source_key.starts_with("bbp-") {
            "制图工具"
        } else if package.managed_component().is_some() {
            "管理组件"
        } else if self.root().is_none() {
            "选择游戏目录"
        } else if self.hub.installed(package) {
            "重新安装"
        } else {
            "下载并安装"
        };
        info = info.push(
            button(icons::label_color(
                Icon::Download,
                label,
                style::colors(&self.theme()).on_solid,
            ))
            .padding([10, 16])
            .style(style::primary)
            .on_press_maybe(
                (!self.locked() && !self.hub.detail_loading)
                    .then_some(message(Action::Install(package.id, true))),
            ),
        );
        let path = package
            .preview_images
            .get(self.hub.preview)
            .map(String::as_str);
        let cover = self.resource_cover(path, true, 240.0);
        let mut gallery = column![cover].spacing(8);
        if package.preview_images.len() > 1 {
            let mut thumbs = row![].spacing(6);
            for (i, path) in package.preview_images.iter().take(12).enumerate() {
                thumbs = thumbs.push(
                    button(container(self.resource_cover(Some(path), false, 48.0)).width(76))
                        .padding(2)
                        .style(style::tab(i == self.hub.preview))
                        .on_press(message(Action::Preview(i))),
                );
            }
            gallery = gallery.push(
                widget::scrollable(thumbs)
                    .direction(widget::scrollable::Direction::Horizontal(
                        widget::scrollable::Scrollbar::new(),
                    ))
                    .height(62)
                    .style(style::scroll),
            );
        }
        if wide {
            row![
                container(gallery).width(iced::Length::FillPortion(6)),
                container(info).width(iced::Length::FillPortion(5))
            ]
            .spacing(24)
            .into()
        } else {
            column![gallery, info].spacing(20).into()
        }
    }
    fn resource_detail<'a>(&'a self, package: &'a hub::Package) -> Element<'a, Message> {
        let back = icon_button(Icon::Back, "返回资源", message(Action::Back), true);
        let website = icon_button(
            Icon::External,
            "网站页面",
            message(Action::Link(format!(
                "{}/resource/{}",
                jobs::origin(),
                package.id
            ))),
            true,
        );
        let top = row![back, space::horizontal(), website].align_y(Alignment::Center);
        if let Some(error) = &self.hub.detail_error {
            return column![
                top,
                text(error).size(14),
                icon_button(Icon::Refresh, "重试", message(Action::RetryDetail), true)
            ]
            .spacing(16)
            .into();
        }
        let hero = widget::responsive(move |size| self.resource_hero(package, size.width >= 760.0))
            .height(iced::Shrink);
        let mut body = column![hero].spacing(22);
        if let Some(detail) = &self.hub.detail
            && !detail.dependencies.is_empty()
        {
            body = body.push(
                text(format!(
                    "同时安装依赖：{}",
                    detail
                        .dependencies
                        .iter()
                        .map(|(_, name)| name.as_str())
                        .collect::<Vec<_>>()
                        .join("、")
                ))
                .size(14),
            );
        }
        if !self.hub.markdown.is_empty() {
            let mut mdstyle = markdown::Style::from(&self.theme());
            mdstyle.font = iced::Font::with_name("Source Han Sans CN");
            body = body.push(
                markdown::view(
                    &self.hub.markdown,
                    markdown::Settings::with_text_size(14, mdstyle),
                )
                .map(|link| message(Action::Link(link))),
            );
        }
        column![
            top,
            widget::scrollable(container(body).padding(iced::Padding::ZERO.right(12)))
                .height(Fill)
                .style(style::scroll)
        ]
        .spacing(16)
        .height(Fill)
        .into()
    }
}
fn picker<'a, T: Clone + PartialEq + std::fmt::Display + 'a>(
    options: Vec<T>,
    selected: T,
    on_select: impl Fn(T) -> Message + 'a,
) -> widget::PickList<'a, T, Vec<T>, T, Message> {
    widget::pick_list(options, Some(selected), on_select)
        .text_size(14)
        .padding(6)
        .width(184)
        .style(style::select)
        .menu_style(style::select_menu)
}
fn icon_button<'a>(
    icon: Icon,
    label: &str,
    message: Message,
    enabled: bool,
) -> Element<'a, Message> {
    button(icons::label(icon, label))
        .padding([8, 10])
        .style(style::secondary)
        .on_press_maybe(enabled.then_some(message))
        .into()
}
fn short(value: &str, max: usize) -> String {
    let s: String = value.chars().take(max).collect();
    if value.chars().count() > max {
        format!("{s}…")
    } else {
        s
    }
}
fn category_label(value: &str) -> &str {
    match value {
        "map" => "地图",
        "mod" | "bmod" | "bmodp" => "模组",
        "sky" => "天空盒",
        "texture" => "纹理",
        "sound" => "音效",
        "x-patch" => "组件与补丁",
        _ => value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn category_filter_leaves_old_details_and_invalidates_pending_detail_response() {
        let (mut app, _) = App::new();
        app.hub.selected = Some(hub::Package {
            id: 123,
            category: "map".into(),
            ..Default::default()
        });
        app.hub.detail_loading = true;
        let previous = app.hub.detail_generation;
        let _ = app.update_catalog(Action::Category(hub::Category::Sky));
        assert!(app.hub.selected.is_none());
        assert!(!app.hub.detail_loading);
        assert!(app.hub.detail_generation > previous);
        assert_eq!(app.hub.query.category, hub::Category::Sky);
    }
    #[test]
    fn stale_search_results_do_not_replace_the_new_query() {
        let (mut app, _) = App::new();
        let _ = app.update_catalog(Action::Search("old".into()));
        let old = app.hub.generation;
        let _ = app.update_catalog(Action::Search("new".into()));
        let _ = app.update_catalog(Action::Listed(
            old,
            Ok(hub::Listing {
                total: 99,
                items: Vec::new(),
            }),
        ));
        assert_eq!(app.hub.query.text, "new");
        assert_eq!(app.hub.total, 0);
        assert!(app.hub.loading);
    }
}
