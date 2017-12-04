extern crate core;
extern crate gl;

use gl::types::*;
use core::nonzero::NonZero;

pub struct VertexBufferId(NonZero<GLuint>);

impl VertexBufferId {
    pub fn new() -> Option<Self> {
        NonZero::new(unsafe {
            let mut ids: [GLuint; 1] = [0];
            gl::GenBuffers(ids.len() as GLsizei, ids.as_mut_ptr());
            ids[0]
        }).map(VertexBufferId)
    }

    pub unsafe fn as_uint(&self) -> GLuint {
        (self.0).get()
    }
}

impl Drop for VertexBufferId {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.as_uint());
        }
    }
}

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
