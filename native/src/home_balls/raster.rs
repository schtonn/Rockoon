//! Perspective-correct CPU rendering with original mesh materials and depth occlusion.
use super::{
    BallAsset, PlatformAsset,
    camera::Camera,
    math::{Q, V},
    mesh::Mesh,
};
use iced::{Rectangle, widget::image};
const RESOLUTION: f32 = 1.5;
pub struct Frame {
    pub rect: Rectangle,
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub depth: Vec<f32>,
}
impl Frame {
    pub fn handle(&self) -> image::Handle {
        image::Handle::from_rgba(self.width, self.height, self.pixels.clone())
    }
    pub(super) fn depth_at(&self, x: f32, y: f32) -> f32 {
        if !self.rect.contains(iced::Point::new(x, y)) {
            return f32::NEG_INFINITY;
        }
        let x = ((x - self.rect.x) / self.rect.width * self.width as f32) as u32;
        let y = ((y - self.rect.y) / self.rect.height * self.height as f32) as u32;
        self.depth[(y.min(self.height - 1) * self.width + x.min(self.width - 1)) as usize]
    }
}
impl Frame {
    pub(super) fn color_at(&self, x: f32, y: f32) -> [f32; 4] {
        if !self.rect.contains(iced::Point::new(x, y)) {
            return [0.0; 4];
        }
        let x = ((x - self.rect.x) / self.rect.width * self.width as f32) as u32;
        let y = ((y - self.rect.y) / self.rect.height * self.height as f32) as u32;
        let index = ((y.min(self.height - 1) * self.width + x.min(self.width - 1)) * 4) as usize;
        std::array::from_fn(|i| self.pixels[index + i] as f32)
    }
}

fn edge(a: V, b: V, x: f32, y: f32) -> f32 {
    (b.0 - a.0) * (y - a.1) - (b.1 - a.1) * (x - a.0)
}
fn sample(texture: &bitmap::RgbaImage, u: f32, v: f32) -> [f32; 4] {
    let w = texture.width() as i32;
    let h = texture.height() as i32;
    let x = u.rem_euclid(1.0) * w as f32 - 0.5;
    let y = v.rem_euclid(1.0) * h as f32 - 0.5;
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let fx = x - ix as f32;
    let fy = y - iy as f32;
    let mut color = [0.0; 4];
    for (dx, dy, weight) in [
        (0, 0, (1.0 - fx) * (1.0 - fy)),
        (1, 0, fx * (1.0 - fy)),
        (0, 1, (1.0 - fx) * fy),
        (1, 1, fx * fy),
    ] {
        let c = texture.get_pixel(
            (ix + dx).rem_euclid(w) as u32,
            (iy + dy).rem_euclid(h) as u32,
        );
        for i in 0..4 {
            color[i] += c[i] as f32 * weight;
        }
    }
    color
}
pub fn platform(asset: &PlatformAsset, camera: Camera) -> Frame {
    let textures: Vec<_> = asset.textures.iter().collect();
    render(
        &asset.mesh,
        &textures,
        V::default(),
        Q::default(),
        camera,
        None,
    )
}
pub fn ball(
    asset: &BallAsset,
    position: V,
    rotation: Q,
    camera: Camera,
    platform: Option<&Frame>,
) -> Frame {
    render(
        &asset.mesh,
        &[&asset.texture],
        position,
        rotation,
        camera,
        platform,
    )
}
fn render(
    mesh: &Mesh,
    textures: &[&bitmap::RgbaImage],
    position: V,
    rotation: Q,
    camera: Camera,
    occluder: Option<&Frame>,
) -> Frame {
    let projected: Vec<_> = mesh
        .positions
        .iter()
        .map(|&p| camera.project(rotation.rotate(p) + position))
        .collect();
    let mut min = V(f32::INFINITY, f32::INFINITY, 0.0);
    let mut max = -min;
    for p in &projected {
        min.0 = min.0.min(p.0);
        min.1 = min.1.min(p.1);
        max.0 = max.0.max(p.0);
        max.1 = max.1.max(p.1);
    }
    let rect = Rectangle {
        x: min.0.floor() - 1.0,
        y: min.1.floor() - 1.0,
        width: (max.0.ceil() - min.0.floor() + 2.0).clamp(1.0, 2048.0),
        height: (max.1.ceil() - min.1.floor() + 2.0).clamp(1.0, 2048.0),
    };
    let width = (rect.width * RESOLUTION).ceil() as u32;
    let height = (rect.height * RESOLUTION).ceil() as u32;
    let positions: Vec<_> = projected
        .iter()
        .map(|p| {
            V(
                (p.0 - rect.x) * RESOLUTION,
                (p.1 - rect.y) * RESOLUTION,
                p.2,
            )
        })
        .collect();
    let normals: Vec<_> = mesh
        .normals
        .iter()
        .map(|&n| camera.vector(rotation.rotate(n)))
        .collect();
    let mut frame = Frame {
        rect,
        width,
        height,
        pixels: vec![0; (width * height * 4) as usize],
        depth: vec![f32::NEG_INFINITY; (width * height) as usize],
    };
    let light = camera.vector(V(-0.38, 0.58, 0.72).unit());
    for (face_index, face) in mesh.faces.iter().enumerate() {
        let [ia, ib, ic] = *face;
        let [a, b, c] = [positions[ia], positions[ib], positions[ic]];
        let area = edge(a, b, c.0, c.1);
        if area.abs() < 1e-6 {
            continue;
        }
        let x0 = a.0.min(b.0).min(c.0).floor().clamp(0.0, width as f32) as u32;
        let x1 = a.0.max(b.0).max(c.0).ceil().clamp(0.0, width as f32) as u32;
        let y0 = a.1.min(b.1).min(c.1).floor().clamp(0.0, height as f32) as u32;
        let y1 = a.1.max(b.1).max(c.1).ceil().clamp(0.0, height as f32) as u32;
        let texture = textures[mesh.materials[face_index]
            .saturating_sub(1)
            .min(textures.len() - 1)];
        for y in y0..y1 {
            for x in x0..x1 {
                let px = x as f32 + 0.5;
                let py = y as f32 + 0.5;
                let wa = edge(b, c, px, py) / area;
                let wb = edge(c, a, px, py) / area;
                let wc = 1.0 - wa - wb;
                if wa < 0.0 || wb < 0.0 || wc < 0.0 {
                    continue;
                }
                let wa = wa * camera.factor(a.2);
                let wb = wb * camera.factor(b.2);
                let wc = wc * camera.factor(c.2);
                let sum = wa + wb + wc;
                let (wa, wb, wc) = (wa / sum, wb / sum, wc / sum);
                let z = wa * a.2 + wb * b.2 + wc * c.2;
                let index = (y * width + x) as usize;
                if z <= frame.depth[index] {
                    continue;
                }
                if occluder.is_some_and(|f| {
                    z < f.depth_at(rect.x + px / RESOLUTION, rect.y + py / RESOLUTION) - 0.01
                }) {
                    continue;
                }
                frame.depth[index] = z;
                let normal = (normals[ia] * wa + normals[ib] * wb + normals[ic] * wc).unit();
                let brightness = 0.48 + 0.52 * normal.dot(light).max(0.0);
                let u = mesh.uv[ia][0] * wa + mesh.uv[ib][0] * wb + mesh.uv[ic][0] * wc;
                let v = mesh.uv[ia][1] * wa + mesh.uv[ib][1] * wb + mesh.uv[ic][1] * wc;
                let color = sample(texture, u, v);
                for (i, c) in color.iter().take(3).enumerate() {
                    frame.pixels[index * 4 + i] = (c * brightness).clamp(0.0, 255.0) as u8;
                }
                frame.pixels[index * 4 + 3] = color[3] as u8;
            }
        }
    }
    frame
}

/// Soft contact shadow projected onto the original platform's actual top surface.
pub fn shadow(position: V, camera: Camera, platform: &Frame) -> Option<Frame> {
    let height = position.1 - 2.0;
    if !(-0.05..20.0).contains(&height) {
        return None;
    }
    let radius = 2.1 + height.max(0.0) * 0.08;
    let corners = [(-1.0, -1.0), (-1.0, 1.0), (1.0, -1.0), (1.0, 1.0)]
        .map(|(x, z)| camera.project(V(position.0 + x * radius, 0.02, position.2 + z * radius)));
    let x = corners
        .iter()
        .map(|p| p.0)
        .fold(f32::INFINITY, f32::min)
        .floor();
    let y = corners
        .iter()
        .map(|p| p.1)
        .fold(f32::INFINITY, f32::min)
        .floor();
    let right = corners
        .iter()
        .map(|p| p.0)
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil();
    let bottom = corners
        .iter()
        .map(|p| p.1)
        .fold(f32::NEG_INFINITY, f32::max)
        .ceil();
    let rect = Rectangle {
        x,
        y,
        width: (right - x).max(1.0),
        height: (bottom - y).max(1.0),
    };
    let width = (rect.width * RESOLUTION).ceil() as u32;
    let height_pixels = (rect.height * RESOLUTION).ceil() as u32;
    let mut frame = Frame {
        rect,
        width,
        height: height_pixels,
        pixels: vec![0; (width * height_pixels * 4) as usize],
        depth: Vec::new(),
    };
    for y in 0..height_pixels {
        for x in 0..width {
            let sx = rect.x + (x as f32 + 0.5) / RESOLUTION;
            let sy = rect.y + (y as f32 + 0.5) / RESOLUTION;
            let ground = camera.ground(sx, sy);
            let d = ((ground.0 - position.0).powi(2) + (ground.2 - position.2).powi(2))
                / (radius * radius);
            if d >= 1.0 || (camera.vector(ground).2 - platform.depth_at(sx, sy)).abs() > 0.12 {
                continue;
            }
            frame.pixels[((y * width + x) * 4 + 3) as usize] =
                ((1.0 - d).powi(2) * 64.0 * (-height.max(0.0) / 8.0).exp()) as u8;
        }
    }
    Some(frame)
}
