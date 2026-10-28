//! Original checkpoint particles: 20 ms emission, 1 +/- .25 s life, 8 +/- 3 units/s,
//! size 3 -> .1, RGBA (.9215687,.1647059,.2745098,1) -> zero, SRCALPHA + ONE.
use super::{camera::Camera, math::V, raster::Frame};
use iced::Rectangle;
fn random(index: u32, salt: u32) -> f32 {
    let mut x = index.wrapping_mul(747796405).wrapping_add(salt);
    x ^= x >> 16;
    x = x.wrapping_mul(2246822519);
    (x >> 8) as f32 / 16777216.0
}
pub fn render(
    texture: &bitmap::RgbaImage,
    emitter: V,
    time: f32,
    camera: Camera,
    platform: &Frame,
    sky: &bitmap::RgbaImage,
) -> Frame {
    let corners = [
        V(-2.8, 0.0, -2.8),
        V(2.8, 0.0, 2.8),
        V(-2.8, 15.0, -2.8),
        V(2.8, 15.0, 2.8),
    ]
    .map(|p| camera.project(emitter + p));
    let x = corners
        .iter()
        .map(|p| p.0)
        .fold(f32::INFINITY, f32::min)
        .floor()
        - 2.0;
    let y = corners
        .iter()
        .map(|p| p.1)
        .fold(f32::INFINITY, f32::min)
        .floor()
        - 2.0;
    let right = corners
        .iter()
        .map(|p| p.0)
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil()
        + 2.0;
    let bottom = corners
        .iter()
        .map(|p| p.1)
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil()
        + 2.0;
    let rect = Rectangle {
        x,
        y,
        width: (right - x).max(1.0),
        height: (bottom - y).max(1.0),
    };
    let width = rect.width as u32;
    let height = rect.height as u32;
    let mut light = vec![[0.0_f32; 3]; (width * height) as usize];
    for i in 0..50 {
        let life = 0.75 + random(i, 41) * 0.5;
        let age = (time + i as f32 * 0.02).rem_euclid(life);
        let phase = age / life;
        let speed = (5.0 + random(i, 97) * 6.0) * 1.2;
        let position = emitter
            + V(
                (random(i, 23) - 0.5) * age * 1.4,
                age * speed,
                (random(i, 61) - 0.5) * age * 1.4,
            );
        let point = camera.project(position);
        let side =
            (3.0 * (1.0 - phase) + 0.1 * phase) * 1.2 * camera.scale * camera.factor(point.2);
        let left = point.0 - rect.x - side * 0.5;
        let top = point.1 - rect.y - side * 0.5;
        let fade = (1.0 - phase).powi(2);
        for y in
            (top.floor().max(0.0) as u32)..((top + side).ceil().clamp(0.0, height as f32) as u32)
        {
            for x in (left.floor().max(0.0) as u32)
                ..((left + side).ceil().clamp(0.0, width as f32) as u32)
            {
                if point.2
                    < platform.depth_at(rect.x + x as f32 + 0.5, rect.y + y as f32 + 0.5) - 0.02
                {
                    continue;
                }
                let u = ((x as f32 + 0.5 - left) / side * texture.width() as f32)
                    .clamp(0.0, texture.width() as f32 - 1.0) as u32;
                let v = ((y as f32 + 0.5 - top) / side * texture.height() as f32)
                    .clamp(0.0, texture.height() as f32 - 1.0) as u32;
                let source = texture.get_pixel(u, v);
                let pixel = &mut light[(y * width + x) as usize];
                for c in 0..3 {
                    pixel[c] += source[c] as f32 * [0.9215687, 0.1647059, 0.2745098][c] * fade;
                }
            }
        }
    }
    let mut pixels = vec![0; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let index = (y * width + x) as usize;
            let light = light[index];
            if light.iter().all(|v| *v < 0.2) {
                continue;
            }
            let sx = rect.x + x as f32 + 0.5;
            let sy = rect.y + y as f32 + 0.5;
            let sky_pixel = sky.get_pixel(
                (sx / camera.width * sky.width() as f32).clamp(0.0, sky.width() as f32 - 1.0)
                    as u32,
                (sy / camera.height * sky.height() as f32).clamp(0.0, sky.height() as f32 - 1.0)
                    as u32,
            );
            let floor = platform.color_at(sx, sy);
            let base: [f32; 3] = std::array::from_fn(|i| {
                floor[i] * (floor[3] / 255.0) + sky_pixel[i] as f32 * (1.0 - floor[3] / 255.0)
            });
            let target: [f32; 3] = std::array::from_fn(|i| (base[i] + light[i]).min(255.0));
            // Encode the original additive result as an equivalent alpha-over sprite for iced.
            // Fully transparent outside emitted light; no opaque rectangle wipes nearby objects.
            let alpha = (0..3)
                .map(|i| (target[i] - base[i]) / (255.0 - base[i]).max(0.001))
                .fold(0.0_f32, f32::max)
                .clamp(0.0, 1.0);
            if alpha < 0.001 {
                continue;
            }
            for i in 0..3 {
                pixels[index * 4 + i] =
                    ((target[i] - base[i] * (1.0 - alpha)) / alpha).clamp(0.0, 255.0) as u8;
            }
            pixels[index * 4 + 3] = (alpha * 255.0) as u8;
        }
    }
    Frame {
        rect,
        width,
        height,
        pixels,
        depth: Vec::new(),
    }
}
