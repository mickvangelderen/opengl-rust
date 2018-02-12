#![allow(non_snake_case)]

extern crate core;
extern crate gl;

use core::nonzero::NonZero;
use gl::types::*;

#[repr(u32)]
pub enum RenderBufferTarget {
    RenderBuffer = gl::RENDERBUFFER,
}

pub struct RenderBufferId(NonZero<GLuint>);

impl RenderBufferId {
    #[inline]
    pub fn new() -> Option<Self> {
        let mut ids: [GLuint; 1] = [0];
        unsafe {
            gl::GenRenderbuffers(ids.len() as GLsizei, ids.as_mut_ptr());
        }
        NonZero::new(ids[0]).map(RenderBufferId)
    }

    #[inline]
    pub unsafe fn as_uint(&self) -> GLuint {
        self.0.get()
    }

    #[inline]
    pub fn bind(&self, target: RenderBufferTarget) {
        unsafe {
            BindRenderbuffer(target, self);
        }
    }
}

impl Drop for RenderBufferId {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            gl::DeleteRenderbuffers(1, &self.as_uint());
        }
    }
}

#[inline]
pub unsafe fn BindRenderbuffer(target: RenderBufferTarget, renderbuffer: &RenderBufferId) {
    gl::BindRenderbuffer(target as GLenum, renderbuffer.as_uint());
}

#[repr(u32)]
#[allow(non_camel_case_types)]
pub enum RenderBufferInternalFormat {
    R8 = gl::R8,
    R8UI = gl::R8UI,
    R8I = gl::R8I,
    R16UI = gl::R16UI,
    R16I = gl::R16I,
    R32UI = gl::R32UI,
    R32I = gl::R32I,
    RG8 = gl::RG8,
    RG8UI = gl::RG8UI,
    RG8I = gl::RG8I,
    RG16UI = gl::RG16UI,
    RG16I = gl::RG16I,
    RG32UI = gl::RG32UI,
    RG32I = gl::RG32I,
    RGB8 = gl::RGB8,
    RGB565 = gl::RGB565,
    RGBA8 = gl::RGBA8,
    SRGB8_ALPHA8 = gl::SRGB8_ALPHA8,
    RGB5_A1 = gl::RGB5_A1,
    RGBA4 = gl::RGBA4,
    RGB10_A2 = gl::RGB10_A2,
    RGBA8UI = gl::RGBA8UI,
    RGBA8I = gl::RGBA8I,
    RGB10_A2UI = gl::RGB10_A2UI,
    RGBA16UI = gl::RGBA16UI,
    RGBA16I = gl::RGBA16I,
    RGBA32I = gl::RGBA32I,
    RGBA32UI = gl::RGBA32UI,
    DEPTH_COMPONENT16 = gl::DEPTH_COMPONENT16,
    DEPTH_COMPONENT24 = gl::DEPTH_COMPONENT24,
    DEPTH_COMPONENT32F = gl::DEPTH_COMPONENT32F,
    DEPTH24_STENCIL8 = gl::DEPTH24_STENCIL8,
    DEPTH32F_STENCIL8 = gl::DEPTH32F_STENCIL8,
    STENCIL_INDEX8 = gl::STENCIL_INDEX8,
}

pub unsafe fn RenderbufferStorage(
    target: RenderBufferTarget,
    internal_format: RenderBufferInternalFormat,
    width: GLsizei,
    height: GLsizei,
) {
    gl::RenderbufferStorage(target as GLenum, internal_format as GLenum, width, height);
}
