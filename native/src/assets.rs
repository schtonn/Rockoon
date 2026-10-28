use iced::widget::image;
use std::{collections::HashMap, path::Path};

/// All artwork comes from the selected installation. No replacement drawings are embedded.
#[derive(Clone, Debug, Default)]
pub struct GameAssets {
    pub sky: Option<image::Handle>,
    pub backgrounds: Vec<Background>,
    pub balls: Option<std::sync::Arc<crate::home_balls::Assets>>,
    pub selected_button: Option<image::Handle>,
    pub normal_button: Option<image::Handle>,
    pub captions: HashMap<&'static str, Caption>,
}

impl GameAssets {
    pub fn load(root: &Path) -> Self {
        let textures = root.join("Textures");
        let balls = crate::home_balls::Assets::load(root);
        Self {
            captions: captions(&textures.join("Font_1.tga")),
            balls,
            sky: decode(&textures.join("Sky/Sky_A_Front.bmp"), None),
            backgrounds: backgrounds(&textures.join("Sky")),
            selected_button: decode(&textures.join("Button01_select.tga"), Some((256, 256))),
            normal_button: decode(&textures.join("Button01_deselect.tga"), Some((256, 256))),
        }
    }

    pub fn background(&self, letter: &str) -> Option<&image::Handle> {
        self.backgrounds
            .iter()
            .find(|b| b.letter == letter)
            .map(|b| &b.image)
            .or(self.sky.as_ref())
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

#[derive(Clone, Debug)]
pub struct Background {
    pub letter: String,
    pub name: String,
    pub image: image::Handle,
    pub thumbnail: image::Handle,
}
fn backgrounds(dir: &Path) -> Vec<Background> {
    let mut result = Vec::new();
    for (letter, name) in [
        ("L", "第 1 关"),
        ("E", "第 2 关"),
        ("A", "第 3 关"),
        ("F", "第 4 / 13 关"),
        ("C", "第 5 关"),
        ("H", "第 6 关"),
        ("D", "第 7 关"),
        ("G", "第 8 关"),
        ("K", "第 9 关"),
        ("B", "第 10 关"),
        ("J", "第 11 关"),
        ("I", "第 12 关"),
        ("X", "自制地图"),
    ] {
        let path = dir.join(format!("Sky_{letter}_Front.bmp"));
        if let (Ok(image), Ok(thumbnail)) = (preview(&path), thumbnail(&path)) {
            result.push(Background {
                letter: letter.into(),
                name: name.into(),
                image,
                thumbnail,
            });
        }
    }
    result
}
pub fn thumbnail(path: &Path) -> Result<image::Handle, String> {
    let mut reader = bitmap::ImageReader::open(path).map_err(|e| e.to_string())?;
    let mut limits = bitmap::Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let pixels = reader
        .decode()
        .map_err(|e| e.to_string())?
        .thumbnail(480, 256)
        .into_rgba8();
    Ok(image::Handle::from_rgba(
        pixels.width(),
        pixels.height(),
        pixels.into_raw(),
    ))
}
pub fn local_thumbnail(root: &Path, entry: &crate::workspace::Entry) -> Option<image::Handle> {
    if entry.directory {
        return None;
    }
    let source = if matches!(
        entry.kind,
        crate::workspace::Kind::Skies | crate::workspace::Kind::Textures
    ) {
        Some(entry.path.clone())
    } else {
        ["png", "jpg", "jpeg", "webp"]
            .into_iter()
            .map(|ext| entry.path.with_extension(ext))
            .find(|p| p.is_file())
    }?;
    let source = crate::workspace::checked(root, &source).ok()?;
    thumbnail(&source).ok()
}

pub type Thumbnails = Vec<(std::path::PathBuf, Option<image::Handle>)>;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn backgrounds_share_the_game_files_and_skip_missing_skies() {
        let root = tempfile::tempdir().unwrap();
        for letter in ["A", "L"] {
            bitmap::RgbaImage::from_pixel(8, 4, bitmap::Rgba([10, 20, 30, 255]))
                .save(root.path().join(format!("Sky_{letter}_Front.bmp")))
                .unwrap();
        }
        let choices = backgrounds(root.path());
        assert_eq!(choices.len(), 2);
        assert_eq!(choices[0].letter, "L");
        assert_eq!(choices[1].letter, "A");
        let visuals = GameAssets {
            backgrounds: choices,
            ..Default::default()
        };
        assert!(visuals.background("L").is_some());
        assert!(visuals.background("missing").is_none());
    }
}
