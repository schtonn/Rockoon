mod viewer;
use crate::workspace::error;
use std::{collections::BTreeMap, path::Path};
pub use viewer::view;
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
    pub fn render(&self, yaw: f32, pitch: f32, full: bool) -> iced::widget::image::Handle {
        let (width, height) = if full { (768, 384) } else { (512, 256) };
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

/// Keep one render in flight and only the newest requested view behind it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub epoch: u64,
    pub yaw: f32,
    pub pitch: f32,
    pub full: bool,
}
#[derive(Default)]
pub struct RenderQueue {
    pub active: Option<Frame>,
    latest: Option<Frame>,
}
impl RenderQueue {
    pub fn request(&mut self, frame: Frame) -> Option<Frame> {
        if self.active.is_some() {
            self.latest = Some(frame);
            None
        } else {
            self.active = Some(frame);
            Some(frame)
        }
    }
    pub fn complete(&mut self, frame: Frame) -> Option<Frame> {
        if self.active != Some(frame) {
            return None;
        }
        self.active = self.latest.take();
        self.active
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dragging_coalesces_to_latest_angle_and_preserves_final_full_resolution() {
        let mut queue = RenderQueue::default();
        let first = Frame {
            epoch: 1,
            yaw: 0.0,
            pitch: 0.0,
            full: false,
        };
        assert_eq!(queue.request(first), Some(first));
        for yaw in 1..100 {
            assert!(
                queue
                    .request(Frame {
                        yaw: yaw as f32,
                        ..first
                    })
                    .is_none()
            );
        }
        let final_frame = Frame {
            yaw: 100.0,
            full: true,
            ..first
        };
        assert!(queue.request(final_frame).is_none());
        assert_eq!(queue.complete(first), Some(final_frame));
        assert!(queue.complete(final_frame).is_none());
        assert!(queue.active.is_none());
    }
    #[test]
    fn an_old_preview_cannot_complete_the_new_preview_render() {
        let mut queue = RenderQueue::default();
        let frame = Frame {
            epoch: 2,
            yaw: 0.0,
            pitch: 0.0,
            full: true,
        };
        queue.request(frame);
        assert!(queue.complete(Frame { epoch: 1, ..frame }).is_none());
        assert_eq!(queue.active, Some(frame));
    }
}
