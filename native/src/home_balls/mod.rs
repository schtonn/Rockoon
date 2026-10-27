//! Home-only falling original game models; local widget redraws, no WebView/GPU runtime.
mod math;
mod mesh;
mod physics;
mod raster;
use crate::app::Message;
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, image,
    image::Renderer as _,
    layout, mouse, renderer,
    widget::{self, Tree},
};
use iced::{Element, Event, Length, Rectangle, Renderer, Size, Theme};
use std::{
    path::Path,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
#[derive(Debug)]
struct BallAsset {
    mesh: mesh::Mesh,
    texture: bitmap::RgbaImage,
}
#[derive(Debug)]
pub struct Assets {
    balls: [BallAsset; 3],
}
impl Assets {
    pub fn load(root: &Path) -> Option<Arc<Self>> {
        let meshes = mesh::load(root)?;
        let mut balls = Vec::new();
        for (mesh, name) in meshes.into_iter().zip(["Wood", "Stone", "Paper"]) {
            let mut reader =
                bitmap::ImageReader::open(root.join(format!("Textures/Ball_{name}.bmp"))).ok()?;
            let mut limits = bitmap::Limits::default();
            limits.max_image_width = Some(1024);
            limits.max_image_height = Some(1024);
            limits.max_alloc = Some(8 * 1024 * 1024);
            reader.limits(limits);
            let texture = reader.decode().ok()?.into_rgba8();
            if texture.width() == 0 || texture.height() == 0 {
                return None;
            }
            balls.push(BallAsset { mesh, texture });
        }
        Some(Arc::new(Self {
            balls: balls.try_into().ok()?,
        }))
    }
}
const FRAME: Duration = Duration::from_nanos(1_000_000_000 / 60);
const SCALE: f32 = 14.0;
pub fn view(assets: Arc<Assets>, sky: image::Handle, active: bool) -> Element<'static, Message> {
    Element::new(Scene {
        assets,
        sky,
        active,
    })
}
struct Scene {
    sky: image::Handle,
    assets: Arc<Assets>,
    active: bool,
}
struct Motion {
    assets: Option<Arc<Assets>>,
    world: physics::World,
    last: Option<Instant>,
    size: Size,
    frames: Vec<(math::V, image::Handle)>,
    tiles: Vec<(image::Handle, Rectangle)>,
}
impl Default for Motion {
    fn default() -> Self {
        Self {
            assets: None,
            world: physics::World::new(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64,
            ),
            last: None,
            size: Size::ZERO,
            frames: Vec::new(),
            tiles: Vec::new(),
        }
    }
}
impl Widget<Message, Theme, Renderer> for Scene {
    fn tag(&self) -> widget::tree::Tag {
        widget::tree::Tag::of::<Motion>()
    }
    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(Motion::default())
    }
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }
    fn layout(&mut self, _: &mut Tree, _: &Renderer, limits: &layout::Limits) -> layout::Node {
        layout::Node::new(limits.max())
    }
    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        _: mouse::Cursor,
        _: &Renderer,
        _: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<Motion>();
        if state
            .assets
            .as_ref()
            .is_none_or(|a| !Arc::ptr_eq(a, &self.assets))
        {
            *state = Motion::default();
            state.assets = Some(self.assets.clone());
        }
        let size = layout.bounds().size();
        if size != state.size && size.width > 0.0 && size.height > 0.0 {
            state.tiles = sky_tiles(&self.sky, size);
            if state.size.width > 0.0 && state.size.height > 0.0 {
                state.world.resized(
                    size.width / state.size.width,
                    size.height / state.size.height,
                );
            }
            state.size = size;
        }
        if !self.active {
            state.last = None;
            return;
        }
        if let Event::Window(iced::window::Event::RedrawRequested(now)) = event {
            let size = layout.bounds().size();
            if size.width < 1.0 || size.height < 1.0 {
                state.last = None;
                return;
            }
            if state
                .last
                .is_none_or(|last| now.saturating_duration_since(last) >= FRAME)
            {
                let dt = state
                    .last
                    .map(|last| now.saturating_duration_since(last).as_secs_f32())
                    .unwrap_or(physics::STEP);
                state
                    .world
                    .advance(dt, size.width / SCALE, size.height / SCALE);
                state.frames = state
                    .world
                    .poses()
                    .map(|(kind, position, rotation)| {
                        (position, raster::render(&self.assets.balls[kind], rotation))
                    })
                    .collect();
                state.last = Some(*now);
            }
        }
        shell.request_redraw_at(
            state
                .last
                .map(|last| last + FRAME)
                .unwrap_or_else(Instant::now),
        );
    }
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        _: &Theme,
        _: &renderer::Style,
        layout: Layout<'_>,
        _: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<Motion>();
        let bounds = layout.bounds();
        for (tile, rect) in &state.tiles {
            let rect = Rectangle {
                x: bounds.x + rect.x,
                y: bounds.y + rect.y,
                ..*rect
            };
            renderer.draw_image(
                image::Image::new(tile.clone()).filter_method(image::FilterMethod::Nearest),
                rect,
                rect,
            );
        }
        // Keep the directory header clear; balls continue off-screen rather than hitting an invisible floor.
        let clear_header = Rectangle {
            x: bounds.x,
            y: bounds.y + 72.0,
            width: bounds.width,
            height: (bounds.height - 72.0).max(0.0),
        };
        let Some(clip) = clear_header.intersection(viewport) else {
            return;
        };
        for (position, frame) in &state.frames {
            let side = 4.5 * SCALE;
            let rect = Rectangle {
                x: bounds.x + position.0 * SCALE - side * 0.5,
                y: bounds.y + bounds.height - position.1 * SCALE - side * 0.5,
                width: side,
                height: side,
            };
            if let Some(visible) = rect.intersection(&clip) {
                // tiny-skia ignores an image's local clip rectangle. A bounded layer clips
                // both the header and edges while retaining small damage regions.
                use iced::advanced::Renderer as _;
                renderer.with_layer(visible, |renderer| {
                    renderer.draw_image(image::Image::new(frame.clone()), rect, rect)
                });
            }
        }
    }
}

// A full-size scaled bitmap is expensive under tiny-skia's per-damage clip masks.
// Cache small pre-scaled tiles so moving a 60px ball only repaints nearby sky pixels.
fn sky_tiles(sky: &image::Handle, size: Size) -> Vec<(image::Handle, Rectangle)> {
    let image::Handle::Rgba {
        width,
        height,
        pixels,
        ..
    } = sky
    else {
        return Vec::new();
    };
    let Some(source) = bitmap::RgbaImage::from_raw(*width, *height, pixels.to_vec()) else {
        return Vec::new();
    };
    let width = size.width.ceil().clamp(1.0, 4096.0) as u32;
    let height = size.height.ceil().clamp(1.0, 4096.0) as u32;
    let scaled = bitmap::imageops::resize(
        &source,
        width,
        height,
        bitmap::imageops::FilterType::Triangle,
    );
    let mut tiles = Vec::new();
    for y in (0..height).step_by(128) {
        for x in (0..width).step_by(128) {
            let w = 128.min(width - x);
            let h = 128.min(height - y);
            let pixels = bitmap::imageops::crop_imm(&scaled, x, y, w, h).to_image();
            let rect = Rectangle {
                x: x as f32 * size.width / width as f32,
                y: y as f32 * size.height / height as f32,
                width: w as f32 * size.width / width as f32,
                height: h as f32 * size.height / height as f32,
            };
            tiles.push((image::Handle::from_rgba(w, h, pixels.into_raw()), rect));
        }
    }
    tiles
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn optional_original_asset_validation() {
        // Opt-in local integration check; normal tests do not require copyrighted game files.
        let Ok(root) = std::env::var("ROCKOON_BALL_ASSET_TEST_ROOT") else {
            return;
        };
        let assets = Assets::load(Path::new(&root)).expect("original game models and textures");
        assert_eq!(
            assets
                .balls
                .each_ref()
                .map(|a| (a.mesh.positions.len(), a.mesh.faces.len())),
            [(316, 280), (390, 720), (315, 320)]
        );
        for (i, asset) in assets.balls.iter().enumerate() {
            let start = Instant::now();
            let mut rotation = math::Q::default();
            let mut last = None;
            for _ in 0..60 {
                rotation.advance(math::V(0.4, 0.6, 0.3), 1.0 / 60.0);
                last = Some(raster::render(asset, rotation));
            }
            eprintln!("ball {i}: {:?} per sprite", start.elapsed() / 60);
            if let image::Handle::Rgba {
                width,
                height,
                pixels,
                ..
            } = last.unwrap()
            {
                assert!(
                    pixels
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .filter(|p| p[3] > 0)
                        .count()
                        > 3000
                );
                if let Ok(folder) = std::env::var("ROCKOON_BALL_RENDER_DIR") {
                    std::fs::write(Path::new(&folder).join(format!("ball-{i}.rgba")), &pixels)
                        .unwrap();
                    assert_eq!((width, height), (raster::SIDE as u32, raster::SIDE as u32));
                }
            } else {
                panic!("raw sprite");
            }
        }
    }
}
