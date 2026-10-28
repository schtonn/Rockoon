use crate::app::Message;
use iced::advanced::{
    Clipboard, Layout, Renderer as _, Shell, Widget, image,
    image::Renderer as _,
    layout, mouse, renderer,
    widget::{self, Tree},
};
use iced::{ContentFit, Element, Event, Length, Point, Rectangle, Renderer, Size, Theme};

pub fn view(handle: image::Handle) -> Element<'static, Message> {
    Element::new(Viewer(handle))
}
struct Viewer(image::Handle);
#[derive(Default)]
struct Drag {
    last: Option<Point>,
}
impl Viewer {
    fn image_bounds(&self, renderer: &Renderer, bounds: Rectangle) -> Rectangle {
        let size = renderer
            .measure_image(&self.0)
            .map(|s| Size::new(s.width as f32, s.height as f32))
            .unwrap_or(bounds.size());
        let size = ContentFit::Contain.fit(size, bounds.size());
        Rectangle {
            x: bounds.x + (bounds.width - size.width) * 0.5,
            y: bounds.y + (bounds.height - size.height) * 0.5,
            width: size.width,
            height: size.height,
        }
    }
}
impl Widget<Message, Theme, Renderer> for Viewer {
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<Drag>()
    }
    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(Drag::default())
    }
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }
    fn layout(&mut self, _: &mut Tree, _: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::Node::new(limits.resolve(Length::Fill, Length::Fill, Size::ZERO))
    }
    fn draw(
        &self,
        _: &Tree,
        renderer: &mut Renderer,
        _: &Theme,
        _: &renderer::Style,
        layout: Layout<'_>,
        _: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = self.image_bounds(renderer, layout.bounds());
        if let Some(clip) = bounds.intersection(viewport) {
            renderer.with_layer(clip, |r| {
                r.draw_image(image::Image::new(self.0.clone()), bounds, bounds)
            });
        }
    }
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        _: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<Drag>();
        let bounds = self.image_bounds(renderer, layout.bounds());
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if cursor.is_over(bounds)
                    && cursor.position().is_some_and(|p| viewport.contains(p)) =>
            {
                state.last = cursor.position();
                shell.capture_event();
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) if state.last.is_some() => {
                let last = state.last.replace(*position).unwrap();
                let dx = -(position.x - last.x) * 180.0 / bounds.width.max(1.0);
                let dy = (position.y - last.y) * 90.0 / bounds.height.max(1.0);
                shell.publish(Message::SkyOrbit(dx, dy));
                shell.capture_event();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.last.take().is_some() =>
            {
                shell.publish(Message::SkyRender);
                shell.capture_event();
                shell.request_redraw();
            }
            Event::Window(iced::window::Event::Unfocused) if state.last.take().is_some() => {
                shell.publish(Message::SkyRender);
            }
            _ => {}
        }
    }
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if tree.state.downcast_ref::<Drag>().last.is_some() {
            mouse::Interaction::Grabbing
        } else if cursor.is_over(self.image_bounds(renderer, layout.bounds()))
            && cursor.position().is_some_and(|p| viewport.contains(p))
        {
            mouse::Interaction::Grab
        } else {
            mouse::Interaction::None
        }
    }
}
