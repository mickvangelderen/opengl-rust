extern crate core;
extern crate gl;

use gl::types::*;
use core::nonzero::NonZero;

type ValidID = NonZero<GLuint>;

pub struct VertexBufferID(ValidID);

impl VertexBufferID {
    pub fn new() -> Option<Self> {
        NonZero::new(unsafe {
            let mut id: GLuint = 0; // mem::uninitialized() fails if id is never assigned to.
            gl::GenBuffers(1, &mut id);
            id
        }).map(VertexBufferID)
    }

    pub unsafe fn as_uint(&self) -> GLuint {
        (self.0).get()
    }
}

impl Drop for VertexBufferID {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.as_uint());
        }
    }
}

pub struct VertexBuffer(VertexBufferID);

impl VertexBuffer {
    pub fn new() -> Result<Self, String> {
        let id = VertexBufferID::new().ok_or_else(|| {
            String::from("Failed to acquire buffer id.")
        })?;
        Ok(VertexBuffer(id))
    }

    pub fn id(&self) -> &VertexBufferID {
        &self.0
    }
}

pub struct VertexArrayID(ValidID);

impl VertexArrayID {
    pub fn new() -> Option<Self> {
        NonZero::new(unsafe {
            let mut id: GLuint = 0; // mem::uninitialized() fails if id is never assigned to.
            gl::GenVertexArrays(1, &mut id);
            id
        }).map(VertexArrayID)
    }

    pub unsafe fn as_uint(&self) -> GLuint {
        (self.0).get()
    }
}

impl Drop for VertexArrayID {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.as_uint());
        }
    }
}

pub struct VertexArray(VertexArrayID);

impl VertexArray {
    pub fn new() -> Result<Self, String> {
        VertexArrayID::new().map(VertexArray).ok_or_else(|| {
            String::from("Failed to acquire vertex array id.")
        })
    }

    pub fn id(&self) -> &VertexArrayID {
        &self.0
    }
}
