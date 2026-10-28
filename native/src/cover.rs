//! Explicit renderer clipping for native cover images; tiny-skia ignores image-local clip bounds.
use iced::advanced::{
    Layout, Renderer as _, Widget, image, image::Renderer as _, layout, mouse, renderer,
    widget::Tree,
};
use iced::{ContentFit, Element, Length, Rectangle, Renderer, Size, Theme};
pub fn view<'a, M: 'a>(handle: image::Handle, height: f32, fit: ContentFit) -> Element<'a, M> {
    Element::new(Cover {
        handle,
        height,
        fit,
    })
}
struct Cover {
    handle: image::Handle,
    height: f32,
    fit: ContentFit,
}
impl<M> Widget<M, Theme, Renderer> for Cover {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fixed(self.height))
    }
    fn layout(&mut self, _: &mut Tree, _: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::Node::new(limits.resolve(Length::Fill, Length::Fixed(self.height), Size::ZERO))
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
        let bounds = layout.bounds();
        let Some(clip) = bounds.intersection(viewport) else {
            return;
        };
        let Some(size) = renderer.measure_image(&self.handle) else {
            return;
        };
        let size = self.fit.fit(
            Size::new(size.width as f32, size.height as f32),
            bounds.size(),
        );
        let rect = Rectangle {
            x: bounds.x + (bounds.width - size.width) * 0.5,
            y: bounds.y + (bounds.height - size.height) * 0.5,
            width: size.width,
            height: size.height,
        };
        renderer.with_layer(clip, |renderer| {
            renderer.draw_image(image::Image::new(self.handle.clone()), rect, rect)
        });
    }
}
