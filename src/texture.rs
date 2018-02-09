extern crate core;
extern crate gl;

use core::nonzero::NonZero;
use gl::types::*;

pub trait TextureTarget {
    fn as_enum() -> GLenum;
}

pub trait TextureTarget1dPlus: TextureTarget {}
pub trait TextureTarget2dPlus: TextureTarget1dPlus {}
pub trait TextureTarget3dPlus: TextureTarget2dPlus {}

impl<T: TextureTarget2dPlus> TextureTarget1dPlus for T {}
impl<T: TextureTarget3dPlus> TextureTarget2dPlus for T {}

#[derive(Debug)]
pub struct TextureTarget1d();

impl TextureTarget for TextureTarget1d {
    fn as_enum() -> GLenum {
        gl::TEXTURE_1D
    }
}

impl TextureTarget1dPlus for TextureTarget1d {}

#[derive(Debug)]
pub struct TextureTarget2d();

impl TextureTarget for TextureTarget2d {
    fn as_enum() -> GLenum {
        gl::TEXTURE_2D
    }
}
impl TextureTarget2dPlus for TextureTarget2d {}

#[derive(Debug)]
pub struct TextureTarget3d;

impl TextureTarget for TextureTarget3d {
    fn as_enum() -> GLenum {
        gl::TEXTURE_3D
    }
}

impl TextureTarget3dPlus for TextureTarget3d {}

pub struct TextureUnitSlot {}

impl TextureUnitSlot {
    pub fn active_texture(&mut self, unit: TextureUnit) -> ActiveTextureUnit {
        ActiveTextureUnit::new(self, unit)
    }
}

#[repr(u32)]
pub enum TextureUnit {
    TextureUnit0 = gl::TEXTURE0,
    TextureUnit1 = gl::TEXTURE1,
}

pub struct ActiveTextureUnit<'a> {
    texture_unit_slot: PhantomData<&'a mut TextureUnitSlot>,
    pub texture_target_1d: TextureTarget1d,
    pub texture_target_2d: TextureTarget2d,
    pub texture_target_3d: TextureTarget3d,
}

impl<'a> ActiveTextureUnit<'a> {
    fn new(_slot: &'a mut TextureUnitSlot, unit: TextureUnit) -> Self {
        unsafe {
            gl::ActiveTexture(unit as GLenum);
        }

        ActiveTextureUnit {
            texture_unit_slot: PhantomData,
            texture_target_1d: TextureTarget1d {},
            texture_target_2d: TextureTarget2d {},
            texture_target_3d: TextureTarget3d {},
        }
    }
}

// NOTE(mickvangelderen): Not necessary.
//
// impl<'a> Drop for ActiveTextureUnit<'a> {
//     fn drop(&mut self) {
//         unsafe {
//             gl::ActiveTexture(TextureUnit::TextureUnit0 as GLenum);
//         }
//     }
// }

// macro_rules! impl_texture_unit {
//     ($Name:ident, $enum:expr) => {
//         pub struct TextureUnit

//     }
// }

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

    pub fn bind<'a, T: 'a + TextureTarget>(&'a self, _target: &'a mut T) -> BoundTextureId<T> {
        unsafe {
            gl::BindTexture(T::as_enum(), self.as_uint());
        }
        BoundTextureId {
            id: PhantomData,
            target: PhantomData,
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

pub struct BoundTextureId<'a, T: 'a + TextureTarget> {
    id: PhantomData<&'a TextureId>,
    target: PhantomData<&'a mut T>,
}

#[repr(u32)]
pub enum TextureFilter {
    /// Returns the value of the texture element that is nearest (in
    /// Manhattan distance) to the specified texture coordinates.
    Nearest = gl::NEAREST,

    /// Returns the weighted average of the four texture elements that
    /// are closest to the specified texture coordinates. These can
    /// include items wrapped or repeated from other parts of a texture,
    /// depending on the values of GL_TEXTURE_WRAP_S and
    /// GL_TEXTURE_WRAP_T, and on the exact mapping.
    Linear = gl::LINEAR,

    /// Chooses the mipmap that most closely matches the size of the
    /// pixel being textured and uses the GL_NEAREST criterion (the
    /// texture element closest to the specified texture coordinates) to
    /// produce a texture value.
    NearestMipmapNearest = gl::NEAREST_MIPMAP_NEAREST,

    /// Chooses the mipmap that most closely matches the size of the
    /// pixel being textured and uses the GL_LINEAR criterion (a
    /// weighted average of the four texture elements that are closest
    /// to the specified texture coordinates) to produce a texture
    /// value.
    LinearMipmapNearest = gl::LINEAR_MIPMAP_NEAREST,


    /// Chooses the two mipmaps that most closely match the size of the
    /// pixel being textured and uses the GL_NEAREST criterion (the
    /// texture element closest to the specified texture coordinates )
    /// to produce a texture value from each mipmap. The final texture
    /// value is a weighted average of those two values.
    NearestMipmapLinear = gl::NEAREST_MIPMAP_LINEAR,

    /// Chooses the two mipmaps that most closely match the size of the
    /// pixel being textured and uses the GL_LINEAR criterion (a
    /// weighted average of the texture elements that are closest to the
    /// specified texture coordinates) to produce a texture value from
    /// each mipmap. The final texture value is a weighted average of
    /// those two values.
    LinearMipmapLinear = gl::LINEAR_MIPMAP_LINEAR,
}

impl<'a, T: 'a + TextureTarget> BoundTextureId<'a, T> {

    fn parameter_i(&mut self, param: GLenum, value: GLint) -> &mut Self {
        unsafe {
            gl::TexParameteri(T::as_enum(), param, value);
        }
        self
    }

    pub fn min_filter(&mut self, value: TextureFilter) -> &mut Self {
        self.parameter_i(gl::TEXTURE_MIN_FILTER, value as GLint)
    }

    pub fn mag_filter(&mut self, value: TextureFilter) -> &mut Self {
        self.parameter_i(gl::TEXTURE_MAG_FILTER, value as GLint)
    }

    pub fn generate_mipmap(&mut self) -> &mut Self {
        unsafe {
            gl::GenerateMipmap(T::as_enum());
        }
        self
    }
}

impl<'a, T: 'a + TextureTarget1dPlus> BoundTextureId<'a, T> {
    pub fn wrap_s(&mut self, value: GLint) -> &mut Self {
        self.parameter_i(gl::TEXTURE_WRAP_S, value)
    }
}

impl<'a, T: 'a + TextureTarget2dPlus> BoundTextureId<'a, T> {
    pub fn wrap_t(&mut self, value: GLint) -> &mut Self {
        self.parameter_i(gl::TEXTURE_WRAP_T, value)
    }
}

impl<'a, T: 'a + TextureTarget3dPlus> BoundTextureId<'a, T> {
    pub fn wrap_r(&mut self, value: GLint) -> &mut Self {
        self.parameter_i(gl::TEXTURE_WRAP_R, value)
    }
}

impl<'a> BoundTextureId<'a, TextureTarget2d> {
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
            TextureTarget2d::as_enum(),
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
}

// NOTE(mickvangelderen): It would be a mistake to implement drop like this.
// The following code requires the texture to stay bound when switching
// texture units.
// ```rust
// gl::ActiveTexture(gl::TEXTURE0);
// gl::BindTexture(gl::TEXTURE_2D, 1);
// gl::ActiveTexture(gl::TEXTURE1);
// gl::BindTexture(gl::TEXTURE_2D, 2);
// ```
// impl<'a, T: 'a + TextureTarget> Drop for BoundTextureId<'a, T> {
//     fn drop(&mut self) {
//         unsafe {
//             gl::BindTexture(T::as_enum(), 0);
//         }
//     }
// }

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
