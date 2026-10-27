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

pub fn button<'a>(
    normal: image::Handle,
    selected: image::Handle,
    caption: Element<'a, Message>,
    enabled: bool,
    message: Message,
) -> Element<'a, Message> {
    iced::widget::button(Element::new(Skin {
        normal,
        selected,
        caption,
        enabled,
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
        Size::new(Length::Fixed(256.0), Length::Fixed(64.0))
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
            &limits.width(256).height(64),
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
