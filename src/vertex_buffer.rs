extern crate core;
extern crate gl;

use gl::types::*;
use core::nonzero::NonZero;

#[allow(unused)]
#[repr(u32)]
pub enum BufferTarget {
    ArrayBuffer = gl::ARRAY_BUFFER,
    AtomicCounterBuffer = gl::ATOMIC_COUNTER_BUFFER,
    CopyReadBuffer = gl::COPY_READ_BUFFER,
    CopyWriteBuffer = gl::COPY_WRITE_BUFFER,
    DispatchIndirectBuffer = gl::DISPATCH_INDIRECT_BUFFER,
    DrawIndirectBuffer = gl::DRAW_INDIRECT_BUFFER,
    ElementArrayBuffer = gl::ELEMENT_ARRAY_BUFFER,
    PixelPackBuffer = gl::PIXEL_PACK_BUFFER,
    PixelUnpackBuffer = gl::PIXEL_UNPACK_BUFFER,
    QueryBuffer = gl::QUERY_BUFFER,
    ShaderStorageBuffer = gl::SHADER_STORAGE_BUFFER,
    TextureBuffer = gl::TEXTURE_BUFFER,
    TransformFeedbackBuffer = gl::TRANSFORM_FEEDBACK_BUFFER,
    UniformBuffer = gl::UNIFORM_BUFFER,
}

#[derive(Debug)]
pub struct VertexBufferId(NonZero<GLuint>);

impl VertexBufferId {
    pub unsafe fn as_uint(&self) -> GLuint {
        (self.0).get()
    }

    pub fn new() -> Option<Self> {
        NonZero::new(unsafe {
            let mut ids: [GLuint; 1] = [0];
            gl::GenBuffers(ids.len() as GLsizei, ids.as_mut_ptr());
            ids[0]
        }).map(VertexBufferId)
    }

    pub fn bind(&self, target: BufferTarget) {
        unsafe {
            gl::BindBuffer(target as GLenum, self.as_uint());
        }
    }
}

impl Drop for VertexBufferId {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.as_uint());
        }
    }
}

#[derive(Debug)]
pub struct VertexBuffer(VertexBufferId);

impl VertexBuffer {
    pub fn new() -> Result<Self, String> {
        let id = VertexBufferId::new().ok_or_else(|| {
            String::from("Failed to acquire buffer id.")
        })?;
        Ok(VertexBuffer(id))
    }

    pub fn id(&self) -> &VertexBufferId {
        &self.0
    }
}
