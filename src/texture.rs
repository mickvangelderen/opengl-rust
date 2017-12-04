extern crate gl;
extern crate core;

use core::nonzero::NonZero;
use gl::types::*;

#[allow(unused)]
#[repr(u32)]
pub enum TextureTarget {
    Texture1D = gl::TEXTURE_1D,
    Texture2D = gl::TEXTURE_2D,
    Texture3D = gl::TEXTURE_3D,
    Texture1DArray = gl::TEXTURE_1D_ARRAY,
    Texture2DArray = gl::TEXTURE_2D_ARRAY,
    TextureRectangle = gl::TEXTURE_RECTANGLE,
    TextureCubeMap = gl::TEXTURE_CUBE_MAP,
    TextureCubeMapArray = gl::TEXTURE_CUBE_MAP_ARRAY,
    TextureBuffer = gl::TEXTURE_BUFFER,
    Texture2DMultisample = gl::TEXTURE_2D_MULTISAMPLE,
    Texture2DMultisampleArray = gl::TEXTURE_2D_MULTISAMPLE_ARRAY,
}

#[derive(Debug)]
pub struct TextureId(NonZero<GLuint>);

impl TextureId {
    pub unsafe fn as_uint(&self) -> GLuint {
        (self.0).get()
    }

    pub fn new() -> Option<Self> {
        NonZero::new(unsafe {
            let mut ids: [GLuint; 1] = [0];
            gl::GenTextures(ids.len() as GLsizei, ids.as_mut_ptr());
            ids[0]
        }).map(TextureId)
    }

    pub fn bind(&self, target: TextureTarget) {
        unsafe {
            gl::BindTexture(target as GLenum, self.as_uint());
        }
    }
}

impl Drop for TextureId {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.as_uint());
        }
    }
}
