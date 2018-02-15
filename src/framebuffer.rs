#![allow(non_snake_case)]

extern crate core;
extern crate gl;

use core::nonzero::NonZero;
use gl::types::*;
use std::marker::PhantomData;
use renderbuffer::RenderbufferId;

pub trait HasFramebufferId {
    unsafe fn id(&self) -> u32;
}

pub struct DefaultFramebufferId(());

pub const DEFAULT_FRAMEBUFFER_ID: DefaultFramebufferId = DefaultFramebufferId(());

impl HasFramebufferId for DefaultFramebufferId {
    #[inline]
    unsafe fn id(&self) -> u32 {
        0
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct FramebufferId(NonZero<GLuint>);

impl FramebufferId {
    #[inline]
    pub fn new() -> Option<Self> {
        let mut ids: [GLuint; 1] = [0];
        unsafe {
            gl::GenFramebuffers(ids.len() as GLsizei, ids.as_mut_ptr());
        }
        NonZero::new(ids[0]).map(FramebufferId)
    }
}

impl Drop for FramebufferId {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.id());
        }
    }
}

impl HasFramebufferId for FramebufferId {
    #[inline]
    unsafe fn id(&self) -> u32 {
        self.0.get()
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

pub unsafe fn CheckFramebufferStatus<FT: IsFramebufferTarget>(target: FT) -> FramebufferStatus {
    // NOTE: The transmute can lead to undefined behaviour when a driver
    // doesn't return a value that can be represented by the enum. It
    // would be safer and perhaps acceptable to use a switch statement
    // instead.
    ::std::mem::transmute::<u32, FramebufferStatus>(gl::CheckFramebufferStatus(target.as_enum()))
}

/// A slot represents a resource that can be occupied.
/// A target represents a reservation of of one or more slots.
/// An id can be bound to a target.

/// this principle can be applied to all opengl resources (probably), for example textures
/// texture_unit_slot
/// texture_unit_target(texture_unit_slot)
/// bound_texture_id(texture_unit_target, texture_id)

pub struct DrawFramebufferSlot {}
pub struct ReadFramebufferSlot {}

pub struct DrawReadFramebufferTarget<'a>(
    PhantomData<&'a mut DrawFramebufferSlot>,
    PhantomData<&'a mut ReadFramebufferSlot>,
);

impl<'a> DrawReadFramebufferTarget<'a> {
    pub fn new(draw: &'a mut DrawFramebufferSlot, read: &mut ReadFramebufferSlot) -> Self {
        let _ = (draw, read);
        DrawReadFramebufferTarget(PhantomData, PhantomData)
    }
}

pub struct DrawFramebufferTarget<'a>(PhantomData<&'a mut DrawFramebufferSlot>);

impl<'a> DrawFramebufferTarget<'a> {
    pub fn new(draw: &'a mut DrawFramebufferSlot) -> Self {
        let _ = draw;
        DrawFramebufferTarget(PhantomData)
    }
}

pub struct ReadFramebufferTarget<'a>(PhantomData<&'a mut ReadFramebufferSlot>);

impl<'a> ReadFramebufferTarget<'a> {
    pub fn new(read: &mut ReadFramebufferSlot) -> Self {
        let _ = read;
        ReadFramebufferTarget(PhantomData)
    }
}

impl<'a> IsFramebufferTarget for DrawReadFramebufferTarget<'a> {
    fn as_enum(&self) -> u32 {
        gl::FRAMEBUFFER
    }
}

impl<'a> HasReadFramebufferSlot<'a> for DrawReadFramebufferTarget<'a> {
    fn prove<'b: 'a>(&'b self) -> &'b PhantomData<&'a mut ReadFramebufferSlot> {
        &self.1
    }
}

impl<'a> HasDrawFramebufferSlot<'a> for DrawReadFramebufferTarget<'a> {
    fn prove<'b: 'a>(&'b self) -> &'b PhantomData<&'a mut DrawFramebufferSlot> {
        &self.0
    }
}

impl<'a> IsFramebufferTarget for DrawFramebufferTarget<'a> {
    fn as_enum(&self) -> u32 {
        gl::DRAW_FRAMEBUFFER
    }
}

impl<'a> HasDrawFramebufferSlot<'a> for DrawFramebufferTarget<'a> {
    fn prove<'b: 'a>(&'b self) -> &'b PhantomData<&'a mut DrawFramebufferSlot> {
        &self.0
    }
}

impl<'a> IsFramebufferTarget for ReadFramebufferTarget<'a> {
    fn as_enum(&self) -> u32 {
        gl::READ_FRAMEBUFFER
    }
}

impl<'a> HasReadFramebufferSlot<'a> for ReadFramebufferTarget<'a> {
    fn prove<'b: 'a>(&'b self) -> &'b PhantomData<&'a mut ReadFramebufferSlot> {
        &self.0
    }
}

/// Marker trait that can only be implemented on framebuffer
/// targets that mutably burrow a DrawFramebufferSlot.
pub trait HasDrawFramebufferSlot<'a>: IsFramebufferTarget {
    fn prove<'b: 'a>(&'b self) -> &'b PhantomData<&'a mut DrawFramebufferSlot>;
}

/// Marker trait that can only be implemented on framebuffer
/// targets that mutably burrow a ReadFramebufferSlot.
pub trait HasReadFramebufferSlot<'a>: IsFramebufferTarget {
    fn prove<'b: 'a>(&'b self) -> &'b PhantomData<&'a mut ReadFramebufferSlot>;
}

pub trait IsFramebufferTarget: Sized {
    fn as_enum(&self) -> u32;

    fn bind<THasFramebufferId: HasFramebufferId>(
        self,
        framebuffer: &THasFramebufferId,
    ) -> BoundFramebufferId<THasFramebufferId, Self> {
        unsafe {
            gl::BindFramebuffer(self.as_enum(), framebuffer.id());
        }
        BoundFramebufferId {
            target: self,
            framebuffer: PhantomData::<&THasFramebufferId>,
        }
    }
}

/// Functions available to bound framebuffers with a draw slot.
pub trait IsDrawableBoundFramebufferId {
    fn draw(&self) -> bool;
}

/// Functions available to bound framebuffers with a read slot.
pub trait IsReadableBoundFramebufferId {
    fn read(&self) -> bool;
}

#[must_use]
pub struct BoundFramebufferId<
    'a,
    THasFramebufferId: 'a + HasFramebufferId,
    TFramebufferTarget: 'a + IsFramebufferTarget,
> {
    target: TFramebufferTarget,
    framebuffer: PhantomData<&'a THasFramebufferId>,
}

impl<'a, THasFramebufferId, THasDrawFramebufferSlot> IsDrawableBoundFramebufferId
    for BoundFramebufferId<'a, THasFramebufferId, THasDrawFramebufferSlot>
where
    THasFramebufferId: 'a + HasFramebufferId,
    THasDrawFramebufferSlot: 'a + HasDrawFramebufferSlot<'a>,
{
    fn draw(&self) -> bool {
        false
    }
}

impl<'a, THasFramebufferId, THasReadFramebufferSlot> IsReadableBoundFramebufferId
    for BoundFramebufferId<'a, THasFramebufferId, THasReadFramebufferSlot>
where
    THasFramebufferId: 'a + HasFramebufferId,
    THasReadFramebufferSlot: 'a + HasReadFramebufferSlot<'a>,
{
    fn read(&self) -> bool {
        false
    }
}

impl<'a> BoundFramebufferId<'a, FramebufferId, DrawReadFramebufferTarget<'a>> {
    /// glFramebufferTexture2D attaches the texture image specified by
    /// texture and level as one of the logical buffers of the currently
    /// bound framebuffer object. attachment specifies whether the texture
    /// image should be attached to the framebuffer object's color, depth,
    /// or stencil buffer. A texture image may not be attached to the
    /// default framebuffer object name 0.
    ///
    /// If texture is not 0, the value of
    /// GL_FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE for the specified attachment
    /// point is set to GL_TEXTURE, the value of
    /// GL_FRAMEBUFFER_ATTACHMENT_OBJECT_NAME is set to texture, and the
    /// value of GL_FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL is set to level. If
    /// texture is a cube map texture, the value of
    /// GL_FRAMEBUFFER_ATTACHMENT_TEXTURE_CUBE_MAP_FACE is set to textarget;
    /// otherwise it is set to the default value
    /// GL_TEXTURE_CUBE_MAP_POSITIVE_X. Any previous attachment to the
    /// attachment logical buffer of the currently bound framebuffer object
    /// is broken.
    ///
    /// If texture is 0, the current image, if any, attached to the
    /// attachment logical buffer of the currently bound framebuffer object
    /// is detached. The value of GL_FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE is
    /// set to GL_NONE. The value of GL_FRAMEBUFFER_ATTACHMENT_OBJECT_NAME
    /// is set to 0. GL_FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL and
    /// GL_FRAMEBUFFER_ATTACHMENT_TEXTURE_CUBE_MAP_FACE are set to the
    /// default values 0 and GL_TEXTURE_CUBE_MAP_POSITIVE_X, respectively.
    pub unsafe fn attach_texture_2d(
        &mut self,
        attachment: FramebufferAttachment,
        textarget: GLenum,
        texture: GLuint,
        level: GLint,
    ) -> &mut Self {
        gl::FramebufferTexture2D(
            self.target.as_enum(),
            attachment as GLenum,
            textarget,
            texture,
            level,
        );
        self
    }

    pub fn attach_renderbuffer(
        &mut self,
        attachment: FramebufferAttachment,
        renderbuffer_id: &RenderbufferId,
    ) -> &mut Self {
        unsafe {
            gl::FramebufferRenderbuffer(
                self.target.as_enum(),
                attachment as GLenum,
                gl::RENDERBUFFER,
                renderbuffer_id.as_u32(),
            );
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::DEFAULT_FRAMEBUFFER_ID;
    use super::FramebufferId;
    use super::DrawFramebufferSlot;
    use super::ReadFramebufferSlot;
    use super::DrawReadFramebufferTarget;
    use super::DrawFramebufferTarget;
    use super::ReadFramebufferTarget;
    use super::IsFramebufferTarget;
    use super::BoundFramebufferId;
    use super::IsDrawableBoundFramebufferId;
    use super::IsReadableBoundFramebufferId;

    #[test]
    fn default_framebuffer_id_has_size_0() {
        assert_eq!(0, ::std::mem::size_of_val(&DEFAULT_FRAMEBUFFER_ID));
    }

    #[test]
    fn framebuffer_id_has_size_4() {
        assert_eq!(4, ::std::mem::size_of::<FramebufferId>());
    }

    #[test]
    fn framebuffer_targets_have_size_0() {
        assert_eq!(0, ::std::mem::size_of::<DrawReadFramebufferTarget>());
        assert_eq!(0, ::std::mem::size_of::<DrawFramebufferTarget>());
        assert_eq!(0, ::std::mem::size_of::<ReadFramebufferTarget>());
    }

    #[test]
    fn bound_framebuffer_ids_have_size_0() {
        assert_eq!(0, ::std::mem::size_of::<BoundFramebufferId<FramebufferId, DrawReadFramebufferTarget>>());
    }

    #[test]
    fn test() {
        let fb0 = DEFAULT_FRAMEBUFFER_ID;
        let fb1 = FramebufferId::new().unwrap();
        let fb2 = FramebufferId::new().unwrap();

        let mut draw = DrawFramebufferSlot {};
        let mut read = ReadFramebufferSlot {};

        {
            let bb0 = DrawReadFramebufferTarget::new(&mut draw, &mut read).bind(&fb0);
            assert_eq!(bb0.draw(), false);
            assert_eq!(bb0.read(), false);
        }

        {
            let br0 = ReadFramebufferTarget::new(&mut read).bind(&fb0);

            // assert_eq!(br0.draw(), false);
            assert_eq!(br0.read(), false);

            {
                let bd1 = DrawFramebufferTarget::new(&mut draw).bind(&fb1);
                assert_eq!(bd1.draw(), false);
                // assert_eq!(bd1.read(), false);
            }

            {
                let bd2 = DrawFramebufferTarget::new(&mut draw).bind(&fb2);
                assert_eq!(bd2.draw(), false);
                // assert_eq!(bd2.read(), false);
            }
        }
    }
}
