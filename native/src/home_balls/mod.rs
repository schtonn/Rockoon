//! Home-only falling original game models; local widget redraws, no WebView/GPU runtime.
mod camera;
mod flames;
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
    platform: Option<PlatformAsset>,
}
#[derive(Debug)]
struct PlatformAsset {
    mesh: mesh::Mesh,
    textures: Vec<bitmap::RgbaImage>,
    collider: Arc<physics::Collider>,
    flames: Option<bitmap::RgbaImage>,
}
impl PlatformAsset {
    fn load(root: &Path) -> Option<Self> {
        let mut mesh = mesh::platform(root)?;
        // Original checkpoint platform, rotated 90 degrees and uniformly scaled.
        // The same transformed vertices drive both rendering and collisions.
        for p in &mut mesh.positions {
            *p = math::V(p.2, p.1 - 2.5, -p.0) * 1.2;
        }
        for n in &mut mesh.normals {
            *n = math::V(n.2, n.1, -n.0);
        }
        let textures = [
            "Floor_Side.bmp",
            "Floor_Top_Checkpoint.bmp",
            "Floor_Top_Profil.bmp",
        ]
        .into_iter()
        .map(|name| load_texture(&root.join("Textures").join(name)))
        .collect::<Option<Vec<_>>>()?;
        let collider = Arc::new(physics::Collider::new(&mesh));
        Some(Self {
            mesh,
            textures,
            collider,
            flames: load_texture(&root.join("Textures/Particle_Flames.bmp")),
        })
    }
}
fn load_texture(path: &Path) -> Option<bitmap::RgbaImage> {
    let mut reader = bitmap::ImageReader::open(path).ok()?;
    let mut limits = bitmap::Limits::default();
    limits.max_image_width = Some(1024);
    limits.max_image_height = Some(1024);
    limits.max_alloc = Some(8 * 1024 * 1024);
    reader.limits(limits);
    let texture = reader.decode().ok()?.into_rgba8();
    (texture.width() > 0 && texture.height() > 0).then_some(texture)
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
            platform: PlatformAsset::load(root),
        }))
    }
}
const FRAME: Duration = Duration::from_nanos(1_000_000_000 / 60);
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
type Tiles = Vec<(image::Handle, Rectangle)>;
struct Motion {
    assets: Option<Arc<Assets>>,
    world: physics::World,
    last: Option<Instant>,
    size: Size,
    frames: Vec<(Rectangle, image::Handle)>,
    platform: Option<raster::Frame>,
    platform_tiles: Vec<(image::Handle, Rectangle)>,
    tiles: Tiles,
    background: Option<bitmap::RgbaImage>,
    flame_time: f32,
    flame_last: f32,
    flame_frames: Vec<(Rectangle, image::Handle)>,
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
            platform: None,
            platform_tiles: Vec::new(),
            tiles: Vec::new(),
            background: None,
            flame_time: 0.0,
            flame_last: -1.0,
            flame_frames: Vec::new(),
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
            state.world.platform = self.assets.platform.as_ref().map(|p| p.collider.clone());
        }
        let size = layout.bounds().size();
        if size != state.size && size.width > 0.0 && size.height > 0.0 {
            (state.tiles, state.background) = sky_tiles(&self.sky, size);
            state.platform = self
                .assets
                .platform
                .as_ref()
                .map(|p| raster::platform(p, camera::Camera::new(size.width, size.height)));
            state.platform_tiles = state.platform.as_ref().map(frame_tiles).unwrap_or_default();
            state.flame_frames.clear();
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
                state.flame_time += dt.clamp(0.0, 0.1);
                state.world.advance(dt, size.width, size.height);
                let camera = camera::Camera::new(size.width, size.height);
                let mut poses: Vec<_> = state.world.poses().collect();
                poses.sort_by(|a, b| camera.vector(a.1).2.total_cmp(&camera.vector(b.1).2));
                state.frames.clear();
                if let Some(platform) = &state.platform {
                    for (_, position, _) in &poses {
                        if let Some(shadow) = raster::shadow(*position, camera, platform) {
                            state.frames.push((shadow.rect, shadow.handle()));
                        }
                    }
                }
                if state.flame_time - state.flame_last >= 1.0 / 30.0
                    || state.flame_frames.is_empty()
                {
                    state.flame_frames.clear();
                    if let (Some(asset), Some(platform), Some(sky)) =
                        (&self.assets.platform, &state.platform, &state.background)
                        && let Some(texture) = &asset.flames
                    {
                        for x in [-9.0, 9.0] {
                            let flame = flames::render(
                                texture,
                                math::V(x, 0.06, 0.0),
                                state.flame_time + x * 0.071,
                                camera,
                                platform,
                                sky,
                            );
                            state.flame_frames.push((flame.rect, flame.handle()));
                        }
                    }
                    state.flame_last = state.flame_time;
                }
                state.frames.extend(state.flame_frames.iter().cloned());
                let balls = poses
                    .into_iter()
                    .map(|(kind, position, rotation)| {
                        let frame = raster::ball(
                            &self.assets.balls[kind],
                            position,
                            rotation,
                            camera,
                            state.platform.as_ref(),
                        );
                        (frame.rect, frame.handle())
                    })
                    .collect::<Vec<_>>();
                state.frames.extend(balls);
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
        for (tile, rect) in &state.platform_tiles {
            let rect = Rectangle {
                x: bounds.x + rect.x,
                y: bounds.y + rect.y,
                ..*rect
            };
            renderer.draw_image(image::Image::new(tile.clone()), rect, rect);
        }
        let Some(clip) = bounds.intersection(viewport) else {
            return;
        };
        for (rect, frame) in &state.frames {
            let rect = Rectangle {
                x: bounds.x + rect.x,
                y: bounds.y + rect.y,
                ..*rect
            };
            if let Some(visible) = rect.intersection(&clip) {
                // Only clip at the content's actual window edges, including the very top.
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
fn sky_tiles(sky: &image::Handle, size: Size) -> (Tiles, Option<bitmap::RgbaImage>) {
    let image::Handle::Rgba {
        width,
        height,
        pixels,
        ..
    } = sky
    else {
        return (Vec::new(), None);
    };
    let Some(source) = bitmap::RgbaImage::from_raw(*width, *height, pixels.to_vec()) else {
        return (Vec::new(), None);
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
    (tiles, Some(scaled))
}

fn frame_tiles(frame: &raster::Frame) -> Vec<(image::Handle, Rectangle)> {
    let Some(pixels) = bitmap::RgbaImage::from_raw(frame.width, frame.height, frame.pixels.clone())
    else {
        return Vec::new();
    };
    let mut tiles = Vec::new();
    for y in (0..frame.height).step_by(192) {
        for x in (0..frame.width).step_by(192) {
            let w = 192.min(frame.width - x);
            let h = 192.min(frame.height - y);
            let data = bitmap::imageops::crop_imm(&pixels, x, y, w, h).to_image();
            let rect = Rectangle {
                x: frame.rect.x + x as f32 / frame.width as f32 * frame.rect.width,
                y: frame.rect.y + y as f32 / frame.height as f32 * frame.rect.height,
                width: w as f32 / frame.width as f32 * frame.rect.width,
                height: h as f32 / frame.height as f32 * frame.rect.height,
            };
            tiles.push((image::Handle::from_rgba(w, h, data.into_raw()), rect));
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
        assert!(
            assets.platform.is_some(),
            "original platform and its three materials"
        );
        for (i, asset) in assets.balls.iter().enumerate() {
            let start = Instant::now();
            let mut rotation = math::Q::default();
            let mut last = None;
            for _ in 0..60 {
                rotation.advance(math::V(0.4, 0.6, 0.3), 1.0 / 60.0);
                last = Some(
                    raster::ball(
                        asset,
                        math::V(0.0, 4.0, 0.0),
                        rotation,
                        camera::Camera::new(1040.0, 700.0),
                        None,
                    )
                    .handle(),
                );
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
                        > 1000
                );
                if let Ok(folder) = std::env::var("ROCKOON_BALL_RENDER_DIR") {
                    std::fs::write(Path::new(&folder).join(format!("ball-{i}.rgba")), &pixels)
                        .unwrap();
                    assert!(width > 32 && height > 32);
                }
            } else {
                panic!("raw sprite");
            }
        }
    }
    #[test]
    fn optional_original_platform_long_run() {
        let Ok(root) = std::env::var("ROCKOON_BALL_ASSET_TEST_ROOT") else {
            return;
        };
        let assets = Assets::load(Path::new(&root)).unwrap();
        let mut world = physics::World::new(42);
        world.platform = Some(assets.platform.as_ref().unwrap().collider.clone());
        let mut supported = false;
        let mut depth_motion = false;
        for _ in 0..6600 {
            world.advance(physics::STEP, 1040.0, 700.0);
            supported |= world
                .bodies
                .iter()
                .any(|b| b.position.1 > 1.9 && b.position.1 < 2.1 && b.velocity.1.abs() < 0.2);
            depth_motion |= world.bodies.iter().any(|b| b.position.2.abs() > 3.0);
            assert!(world.bodies.len() <= 12);
            assert!(
                world
                    .bodies
                    .iter()
                    .all(|b| b.position.length().is_finite() && b.velocity.length() < 100.0)
            );
        }
        assert!(supported && depth_motion);
        assert!(
            world.serial > 45,
            "emitter stalled at {} births",
            world.serial
        );
        eprintln!(
            "100 second original platform run: {} live",
            world.bodies.len()
        );
    }
}
