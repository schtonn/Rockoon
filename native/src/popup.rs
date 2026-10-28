//! Anchored menus lay out only their trigger in the document.
use crate::app::Message;
use iced::advanced::Renderer as _;
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer,
    widget::{self, Tree},
};
use iced::{Element, Event, Length, Point, Rectangle, Renderer, Size, Theme, Vector};

/// A menu anchored beneath its trigger without taking space in the content flow.
pub fn menu<'a>(
    trigger: impl Into<Element<'a, Message>>,
    panel: Option<Element<'a, Message>>,
) -> Element<'a, Message> {
    Element::new(Popup {
        trigger: trigger.into(),
        open: panel.is_some(),
        panel: panel.unwrap_or_else(|| iced::widget::space().into()),
        right: false,
        context: None,
    })
}
/// File-manager style menu at the pointer; nested contexts consume the click first.
pub fn context<'a>(
    content: impl Into<Element<'a, Message>>,
    panel: Option<Element<'a, Message>>,
    on_open: Message,
) -> Element<'a, Message> {
    Element::new(Popup {
        trigger: content.into(),
        open: panel.is_some(),
        panel: panel.unwrap_or_else(|| iced::widget::space().into()),
        right: false,
        context: Some(on_open),
    })
}
#[derive(Default)]
struct Pointer {
    at: Option<Point>,
}
struct Popup<'a> {
    trigger: Element<'a, Message>,
    panel: Element<'a, Message>,
    open: bool,
    right: bool,
    context: Option<Message>,
}
impl Widget<Message, Theme, Renderer> for Popup<'_> {
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<Pointer>()
    }
    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(Pointer::default())
    }
    fn size(&self) -> Size<Length> {
        self.trigger.as_widget().size()
    }
    fn size_hint(&self) -> Size<Length> {
        self.trigger.as_widget().size_hint()
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.trigger), Tree::new(&self.panel)]
    }
    fn diff(&self, tree: &mut Tree) {
        if tree.children.len() != 2 {
            tree.children = self.children();
        }
        tree.children[0].diff(&self.trigger);
        tree.children[1].diff(&self.panel);
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.trigger
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
        self.trigger.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
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
        // Left-clicking a row's more button must anchor the same menu at that click,
        // including after this row previously opened a right-click menu elsewhere.
        if self.context.is_some()
            && matches!(
                event,
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            )
            && cursor.is_over(layout.bounds())
            && cursor.position().is_some_and(|p| viewport.contains(p))
        {
            tree.state.downcast_mut::<Pointer>().at = cursor.position();
        }
        self.trigger.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        if !shell.is_event_captured()
            && matches!(
                event,
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right))
            )
            && cursor.is_over(layout.bounds())
            && cursor.position().is_some_and(|p| viewport.contains(p))
            && let Some(message) = &self.context
        {
            tree.state.downcast_mut::<Pointer>().at = cursor.position();
            shell.publish(message.clone());
            shell.capture_event();
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
        self.trigger.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.trigger
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
        if self.open {
            let anchor = if self.context.is_some() {
                let at = tree
                    .state
                    .downcast_ref::<Pointer>()
                    .at
                    .unwrap_or(layout.position());
                Rectangle::new(at, Size::ZERO)
            } else {
                layout.bounds() + translation
            };
            if if self.context.is_some() {
                !viewport.contains(anchor.position())
            } else {
                !anchor.intersects(viewport)
            } {
                return None;
            }
            Some(overlay::Element::new(Box::new(Menu {
                panel: &mut self.panel,
                tree: &mut tree.children[1],
                anchor,
                right: self.right,
            })))
        } else {
            self.trigger.as_widget_mut().overlay(
                &mut tree.children[0],
                layout,
                renderer,
                viewport,
                translation,
            )
        }
    }
}
struct Menu<'a, 'b> {
    panel: &'b mut Element<'a, Message>,
    tree: &'b mut Tree,
    anchor: Rectangle,
    right: bool,
}
impl overlay::Overlay<Message, Theme, Renderer> for Menu<'_, '_> {
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let limits = layout::Limits::new(
            Size::ZERO,
            Size::new(
                (bounds.width - 16.0).max(0.0),
                (bounds.height - 16.0).max(0.0),
            ),
        );
        let node = self
            .panel
            .as_widget_mut()
            .layout(self.tree, renderer, &limits);
        let size = node.size();
        let x = if self.right {
            self.anchor.x + self.anchor.width - size.width
        } else {
            self.anchor.x
        };
        let below = self.anchor.y + self.anchor.height + 5.0;
        let y = if below + size.height <= bounds.height - 8.0 {
            below
        } else {
            self.anchor.y - size.height - 5.0
        };
        node.move_to(Point::new(
            x.clamp(8.0, (bounds.width - size.width - 8.0).max(8.0)),
            y.clamp(8.0, (bounds.height - size.height - 8.0).max(8.0)),
        ))
    }
    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        renderer.with_layer(layout.bounds(), |renderer| {
            self.panel.as_widget().draw(
                self.tree,
                renderer,
                theme,
                style,
                layout,
                cursor,
                &layout.bounds(),
            );
        });
    }
    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let escape = matches!(
            event,
            Event::Keyboard(iced::keyboard::Event::KeyPressed {
                key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
                ..
            })
        );
        let outside = match event {
            Event::Touch(iced::touch::Event::FingerPressed { position, .. }) => {
                !layout.bounds().contains(*position)
            }
            Event::Mouse(mouse::Event::ButtonPressed(_) | mouse::Event::WheelScrolled { .. }) => {
                !cursor.is_over(layout.bounds())
            }
            _ => false,
        };
        if escape || outside {
            shell.publish(Message::CloseMenus);
            if !outside
                || !matches!(
                    event,
                    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right))
                )
            {
                shell.capture_event();
            }
            return;
        }
        self.panel.as_widget_mut().update(
            self.tree,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &layout.bounds(),
        );
    }
    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.panel.as_widget().mouse_interaction(
            self.tree,
            layout,
            cursor,
            &layout.bounds(),
            renderer,
        )
    }
    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        self.panel
            .as_widget_mut()
            .operate(self.tree, layout, renderer, operation);
    }
    fn overlay<'a>(
        &'a mut self,
        layout: Layout<'a>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        self.panel.as_widget_mut().overlay(
            self.tree,
            layout,
            renderer,
            &layout.bounds(),
            Vector::ZERO,
        )
    }
}
