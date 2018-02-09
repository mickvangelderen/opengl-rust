#![allow(non_snake_case)]

extern crate core;
extern crate gl;

use core::nonzero::NonZero;
use gl::types::*;

#[derive(Debug, Eq, PartialEq)]
pub struct FramebufferId(NonZero<GLuint>);

#[repr(u32)]
pub enum FramebufferTarget {
    Framebuffer = gl::FRAMEBUFFER,
    DrawFramebuffer = gl::DRAW_FRAMEBUFFER,
    ReadFramebuffer = gl::READ_FRAMEBUFFER,
}

impl FramebufferId {
    #[inline]
    pub fn new() -> Option<Self> {
        let mut ids: [GLuint; 1] = [0];
        unsafe {
            gl::GenFramebuffers(ids.len() as GLsizei, ids.as_mut_ptr());
        }
        NonZero::new(ids[0]).map(FramebufferId)
    }

    #[inline]
    pub unsafe fn as_uint(&self) -> GLuint {
        self.0.get()
    }

    #[inline]
    pub fn bind(&self, target: FramebufferTarget) {
        unsafe {
            BindFramebuffer(target, self);
        }
    }

    pub fn bind_default(target: FramebufferTarget) {
        unsafe {
            gl::BindFramebuffer(target as GLenum, 0);
        }
    }
}

impl Drop for FramebufferId {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.as_uint());
        }
    }
}

#[repr(u32)]
#[non_exhaustive]
pub enum FramebufferAttachment {
    Color0 = gl::COLOR_ATTACHMENT0,
    Color1 = gl::COLOR_ATTACHMENT1,
    Color2 = gl::COLOR_ATTACHMENT2,
    Color3 = gl::COLOR_ATTACHMENT3,
    Color4 = gl::COLOR_ATTACHMENT4,
    Color5 = gl::COLOR_ATTACHMENT5,
    Color6 = gl::COLOR_ATTACHMENT6,
    Color7 = gl::COLOR_ATTACHMENT7,
    // Additional colors can be constructed with FramebufferAttachment::color(index: u32).
    Depth = gl::DEPTH_ATTACHMENT,
    Stencil = gl::STENCIL_ATTACHMENT,
    DepthStencil = gl::DEPTH_STENCIL_ATTACHMENT,
}

impl FramebufferAttachment {
    pub fn color(index: u32) -> Self {
        // TODO: Is this function unsafe? What happens when you do color(1234123412341234) and then use that?
        unsafe {
            ::std::mem::transmute::<u32, FramebufferAttachment>(gl::COLOR_ATTACHMENT0 + index)
        }
    }
}

#[inline]
pub unsafe fn BindFramebuffer(target: FramebufferTarget, framebuffer: &FramebufferId) {
    gl::BindFramebuffer(target as GLenum, framebuffer.as_uint());
}

#[inline]
pub unsafe fn FramebufferTexture2D(
    target: FramebufferTarget,
    attachment: FramebufferAttachment,
    tex_target: super::texture::TextureTarget,
    texture: super::texture::TextureId,
    mipmap_level: GLint,
) {
    gl::FramebufferTexture2D(
        target as GLenum,
        attachment as GLenum,
        tex_target.as_enum(),
        texture.as_uint(),
        mipmap_level,
    );
}

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

#[repr(u32)]
#[allow(non_camel_case_types)]
#[non_exhaustive]
pub enum FramebufferStatus {
    ERROR = 0,
    FRAMEBUFFER_COMPLETE = gl::FRAMEBUFFER_COMPLETE,
    FRAMEBUFFER_UNDEFINED = gl::FRAMEBUFFER_UNDEFINED,
    FRAMEBUFFER_INCOMPLETE_ATTACHMENT = gl::FRAMEBUFFER_INCOMPLETE_ATTACHMENT,
    FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT = gl::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT,
    FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER = gl::FRAMEBUFFER_INCOMPLETE_DRAW_BUFFER,
    FRAMEBUFFER_INCOMPLETE_READ_BUFFER = gl::FRAMEBUFFER_INCOMPLETE_READ_BUFFER,
    FRAMEBUFFER_UNSUPPORTED = gl::FRAMEBUFFER_UNSUPPORTED,
    FRAMEBUFFER_INCOMPLETE_MULTISAMPLE = gl::FRAMEBUFFER_INCOMPLETE_MULTISAMPLE,
    FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS = gl::FRAMEBUFFER_INCOMPLETE_LAYER_TARGETS,
}

pub unsafe fn CheckFramebufferStatus(target: FramebufferTarget) -> FramebufferStatus {
    // NOTE: The transmute can lead to undefined behaviour when a driver
    // doesn't return a value that can be represented by the enum. It
    // would be safer and perhaps acceptable to use a switch statement
    // instead.
    ::std::mem::transmute::<GLenum, FramebufferStatus>(gl::CheckFramebufferStatus(target as GLenum))
}
