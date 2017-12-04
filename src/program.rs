extern crate gl;
extern crate core;

use core::nonzero::NonZero;
use gl::types::{GLuint, GLint, GLchar, GLsizei};

use super::shader::CompiledShaderId;

#[derive(Debug)]
pub struct ProgramId(NonZero<GLuint>);

impl ProgramId {
    pub fn new() -> Option<Self> {
        NonZero::new(unsafe { gl::CreateProgram() }).map(ProgramId)
    }

    pub unsafe fn as_uint(&self) -> GLuint {
        (self.0).get()
    }

    pub fn link(self, shaders: &[&CompiledShaderId]) -> Result<LinkedProgramId, String> {
        for shader in shaders {
            unsafe {
                gl::AttachShader(self.as_uint(), shader.as_uint());
            }
        }

        unsafe {
            gl::LinkProgram(self.as_uint());
        }

        let status = unsafe {
            let mut status = gl::FALSE as GLint;
            gl::GetProgramiv(self.as_uint(), gl::LINK_STATUS, &mut status);
            status
        };

        if status == (gl::TRUE as GLint) {
            Ok(LinkedProgramId(self))
        } else {
            let capacity = unsafe {
                let mut capacity: GLint = 0;
                gl::GetProgramiv(self.as_uint(), gl::INFO_LOG_LENGTH, &mut capacity);
                assert!(capacity >= 0);
                capacity
            };

            let buffer = unsafe {
                let mut buffer: Vec<u8> = Vec::with_capacity(capacity as usize);
                let mut length: GLsizei = 0;
                gl::GetProgramInfoLog(
                    self.as_uint(),
                    capacity,
                    &mut length,
                    buffer.as_mut_ptr() as *mut GLchar,
                );
                assert!(length >= 0 && length <= capacity);
                buffer.set_len(length as usize);
                buffer
            };

            Err(String::from_utf8(buffer).expect(
                "Program info log is not utf8",
            ))
        }
    }
}

impl Drop for ProgramId {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.as_uint());
        }
    }
}

#[derive(Debug)]
pub struct LinkedProgramId(ProgramId);

impl LinkedProgramId {
    pub unsafe fn as_uint(&self) -> GLuint {
        self.0.as_uint()
    }

    pub fn bind(&self) {
        unsafe { gl::UseProgram(self.as_uint()) }
    }
}
