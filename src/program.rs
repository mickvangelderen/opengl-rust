extern crate gl;
extern crate core;

use core::nonzero::NonZero;
use gl::types::{GLuint, GLint, GLchar};
use std::ptr;

use super::shader;

pub struct ProgramId(NonZero<GLuint>);

impl ProgramId {
    pub fn new() -> Option<Self> {
        NonZero::new(unsafe { gl::CreateProgram() }).map(ProgramId)
    }

    pub fn attach<T: AsRef<shader::CompiledShaderId>>(&self, shader_id: T) {
        unsafe {
            gl::AttachShader(self.as_uint(), shader_id.as_ref().as_uint());
        }
    }

    pub fn link(self) -> Result<LinkedProgramId, String> {
        unsafe {
            gl::LinkProgram(self.as_uint());
        }

        let mut status = gl::FALSE as GLint;

        unsafe {
            gl::GetProgramiv(self.as_uint(), gl::LINK_STATUS, &mut status);
        }

        if status != (gl::TRUE as GLint) {
            let mut len = 0;

            unsafe {
                gl::GetProgramiv(self.as_uint(), gl::INFO_LOG_LENGTH, &mut len);
            }

            let mut buf = Vec::with_capacity(len as usize);

            unsafe {
                buf.set_len((len as usize) - 1);
            }

            unsafe {
                gl::GetProgramInfoLog(
                    self.as_uint(),
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar,
                );
            }

            Err(String::from_utf8(buf).expect(
                "Program info log is not utf8",
            ))
        } else {
            Ok(LinkedProgramId(self))
        }
    }

    pub unsafe fn as_uint(&self) -> GLuint {
        (self.0).get()
    }
}

impl Drop for ProgramId {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.as_uint());
        }
    }
}

pub struct LinkedProgramId(ProgramId);

impl LinkedProgramId {
    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.as_uint());
        }
    }

    pub unsafe fn as_uint(&self) -> GLuint {
        self.0.as_uint()
    }
}
