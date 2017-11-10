extern crate core;
extern crate gl;

use super::shader;
use gl::types::*;
use std::ptr;
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

pub struct ProgramID(ValidID);

impl ProgramID {
    pub fn new() -> Option<Self> {
        NonZero::new(unsafe { gl::CreateProgram() }).map(ProgramID)
    }

    pub unsafe fn as_uint(&self) -> GLuint {
        (self.0).get()
    }
}

impl Drop for ProgramID {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.as_uint());
        }
    }
}

pub struct Program(ProgramID);

impl Program {
    pub fn new() -> Result<Self, String> {
        ProgramID::new().map(Program).ok_or_else(|| {
            String::from("Failed to acquire program id")
        })
    }

    pub fn id(&self) -> &ProgramID {
        &self.0
    }

    pub fn attach<T: AsRef<shader::CompiledShaderId>>(&self, shader_id: T) {
        unsafe {
            gl::AttachShader(self.id().as_uint(), shader_id.as_ref().as_uint());
        }
    }

    pub fn link(&self) -> Result<(), String> {
        unsafe {
            gl::LinkProgram(self.id().as_uint());
        }

        let mut status = gl::FALSE as GLint;

        unsafe {
            gl::GetProgramiv(self.id().as_uint(), gl::LINK_STATUS, &mut status);
        }

        if status != (gl::TRUE as GLint) {
            let mut len = 0;

            unsafe {
                gl::GetProgramiv(self.id().as_uint(), gl::INFO_LOG_LENGTH, &mut len);
            }

            let mut buf = Vec::with_capacity(len as usize);

            unsafe {
                buf.set_len((len as usize) - 1);
            }

            unsafe {
                gl::GetProgramInfoLog(
                    self.id().as_uint(),
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar,
                );
            }

            return Err(String::from_utf8(buf).expect(
                "Program info log is not utf8",
            ));
        }

        Ok(())
    }
}

