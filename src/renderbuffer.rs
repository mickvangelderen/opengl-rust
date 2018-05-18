#![allow(non_snake_case)]

extern crate core;
extern crate gl;

use id::Id;
use gl::types::*;
use std::marker::PhantomData;
use phantomdata::into_phantom_data;

pub struct RenderbufferId(Id);

impl RenderbufferId {
    #[inline]
    pub fn new() -> Option<Self> {
        let mut ids: [GLuint; 1] = [0];
        unsafe {
            gl::GenRenderbuffers(ids.len() as GLsizei, ids.as_mut_ptr());
        }
        Id::new(ids[0]).map(RenderbufferId)
    }

    #[inline]
    pub unsafe fn as_u32(&self) -> GLuint {
        self.0.get()
    }
}

impl Drop for RenderbufferId {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            gl::DeleteRenderbuffers(1, &self.as_u32());
        }
    }
}

pub struct RenderbufferSlot;

impl RenderbufferSlot {
    #[inline]
    pub fn target(&mut self) -> RenderbufferTarget {
        RenderbufferTarget::new(self)
    }
}

pub struct RenderbufferTarget<'s>(PhantomData<&'s mut RenderbufferSlot>);

impl<'s> RenderbufferTarget<'s> {
    pub fn new(slot: &'s mut RenderbufferSlot) -> Self {
        RenderbufferTarget(into_phantom_data(slot))
    }
}

impl<'s> RenderbufferTarget<'s> {
    #[inline]
    fn as_enum(&self) -> u32 {
        gl::RENDERBUFFER
    }

    #[inline]
    pub fn unbind(self) {
        unsafe {
            gl::BindRenderbuffer(self.as_enum(), 0);
        }
    }

    #[inline]
    pub fn bind<'t, 'i>(
        &'t mut self,
        renderbuffer_id: &'i RenderbufferId,
    ) -> BoundRenderbufferId<'s, 't, 'i> {
        unsafe {
            gl::BindRenderbuffer(self.as_enum(), renderbuffer_id.as_u32());
        }
        BoundRenderbufferId {
            target: self,
            renderbuffer_id: into_phantom_data(renderbuffer_id),
        }
    }
}

pub struct BoundRenderbufferId<'s: 't, 't, 'i> {
    target: &'t mut RenderbufferTarget<'s>,
    renderbuffer_id: PhantomData<&'i RenderbufferId>,
}

impl<'s: 't, 't, 'i> BoundRenderbufferId<'s, 't, 'i> {
    #[inline]
    pub fn storage(
        &mut self,
        internal_format: RenderbufferInternalFormat,
        width: GLsizei,
        height: GLsizei,
    ) -> &mut Self {
        unsafe {
            gl::RenderbufferStorage(
                self.target.as_enum(),
                internal_format as GLenum,
                width,
                height,
            );
        }
        self
    }
}

#[repr(u32)]
#[allow(non_camel_case_types)]
pub enum RenderbufferInternalFormat {
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
