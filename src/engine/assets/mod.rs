use std::ops::Deref;

use egui::ColorImage;
use wgpu::*;
use wgpu_glyph::ab_glyph::FontArc;

pub use manager::*;
pub use progress::*;

pub mod progress;
pub mod manager;


#[derive(Debug)]
pub struct TextureWrapper {
    pub texture: Texture,
    pub view: TextureView,
    pub sampler: Sampler,
    pub info: TextureInfo,
}

#[derive(Default, Debug)]
pub struct TextureInfo {
    pub width: u32,
    pub height: u32,
}

impl TextureInfo {
    pub(crate) fn new(width: u32, height: u32) -> TextureInfo {
        Self { width, height }
    }
}


#[repr(transparent)]
#[derive(Clone)]
pub struct FontWrapper(pub FontArc);

impl Deref for FontWrapper {
    type Target = FontArc;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&FontArc> for FontWrapper {
    fn from(f: &FontArc) -> Self {
        Self {
            0: f.clone()
        }
    }
}

pub fn load_image_from_memory(image_data: &[u8]) -> Result<ColorImage, image::ImageError> {
    let image = image::load_from_memory(image_data)?;
    let size = [image.width() as _, image.height() as _];
    let image_buffer = image.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    Ok(ColorImage::from_rgba_unmultiplied(
        size,
        pixels.as_slice(),
    ))
}
