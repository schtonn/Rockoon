//! Small CPU triangle rasterizer: original geometry, normals and repeat-addressed UVs.
use super::{
    BallAsset,
    math::{Q, V},
};
use iced::widget::image;
pub const SIDE: usize = 96;
fn edge(a: V, b: V, x: f32, y: f32) -> f32 {
    (b.0 - a.0) * (y - a.1) - (b.1 - a.1) * (x - a.0)
}
fn sample(texture: &bitmap::RgbaImage, u: f32, v: f32) -> [f32; 3] {
    let w = texture.width() as i32;
    let h = texture.height() as i32;
    let x = u.rem_euclid(1.0) * w as f32 - 0.5;
    let y = v.rem_euclid(1.0) * h as f32 - 0.5;
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let fx = x - ix as f32;
    let fy = y - iy as f32;
    let mut rgb = [0.0; 3];
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
        for i in 0..3 {
            rgb[i] += c[i] as f32 * weight;
        }
    }
    rgb
}
pub fn render(asset: &BallAsset, rotation: Q) -> image::Handle {
    let mesh = &asset.mesh;
    let scale = SIDE as f32 / 4.5;
    let center = SIDE as f32 * 0.5;
    let positions: Vec<_> = mesh
        .positions
        .iter()
        .map(|&v| {
            let p = rotation.rotate(v);
            V(center + p.0 * scale, center - p.1 * scale, p.2)
        })
        .collect();
    let normals: Vec<_> = mesh.normals.iter().map(|&n| rotation.rotate(n)).collect();
    let mut pixels = vec![0u8; SIDE * SIDE * 4];
    let mut depth = vec![f32::NEG_INFINITY; SIDE * SIDE];
    let light = V(-0.38, 0.58, 0.72).unit();
    for face in &mesh.faces {
        let [ia, ib, ic] = *face;
        let [a, b, c] = [positions[ia], positions[ib], positions[ic]];
        let area = edge(a, b, c.0, c.1);
        if area.abs() < 1e-6 {
            continue;
        }
        let x0 = a.0.min(b.0).min(c.0).floor().max(0.0) as usize;
        let x1 = a.0.max(b.0).max(c.0).ceil().clamp(0.0, SIDE as f32) as usize;
        let y0 = a.1.min(b.1).min(c.1).floor().max(0.0) as usize;
        let y1 = a.1.max(b.1).max(c.1).ceil().clamp(0.0, SIDE as f32) as usize;
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
                let z = wa * a.2 + wb * b.2 + wc * c.2;
                let index = y * SIDE + x;
                if z <= depth[index] {
                    continue;
                }
                depth[index] = z;
                let normal = (normals[ia] * wa + normals[ib] * wb + normals[ic] * wc).unit();
                let brightness = 0.48 + 0.52 * normal.dot(light).max(0.0);
                let u = mesh.uv[ia][0] * wa + mesh.uv[ib][0] * wb + mesh.uv[ic][0] * wc;
                let v = mesh.uv[ia][1] * wa + mesh.uv[ib][1] * wb + mesh.uv[ic][1] * wc;
                let color = sample(&asset.texture, u, v);
                for i in 0..3 {
                    pixels[index * 4 + i] = (color[i] * brightness).clamp(0.0, 255.0) as u8;
                }
                pixels[index * 4 + 3] = 255;
            }
        }
    }
    image::Handle::from_rgba(SIDE as u32, SIDE as u32, pixels)
}
