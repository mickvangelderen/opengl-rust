extern crate core;
extern crate gl;

use core::nonzero::NonZero;
use gl::types::*;

pub struct TextureTarget(u32);

impl TextureTarget {
    pub fn texture_2d() -> Self {
        TextureTarget(gl::TEXTURE_2D)
    }

    pub fn as_enum(&self) -> GLenum {
        self.0
    }
}

// #[allow(unused)]
// #[repr(u32)]
// pub enum TextureTarget {
//     Texture1D = gl::TEXTURE_1D,
//     Texture2D = gl::TEXTURE_2D,
//     Texture3D = gl::TEXTURE_3D,
//     Texture1DArray = gl::TEXTURE_1D_ARRAY,
//     Texture2DArray = gl::TEXTURE_2D_ARRAY,
//     TextureRectangle = gl::TEXTURE_RECTANGLE,
//     TextureCubeMap = gl::TEXTURE_CUBE_MAP,
//     TextureCubeMapArray = gl::TEXTURE_CUBE_MAP_ARRAY,
//     TextureBuffer = gl::TEXTURE_BUFFER,
//     Texture2DMultisample = gl::TEXTURE_2D_MULTISAMPLE,
//     Texture2DMultisampleArray = gl::TEXTURE_2D_MULTISAMPLE_ARRAY,
// }

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

    pub fn bind<'a>(&'a self, target: &'a mut TextureTarget) -> BoundTextureId {
        unsafe {
            gl::BindTexture(target.as_enum(), self.as_uint());
        }
        BoundTextureId {
            id: PhantomData,
            target,
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

use std::marker::PhantomData;

pub struct BoundTextureId<'a> {
    id: PhantomData<&'a TextureId>,
    target: &'a mut TextureTarget,
}

impl<'a> BoundTextureId<'a> {
    fn parameter_i(&mut self, param: GLenum, value: GLint) -> &mut Self {
        unsafe {
            gl::TexParameteri(self.target.as_enum(), param, value);
        }
        self
    }

    pub fn min_filter(&mut self, value: GLint) -> &mut Self {
        self.parameter_i(gl::TEXTURE_MIN_FILTER, value)
    }

    pub fn mag_filter(&mut self, value: GLint) -> &mut Self {
        self.parameter_i(gl::TEXTURE_MAG_FILTER, value)
    }

    pub fn wrap_s(&mut self, value: GLint) -> &mut Self {
        self.parameter_i(gl::TEXTURE_WRAP_S, value)
    }

    // TODO(mickvangelderen): Only define this for 2D+ textures.
    pub fn wrap_t(&mut self, value: GLint) -> &mut Self {
        self.parameter_i(gl::TEXTURE_WRAP_T, value)
    }

    // TODO(mickvangelderen): Only define this for 3D+ textures.
    pub fn wrap_r(&mut self, value: GLint) -> &mut Self {
        self.parameter_i(gl::TEXTURE_WRAP_R, value)
    }

    // TODO(mickvangelderen): Enums
    pub unsafe fn image_2d(
        &mut self,
        mipmap_level: GLint,
        internal_format: GLint,
        width: GLint,
        height: GLint,
        format: GLenum,
        component_format: GLenum,
        data: *const GLvoid,
    ) -> &mut Self {
        gl::TexImage2D(
            self.target.as_enum(),
            mipmap_level,
            internal_format,
            width,
            height,
            0, // border, must be zero
            format,
            component_format,
            data,
        );
        self
    }

    pub fn generate_mipmap(&mut self) -> &mut Self {
        unsafe {
            gl::GenerateMipmap(self.target.as_enum());
        }
        self
    }
}

impl<'a> Drop for BoundTextureId<'a> {
    fn drop(&mut self) {
        unsafe {
            gl::BindTexture(self.target.as_enum(), 0);
        }
    }
}

// pub unsafe fn GenTextures(count: usize, buffer: &mut [Option<TextureId>]) {
//     assert!(count == buffer.len());
//     // can you do this?
//     gl::GenTextures(buffer.len(), buffer.as_mut_ptr() as *mut GLuint);
// }

// #[test]
// fn test_gen_textures() {
//     unsafe {
//         let ids: [Option<TextureId>; 2] = std::mem::uninitialized();
//         GenTextures(1, &mut ids[..]);

//     }
// }
