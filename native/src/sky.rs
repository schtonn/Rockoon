use crate::workspace::error;
use std::{collections::BTreeMap, path::Path};
#[derive(Debug)]
pub struct Skybox {
    faces: BTreeMap<String, bitmap::RgbaImage>,
}
impl Skybox {
    pub fn load(path: &Path) -> Result<Self, String> {
        let name = path
            .file_stem()
            .ok_or("天空盒文件名无效")?
            .to_string_lossy();
        let (prefix, _) = name
            .rsplit_once('_')
            .ok_or("天空盒文件名应为 Sky_A_Front.bmp")?;
        if !prefix.to_lowercase().starts_with("sky_") {
            return Err("天空盒文件名应为 Sky_A_Front.bmp".into());
        }
        let folder = path.parent().ok_or("天空盒缺少目录")?;
        let mut faces = BTreeMap::new();
        for side in ["Front", "Back", "Left", "Right", "Down"] {
            let wanted = format!("{prefix}_{side}.bmp");
            let actual = std::fs::read_dir(folder)
                .map_err(error)?
                .filter_map(Result::ok)
                .find(|e| {
                    e.file_name()
                        .to_string_lossy()
                        .eq_ignore_ascii_case(&wanted)
                });
            if let Some(file) = actual {
                let mut reader = bitmap::ImageReader::open(file.path()).map_err(error)?;
                let mut limits = bitmap::Limits::default();
                limits.max_image_width = Some(4096);
                limits.max_image_height = Some(4096);
                limits.max_alloc = Some(64 * 1024 * 1024);
                reader.limits(limits);
                faces.insert(side.into(), reader.decode().map_err(error)?.into_rgba8());
            }
        }
        if faces.is_empty() {
            return Err("没有可读取的天空盒图片".into());
        }
        Ok(Self { faces })
    }
    pub fn render(&self, yaw: f32, pitch: f32) -> iced::widget::image::Handle {
        let (width, height) = (768, 384);
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);
        let (sy, cy) = yaw.to_radians().sin_cos();
        let (sp, cp) = pitch.to_radians().sin_cos();
        for py in 0..height {
            for px in 0..width {
                let x =
                    (2.0 * (px as f32 + 0.5) / width as f32 - 1.0) * width as f32 / height as f32;
                let y = 1.0 - 2.0 * (py as f32 + 0.5) / height as f32;
                let (y, z) = (y * cp - sp, cp + y * sp);
                let (x, z) = (x * cy + z * sy, z * cy - x * sy);
                let (ax, ay, az) = (x.abs(), y.abs(), z.abs());
                let (side, u, v) = if ay > ax && ay > az {
                    if y > 0.0 {
                        ("Up", 0.0, 0.0)
                    } else {
                        ("Down", x / ay, z / ay)
                    }
                } else if ax > az {
                    if x > 0.0 {
                        ("Right", -z / ax, -y / ax)
                    } else {
                        ("Left", z / ax, -y / ax)
                    }
                } else if z > 0.0 {
                    ("Front", x / az, -y / az)
                } else {
                    ("Back", -x / az, -y / az)
                };
                if let Some(face) = self.faces.get(side) {
                    let fx = ((u + 1.0) * 0.5 * face.width() as f32)
                        .clamp(0.0, face.width().saturating_sub(1) as f32)
                        as u32;
                    let fy = ((v + 1.0) * 0.5 * face.height() as f32)
                        .clamp(0.0, face.height().saturating_sub(1) as f32)
                        as u32;
                    pixels.extend_from_slice(&face.get_pixel(fx, fy).0);
                } else {
                    pixels.extend_from_slice(&[29, 30, 31, 255]);
                }
            }
        }
        iced::widget::image::Handle::from_rgba(width, height, pixels)
    }
}
