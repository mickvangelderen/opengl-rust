#![allow(non_snake_case)]

extern crate core;
extern crate gl;

use core::nonzero::NonZero;
use gl::types::*;
use std::marker::PhantomData;

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

impl HasFramebufferId for FramebufferId {
    #[inline]
    unsafe fn id(&self) -> u32 {
        self.0.get()
    }
}

#[repr(u32)]
pub enum FramebufferTarget {
    Framebuffer = gl::FRAMEBUFFER,
    DrawFramebuffer = gl::DRAW_FRAMEBUFFER,
    ReadFramebuffer = gl::READ_FRAMEBUFFER,
}

pub struct FramebufferSlot {}

impl FramebufferSlot {
    #[inline]
    pub fn bind<THasFramebufferId>(
        &mut self,
        target: FramebufferTarget,
        framebuffer: &THasFramebufferId,
    ) -> BoundFramebufferId<THasFramebufferId>
    where
        THasFramebufferId: HasFramebufferId,
    {
        unsafe {
            gl::BindFramebuffer(target as GLenum, framebuffer.id());
        }
        BoundFramebufferId {
            slot: PhantomData,
            framebuffer: PhantomData,
        }
    }
}

pub struct BoundFramebufferId<'s, 'f, THasFramebufferId: 'f + HasFramebufferId> {
    slot: PhantomData<&'s mut FramebufferSlot>,
    framebuffer: PhantomData<&'f THasFramebufferId>,
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
}

impl Drop for FramebufferId {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.id());
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
    gl::BindFramebuffer(target as GLenum, framebuffer.id());
}

#[inline]
pub unsafe fn FramebufferTexture2D<TT: super::texture::TextureTarget>(
    target: FramebufferTarget,
    attachment: FramebufferAttachment,
    _tex_target: &mut TT,
    texture: super::texture::TextureId,
    mipmap_level: GLint,
) {
    gl::FramebufferTexture2D(
        target as GLenum,
        attachment as GLenum,
        TT::as_enum(),
        texture.as_uint(),
        mipmap_level,
    );
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

mod experiment {
    /// A slot represents a resource that can be occupied.
    /// A target represents a reservation of of one or more slots.
    /// An id can be bound to a target.

    /// this principle can be applied to all opengl resources (probably), for example textures
    /// texture_unit_slot
    /// texture_unit_target(texture_unit_slot)
    /// bound_texture_id(texture_unit_target, texture_id)

    use std::marker::PhantomData;

    use super::FramebufferId;
    use super::HasFramebufferId;

    use super::gl;

    pub struct DrawFramebufferSlot {}

    pub struct ReadFramebufferSlot {}

    // pub struct FramebufferSlot {
    //     draw: DrawFramebufferSlot,
    //     read: ReadFramebufferSlot,
    // }

    pub struct DrawReadFramebufferTarget<'a>(
        &'a mut DrawFramebufferSlot,
        &'a mut ReadFramebufferSlot,
    );
    pub struct DrawFramebufferTarget<'a>(&'a mut DrawFramebufferSlot);
    pub struct ReadFramebufferTarget<'a>(&'a mut ReadFramebufferSlot);

    impl<'a> traits::IsFramebufferTarget for DrawReadFramebufferTarget<'a> {
        fn as_enum() -> u32 {
            gl::FRAMEBUFFER
        }
    }

    impl<'a> traits::IsReadFramebufferTarget<'a> for DrawReadFramebufferTarget<'a> {
        fn prove<'b: 'a>(&'b self) -> &'b &'a mut ReadFramebufferSlot {
            &self.1
        }
    }

    impl<'a> traits::IsDrawFramebufferTarget<'a> for DrawReadFramebufferTarget<'a> {
        fn prove<'b: 'a>(&'b self) -> &'b &'a mut DrawFramebufferSlot {
            &self.0
        }
    }

    impl<'a> traits::IsFramebufferTarget for DrawFramebufferTarget<'a> {
        fn as_enum() -> u32 {
            gl::DRAW_FRAMEBUFFER
        }
    }

    impl<'a> traits::IsDrawFramebufferTarget<'a> for DrawFramebufferTarget<'a> {
        fn prove<'b: 'a>(&'b self) -> &'b &'a mut DrawFramebufferSlot {
            &self.0
        }
    }

    impl<'a> traits::IsFramebufferTarget for ReadFramebufferTarget<'a> {
        fn as_enum() -> u32 {
            gl::READ_FRAMEBUFFER
        }
    }

    impl<'a> traits::IsReadFramebufferTarget<'a> for ReadFramebufferTarget<'a> {
        fn prove<'b: 'a>(&'b self) -> &'b &'a mut ReadFramebufferSlot {
            &self.0
        }
    }

    mod traits {
        use super::*;

        pub trait IsFramebufferTarget: Sized {
            fn as_enum() -> u32;

            fn bind<TFramebufferId: HasFramebufferId>(
                self,
                framebuffer: &TFramebufferId,
            ) -> BoundFramebufferId<Self> {
                unsafe {
                    gl::BindFramebuffer(Self::as_enum(), framebuffer.id());
                }
                BoundFramebufferId {
                    target: self,
                    framebuffer: PhantomData,
                }
            }
        }

        /// Marker trait that can only be implemented on framebuffer
        /// targets that mutably burrow a DrawFramebufferSlot.
        pub trait IsDrawFramebufferTarget<'a>: IsFramebufferTarget {
            fn prove<'b: 'a>(&'b self) -> &'b &'a mut DrawFramebufferSlot;
        }

        /// Marker trait that can only be implemented on framebuffer
        /// targets that mutably burrow a ReadFramebufferSlot.
        pub trait IsReadFramebufferTarget<'a>: IsFramebufferTarget {
            fn prove<'b: 'a>(&'b self) -> &'b &'a mut ReadFramebufferSlot;
        }

        /// Functions available to bound framebuffers with a draw slot.
        pub trait IsDrawableBoundFramebufferId {
            fn draw(&self) -> bool;
        }

        /// Functions available to bound framebuffers with a read slot.
        pub trait IsReadableBoundFramebufferId {
            fn read(&self) -> bool;
        }
    }

    #[must_use]
    pub struct BoundFramebufferId<'a, TFramebufferTarget: 'a + traits::IsFramebufferTarget> {
        target: TFramebufferTarget,
        framebuffer: PhantomData<&'a FramebufferId>,
    }

    impl<'a, TIsDrawFramebufferTarget> traits::IsDrawableBoundFramebufferId
        for BoundFramebufferId<'a, TIsDrawFramebufferTarget>
    where
        TIsDrawFramebufferTarget: 'a + traits::IsDrawFramebufferTarget<'a>,
    {
        fn draw(&self) -> bool {
            false
        }
    }

    impl<'a, TIsReadFramebufferTarget> traits::IsReadableBoundFramebufferId
        for BoundFramebufferId<'a, TIsReadFramebufferTarget>
    where
        TIsReadFramebufferTarget: 'a + traits::IsReadFramebufferTarget<'a>,
    {
        fn read(&self) -> bool {
            false
        }
    }

    #[test]
    fn test() {
        let fb0 = super::DEFAULT_FRAMEBUFFER_ID;
        let fb1 = FramebufferId::new().unwrap();
        let fb2 = FramebufferId::new().unwrap();

        let mut draw = DrawFramebufferSlot {};
        let mut read = ReadFramebufferSlot {};

        use self::traits::*;

        {
            let bb0 = DrawReadFramebufferTarget(&mut draw, &mut read).bind(&fb0);
            assert_eq!(bb0.draw(), false);
            assert_eq!(bb0.read(), false);
        }

        {
            let br0 = ReadFramebufferTarget(&mut read).bind(&fb0);

            // assert_eq!(br0.draw(), false);
            assert_eq!(br0.read(), false);

            {
                let bd1 = DrawFramebufferTarget(&mut draw).bind(&fb1);
                assert_eq!(bd1.draw(), false);
                // assert_eq!(bd1.read(), false);
            }

            {
                let bd2 = DrawFramebufferTarget(&mut draw).bind(&fb2);
                assert_eq!(bd2.draw(), false);
                // assert_eq!(bd2.read(), false);
            }
        }
    }
}
