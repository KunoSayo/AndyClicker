use std::ops::Deref;

use wgpu::*;
use wgpu_glyph::ab_glyph::FontArc;

pub use manager::*;
pub use progress::*;

pub mod progress;
pub mod manager;

pub type TexHandle = u32;

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

