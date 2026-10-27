use iced::widget::image;
use std::path::Path;

/// All artwork comes from the selected installation. No replacement drawings are embedded.
#[derive(Clone, Debug, Default)]
pub struct GameAssets {
    pub sky: Option<image::Handle>,
    pub selected_button: Option<image::Handle>,
    pub normal_button: Option<image::Handle>,
}

impl GameAssets {
    pub fn load(root: &Path) -> Self {
        let textures = root.join("Textures");
        Self {
            sky: decode(&textures.join("Sky/Sky_A_Front.bmp"), None),
            selected_button: decode(&textures.join("Button01_select.tga"), Some((256, 256))),
            normal_button: decode(&textures.join("Button01_deselect.tga"), Some((256, 256))),
        }
    }

    pub fn launch_button(&self) -> Option<&image::Handle> {
        self.selected_button
            .as_ref()
            .or(self.normal_button.as_ref())
    }
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
