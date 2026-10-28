//! Original menu skins animate during widget redraws, without rebuilding the app each frame.
use crate::{app::Message, style};
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, image,
    image::Renderer as _,
    layout, mouse, renderer,
    widget::{self, Tree},
};
use iced::{Element, Event, Length, Rectangle, Renderer, Size, Theme};
use std::time::{Duration, Instant};

/// Match the original menu's label-to-button ratio, centered over the whole 256px skin.
pub fn caption<'a>(
    visuals: &crate::assets::GameAssets,
    label: &str,
    enabled: bool,
) -> Element<'a, Message> {
    use iced::widget::{container, image, text};
    let opacity = if enabled { 1.0 } else { 0.5 };
    let content: Element<'a, Message> = if let Some(caption) = visuals.captions.get(label) {
        let height = 11.0;
        image(caption.handle.clone())
            .width(caption.width as f32 * height / caption.height as f32)
            .height(height)
            .opacity(opacity)
            .into()
    } else {
        text(label.to_uppercase())
            .size(13)
            .color(iced::Color::WHITE.scale_alpha(opacity))
            .into()
    };
    container(content)
        .center_x(iced::Fill)
        .center_y(iced::Fill)
        .into()
}

pub fn button<'a>(
    normal: image::Handle,
    selected: image::Handle,
    caption: Element<'a, Message>,
    enabled: bool,
    message: Message,
) -> Element<'a, Message> {
    sized_button(
        normal,
        selected,
        caption,
        enabled,
        message,
        Size::new(256.0, 64.0),
    )
}
pub fn sized_button<'a>(
    normal: image::Handle,
    selected: image::Handle,
    caption: Element<'a, Message>,
    enabled: bool,
    message: Message,
    size: Size,
) -> Element<'a, Message> {
    iced::widget::button(Element::new(Skin {
        normal,
        selected,
        caption,
        enabled,
        size,
    }))
    .padding(0)
    .style(style::original_button)
    .on_press_maybe(enabled.then_some(message))
    .into()
}

struct Motion {
    from: f32,
    target: f32,
    since: Instant,
}
impl Motion {
    const DURATION: Duration = Duration::from_millis(140);
    fn value(&self) -> f32 {
        let t = (self.since.elapsed().as_secs_f32() / Self::DURATION.as_secs_f32()).min(1.0);
        self.from + (self.target - self.from) * t * t * (3.0 - 2.0 * t)
    }
    fn active(&self) -> bool {
        self.from != self.target && self.since.elapsed() < Self::DURATION
    }
    fn hover(&mut self, hovered: bool) -> bool {
        let target = if hovered { 1.0 } else { 0.0 };
        if target == self.target {
            return false;
        }
        self.from = self.value();
        self.target = target;
        self.since = Instant::now();
        true
    }
}
impl Default for Motion {
    fn default() -> Self {
        Self {
            from: 0.0,
            target: 0.0,
            since: Instant::now(),
        }
    }
}
struct Skin<'a> {
    normal: image::Handle,
    selected: image::Handle,
    caption: Element<'a, Message>,
    enabled: bool,
    size: Size,
}
impl Widget<Message, Theme, Renderer> for Skin<'_> {
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<Motion>()
    }
    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(Motion::default())
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.caption)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.caption));
    }
    fn size(&self) -> Size<Length> {
        Size::new(
            Length::Fixed(self.size.width),
            Length::Fixed(self.size.height),
        )
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.caption.as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            &limits.width(self.size.width).height(self.size.height),
        )
    }
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.caption.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        let state = tree.state.downcast_mut::<Motion>();
        let hovered = self.enabled && cursor.is_over(layout.bounds());
        let changed = state.hover(hovered);
        if changed || state.active() {
            shell.request_redraw();
        }
    }
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<Motion>();
        let progress = if self.enabled { state.value() } else { 0.0 };
        let opacity = if self.enabled { 1.0 } else { 0.5 };
        let bounds = layout.bounds();
        renderer.draw_image(
            image::Image::new(self.normal.clone()).opacity(opacity * (1.0 - progress)),
            bounds,
            bounds,
        );
        renderer.draw_image(
            image::Image::new(self.selected.clone()).opacity(progress),
            bounds,
            bounds,
        );
        self.caption.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }
}
