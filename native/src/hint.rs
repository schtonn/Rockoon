//! Dotted underlines mark concise help that follows the pointer.
use crate::app::Message;
use iced::advanced::{
    Clipboard, Layout, Renderer as _, Shell, Widget, layout, mouse, overlay, renderer,
    widget::{self, Tree},
};
use iced::{Element, Event, Length, Rectangle, Renderer, Size, Theme, Vector};
pub fn label<'a>(label: impl Into<String>, help: impl Into<String>) -> Element<'a, Message> {
    wrap(iced::widget::text(label.into()).size(14), help)
}
pub fn wrap<'a>(
    content: impl Into<Element<'a, Message>>,
    help: impl Into<String>,
) -> Element<'a, Message> {
    let content = content.into();
    let help = help.into();
    if help.is_empty() {
        return content;
    }
    iced::widget::tooltip(
        Element::new(Underline(content)),
        iced::widget::container(iced::widget::text(help).size(14))
            .padding(8)
            .width(320),
        iced::widget::tooltip::Position::FollowCursor,
    )
    .gap(12)
    .delay(std::time::Duration::from_millis(300))
    .style(crate::style::menu)
    .into()
}
struct Underline<'a>(Element<'a, Message>);
impl Widget<Message, Theme, Renderer> for Underline<'_> {
    fn size(&self) -> Size<Length> {
        self.0.as_widget().size()
    }
    fn size_hint(&self) -> Size<Length> {
        self.0.as_widget().size_hint()
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.0)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.0));
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.0
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
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
        self.0.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
        let b = layout.bounds();
        let Some(clip) = b.intersection(viewport) else {
            return;
        };
        renderer.with_layer(clip, |renderer| {
            let mut x = b.x;
            while x < b.x + b.width {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x,
                            y: b.y + b.height - 1.0,
                            width: 2.0_f32.min(b.x + b.width - x),
                            height: 1.0,
                        },
                        ..Default::default()
                    },
                    crate::style::colors(theme).focus,
                );
                x += 4.0;
            }
        });
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
        self.0.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.0
            .as_widget()
            .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.0
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        self.0.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}
