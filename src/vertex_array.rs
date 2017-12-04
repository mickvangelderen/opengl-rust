extern crate core;
extern crate gl;

use gl::types::*;
use core::nonzero::NonZero;

pub struct VertexArrayId(NonZero<GLuint>);

impl VertexArrayId {
    pub fn new() -> Option<Self> {
        NonZero::new(unsafe {
            let mut ids: [GLuint; 1] = [0];
            gl::GenVertexArrays(ids.len() as GLsizei, ids.as_mut_ptr());
            ids[0]
        }).map(VertexArrayId)
    }

    pub unsafe fn as_uint(&self) -> GLuint {
        (self.0).get()
    }
}

impl Drop for VertexArrayId {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.as_uint());
        }
    }
}

pub struct VertexArray(VertexArrayId);

impl VertexArray {
    pub fn new() -> Result<Self, String> {
        VertexArrayId::new().map(VertexArray).ok_or_else(|| {
            String::from("Failed to acquire vertex array id.")
        })
    }

    pub fn id(&self) -> &VertexArrayId {
        &self.0
    }
}
