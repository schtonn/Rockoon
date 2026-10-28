use iced::widget::image;
use std::{collections::HashMap, path::Path};

/// All artwork comes from the selected installation. No replacement drawings are embedded.
#[derive(Clone, Debug, Default)]
pub struct GameAssets {
    pub sky: Option<image::Handle>,
    pub menu_ball: Option<image::Handle>,
    pub balls: Option<std::sync::Arc<crate::home_balls::Assets>>,
    pub selected_button: Option<image::Handle>,
    pub normal_button: Option<image::Handle>,
    pub captions: HashMap<&'static str, Caption>,
}

impl GameAssets {
    pub fn load(root: &Path) -> Self {
        let textures = root.join("Textures");
        let balls = crate::home_balls::Assets::load(root);
        let menu_ball = balls.as_ref().map(|b| b.menu_icon());
        Self {
            menu_ball,
            captions: captions(&textures.join("Font_1.tga")),
            balls,
            sky: decode(&textures.join("Sky/Sky_A_Front.bmp"), None),
            selected_button: decode(&textures.join("Button01_select.tga"), Some((256, 256))),
            normal_button: decode(&textures.join("Button01_deselect.tga"), Some((256, 256))),
        }
    }

    pub fn launch_button(&self, hovered: bool) -> Option<&image::Handle> {
        if hovered {
            self.selected_button
                .as_ref()
                .or(self.normal_button.as_ref())
        } else {
            self.normal_button
                .as_ref()
                .or(self.selected_button.as_ref())
        }
    }
}

#[derive(Clone, Debug)]
pub struct Caption {
    pub handle: image::Handle,
    pub width: u32,
    pub height: u32,
}

/// Original menu labels use initial caps with the atlas's smaller capital-shaped lowercase glyphs.
fn captions(path: &Path) -> HashMap<&'static str, Caption> {
    let Some(atlas) = bitmap::ImageReader::open(path)
        .ok()
        .and_then(|reader| reader.decode().ok())
        .map(|image| image.into_rgba8())
    else {
        return HashMap::new();
    };
    if atlas.dimensions() != (512, 512) {
        return HashMap::new();
    }
    ["Start", "Stop", "Starting...", "Browse"]
        .into_iter()
        .filter_map(|label| bitmap_caption(&atlas, label).map(|caption| (label, caption)))
        .collect()
}

/// Keep the font's source strokes, alpha and shared baseline, trimming transparent side bearings.
fn bitmap_caption(atlas: &bitmap::RgbaImage, label: &str) -> Option<Caption> {
    let mut glyphs = Vec::new();
    for code in label.bytes() {
        let x = u32::from(code % 16) * 32;
        let y = u32::from(code / 16) * 32;
        let columns: Vec<_> = (0..32)
            .filter(|col| (0..32).any(|row| atlas.get_pixel(x + col, y + row)[3] != 0))
            .collect();
        let left = *columns.first()?;
        let right = *columns.last()? + 1;
        glyphs.push(bitmap::imageops::crop_imm(atlas, x + left, y, right - left, 32).to_image());
    }
    let width = glyphs.iter().map(bitmap::RgbaImage::width).sum::<u32>()
        + 2 * glyphs.len().saturating_sub(1) as u32;
    let mut caption = bitmap::RgbaImage::new(width, 32);
    let mut x = 0;
    for glyph in glyphs {
        bitmap::imageops::replace(&mut caption, &glyph, x, 0);
        x += i64::from(glyph.width()) + 2;
    }
    let rows: Vec<_> = (0..32)
        .filter(|y| (0..width).any(|x| caption.get_pixel(x, *y)[3] != 0))
        .collect();
    let top = *rows.first()?;
    let height = *rows.last()? + 1 - top;
    let pixels = bitmap::imageops::crop_imm(&caption, 0, top, width, height).to_image();
    Some(Caption {
        handle: image::Handle::from_rgba(width, height, pixels.into_raw()),
        width,
        height,
    })
}

/// Keep source pixels unchanged, extracting the button's exact atlas rectangle before upload.
/// This also avoids relying on image clipping in the software renderer.
fn decode(path: &Path, expected_size: Option<(u32, u32)>) -> Option<image::Handle> {
    let decoded = bitmap::ImageReader::open(path).ok()?.decode().ok()?;
    let (width, height) = (decoded.width(), decoded.height());
    if expected_size.is_some_and(|size| size != (width, height)) {
        return None;
    }
    let pixels = if expected_size.is_some() {
        decoded.crop_imm(0, 0, 256, 64)
    } else {
        decoded
    };
    Some(image::Handle::from_rgba(
        pixels.width(),
        pixels.height(),
        pixels.into_rgba8().into_raw(),
    ))
}

pub fn preview(path: &Path) -> Result<image::Handle, String> {
    let reader = bitmap::ImageReader::open(path).map_err(|e| e.to_string())?;
    let mut limits = bitmap::Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    limits.max_alloc = Some(128 * 1024 * 1024);
    let mut reader = reader;
    reader.limits(limits);
    let pixels = reader.decode().map_err(|e| e.to_string())?.into_rgba8();
    Ok(image::Handle::from_rgba(
        pixels.width(),
        pixels.height(),
        pixels.into_raw(),
    ))
}
