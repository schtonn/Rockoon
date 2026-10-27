use iced::widget::image;
use std::path::Path;

/// All artwork comes from the selected installation. No replacement drawings are embedded.
#[derive(Clone, Debug, Default)]
pub struct GameAssets {
    pub sky: Option<image::Handle>,
    pub selected_button: Option<image::Handle>,
    pub normal_button: Option<image::Handle>,
    pub start: Option<image::Handle>,
}

impl GameAssets {
    pub fn load(root: &Path) -> Self {
        let textures = root.join("Textures");
        Self {
            start: start_caption(&textures.join("Font_1.tga")),
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

/// The game's bitmap font is a 16 by 16 ASCII atlas with 32px cells.
/// Trim transparent side bearings and keep each glyph's original pixels and baseline.
fn start_caption(path: &Path) -> Option<image::Handle> {
    let atlas = bitmap::ImageReader::open(path)
        .ok()?
        .decode()
        .ok()?
        .into_rgba8();
    if atlas.dimensions() != (512, 512) {
        return None;
    }
    let mut glyphs = Vec::new();
    for code in b"START" {
        let x = u32::from(code % 16) * 32;
        let y = u32::from(code / 16) * 32;
        let columns: Vec<_> = (0..32)
            .filter(|col| (0..32).any(|row| atlas.get_pixel(x + col, y + row)[3] != 0))
            .collect();
        let left = *columns.first()?;
        let right = *columns.last()? + 1;
        glyphs.push(bitmap::imageops::crop_imm(&atlas, x + left, y, right - left, 32).to_image());
    }
    let width = glyphs.iter().map(bitmap::RgbaImage::width).sum::<u32>() + 8;
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
    Some(image::Handle::from_rgba(width, height, pixels.into_raw()))
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
