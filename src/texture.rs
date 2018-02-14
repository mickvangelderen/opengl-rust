extern crate core;
extern crate gl;

use core::nonzero::NonZero;
use gl::types::*;
use std::marker::PhantomData;

/// A more restricted way to construct PhantomData requiring an object
/// of the type embedded in the PhantomData. Helps to prevent mistakes
/// in the implementation.
fn as_phantom_data<T>(_: T) -> PhantomData<T> {
    PhantomData
}

pub trait HasTextureId {
    unsafe fn id(&self) -> u32;
}

//

#[derive(Debug)]
pub struct DefaultTextureId;

impl HasTextureId for DefaultTextureId {
    unsafe fn id(&self) -> u32 {
        0
    }
}

//

#[derive(Debug)]
pub struct TextureId(NonZero<GLuint>);

impl TextureId {
    pub fn new() -> Option<Self> {
        NonZero::new(unsafe {
            let mut ids: [GLuint; 1] = [0];
            gl::GenTextures(ids.len() as GLsizei, ids.as_mut_ptr());
            ids[0]
        }).map(TextureId)
    }
}

impl HasTextureId for TextureId {
    unsafe fn id(&self) -> u32 {
        (self.0).get()
    }
}

impl Drop for TextureId {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id());
        }
    }
}

//

pub struct BoundTextureId<'a, THasTextureId: 'a + HasTextureId, TTextureTarget: TextureTarget> {
    target: TTextureTarget,
    texture: PhantomData<&'a THasTextureId>,
}

pub struct TextureSlot1D;
pub struct TextureSlot2D;
pub struct TextureSlot3D;
pub struct TextureSlot1DArray;
pub struct TextureSlot2DArray;
pub struct TextureSlotRectangle;
pub struct TextureSlotCubeMap;
pub struct TextureSlotCubeMapArray;
pub struct TextureSlotBuffer;
pub struct TextureSlot2DMultisample;
pub struct TextureSlot2DMultisampleArray;

impl TextureSlot1D {
    pub fn target(&mut self) -> TextureTarget1D {
        TextureTarget1D::new(self)
    }
}

impl TextureSlot2D {
    pub fn target(&mut self) -> TextureTarget2D {
        TextureTarget2D::new(self)
    }
}

impl TextureSlot3D {
    pub fn target(&mut self) -> TextureTarget3D {
        TextureTarget3D::new(self)
    }
}

pub trait TextureTarget: Sized {
    fn as_enum(&self) -> u32;

    fn bind<THasTextureId: HasTextureId>(
        self,
        texture: &THasTextureId,
    ) -> BoundTextureId<THasTextureId, Self> {
        unsafe {
            gl::BindTexture(self.as_enum(), texture.id());
        }
        BoundTextureId {
            target: self,
            texture: as_phantom_data(texture),
        }
    }
}

macro_rules! impl_texture_target(
    ($TT:ident, $TS:ident, $enum:expr) => {
        pub struct $TT<'a>(PhantomData<&'a mut $TS>);

        impl<'a> $TT<'a> {
            pub fn new(slot: &'a mut $TS) -> Self {
                $TT(as_phantom_data(slot))
            }
        }

        impl<'a> TextureTarget for $TT<'a> {
            fn as_enum(&self) -> u32 {
                $enum
            }
        }
    }
);

impl_texture_target!(TextureTarget1D, TextureSlot1D, gl::TEXTURE_1D);
impl_texture_target!(TextureTarget2D, TextureSlot2D, gl::TEXTURE_2D);
impl_texture_target!(TextureTarget3D, TextureSlot3D, gl::TEXTURE_3D);

// TODO: the rest
// pub struct TextureTarget1DArray<'a>(PhantomData<&'a mut TextureSlot1DArray>);
// pub struct TextureTarget2DArray<'a>(PhantomData<&'a mut TextureSlot2DArray>);
// pub struct TextureTargetRectangle<'a>(PhantomData<&'a mut TextureSlotRectangle>);
// pub struct TextureTargetCubeMap<'a>(PhantomData<&'a mut TextureSlotCubeMap>);
// pub struct TextureTargetCubeMapArray<'a>(PhantomData<&'a mut TextureSlotCubeMapArray>);
// pub struct TextureTargetBuffer<'a>(PhantomData<&'a mut TextureSlotBuffer>);
// pub struct TextureTarget2DMultisample<'a>(PhantomData<&'a mut TextureSlot2DMultisample>);
// pub struct TextureTarget2DMultisampleArray<'a>(PhantomData<&'a mut TextureSlot2DMultisampleArray>);

pub trait TextureTargetGroup1DPlus: TextureTarget {}
pub trait TextureTargetGroup2DPlus: TextureTargetGroup1DPlus {}
pub trait TextureTargetGroup3DPlus: TextureTargetGroup2DPlus {}

pub trait TextureTargetGroup1D: TextureTargetGroup1DPlus {}
pub trait TextureTargetGroup2D: TextureTargetGroup2DPlus {}
pub trait TextureTargetGroup3D: TextureTargetGroup3DPlus {}

// Automatically implement lower dimensional plus groups.
impl<T: TextureTargetGroup3DPlus> TextureTargetGroup2DPlus for T {}
impl<T: TextureTargetGroup2DPlus> TextureTargetGroup1DPlus for T {}

// Automatically implement plus groups.
// impl<T: TextureTargetGroup1D> TextureTargetGroup1DPlus for T {}
// impl<T: TextureTargetGroup2D> TextureTargetGroup2DPlus for T {}
// impl<T: TextureTargetGroup3D> TextureTargetGroup3DPlus for T {}

// Implement groups.
impl<'a> TextureTargetGroup1D for TextureTarget1D<'a> {}
impl<'a> TextureTargetGroup1DPlus for TextureTarget1D<'a> {}
impl<'a> TextureTargetGroup2D for TextureTarget2D<'a> {}
impl<'a> TextureTargetGroup2DPlus for TextureTarget2D<'a> {}
impl<'a> TextureTargetGroup3D for TextureTarget3D<'a> {}
impl<'a> TextureTargetGroup3DPlus for TextureTarget3D<'a> {}

impl<'a, T: 'a + TextureTarget> BoundTextureId<'a, TextureId, T> {
    pub fn target_as_enum(&self) -> u32 {
        self.target.as_enum()
    }

    fn parameter_i(&mut self, param: GLenum, value: GLint) -> &mut Self {
        unsafe {
            gl::TexParameteri(self.target.as_enum(), param, value);
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
            gl::GenerateMipmap(self.target.as_enum());
        }
        self
    }
}

impl<'a, T: 'a + TextureTargetGroup1DPlus> BoundTextureId<'a, TextureId, T> {
    pub fn wrap_s(&mut self, value: GLint) -> &mut Self {
        self.parameter_i(gl::TEXTURE_WRAP_S, value)
    }
}

impl<'a, T: 'a + TextureTargetGroup2DPlus> BoundTextureId<'a, TextureId, T> {
    pub fn wrap_t(&mut self, value: GLint) -> &mut Self {
        self.parameter_i(gl::TEXTURE_WRAP_T, value)
    }
}

impl<'a, T: 'a + TextureTargetGroup3DPlus> BoundTextureId<'a, TextureId, T> {
    pub fn wrap_r(&mut self, value: GLint) -> &mut Self {
        self.parameter_i(gl::TEXTURE_WRAP_R, value)
    }
}

impl<'a, T: 'a + TextureTargetGroup2D> BoundTextureId<'a, TextureId, T> {
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

pub struct TextureUnitSlot;

impl TextureUnitSlot {
    pub fn activate(&mut self, unit: TextureUnit) -> ActiveTextureUnit {
        unsafe {
            gl::ActiveTexture(unit as GLenum);
        }

        ActiveTextureUnit {
            slot: as_phantom_data(self),
            texture_slot_1d: TextureSlot1D,
            texture_slot_2d: TextureSlot2D,
            texture_slot_3d: TextureSlot3D,
            texture_slot_1d_array: TextureSlot1DArray,
            texture_slot_2d_array: TextureSlot2DArray,
            texture_slot_rectangle: TextureSlotRectangle,
            texture_slot_cube_map: TextureSlotCubeMap,
            texture_slot_cube_map_array: TextureSlotCubeMapArray,
            texture_slot_buffer: TextureSlotBuffer,
            texture_slot_2d_multisample: TextureSlot2DMultisample,
            texture_slot_2d_multisample_array: TextureSlot2DMultisampleArray,
        }
    }
}

#[repr(u32)]
pub enum TextureUnit {
    TextureUnit0 = gl::TEXTURE0,
    TextureUnit1 = gl::TEXTURE1,
}

pub struct ActiveTextureUnit<'a> {
    slot: PhantomData<&'a mut TextureUnitSlot>,
    pub texture_slot_1d: TextureSlot1D,
    pub texture_slot_2d: TextureSlot2D,
    pub texture_slot_3d: TextureSlot3D,
    pub texture_slot_1d_array: TextureSlot1DArray,
    pub texture_slot_2d_array: TextureSlot2DArray,
    pub texture_slot_rectangle: TextureSlotRectangle,
    pub texture_slot_cube_map: TextureSlotCubeMap,
    pub texture_slot_cube_map_array: TextureSlotCubeMapArray,
    pub texture_slot_buffer: TextureSlotBuffer,
    pub texture_slot_2d_multisample: TextureSlot2DMultisample,
    pub texture_slot_2d_multisample_array: TextureSlot2DMultisampleArray,
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
