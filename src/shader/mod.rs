extern crate gl;
extern crate core;

pub mod specialization;

use core::nonzero::NonZero;
use gl::types::*;
use std::ffi::CStr;
use std::ptr;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub enum ShaderKind {
    Compute = gl::COMPUTE_SHADER as isize,
    Fragment = gl::FRAGMENT_SHADER as isize,
    Geometry = gl::GEOMETRY_SHADER as isize,
    Vertex = gl::VERTEX_SHADER as isize,
    TesselationControl = gl::TESS_CONTROL_SHADER as isize,
    TesselationEvaluation = gl::TESS_EVALUATION_SHADER as isize,
}

#[derive(Debug)]
pub struct ShaderId(NonZero<GLuint>);

impl ShaderId {
    pub fn new(kind: ShaderKind) -> Option<Self> {
        NonZero::new(unsafe { gl::CreateShader(kind as GLenum) }).map(ShaderId)
    }

    #[inline]
    pub unsafe fn as_uint(&self) -> GLuint {
        self.0.get()
    }

    pub fn compile<T: AsRef<CStr>>(self, source: T) -> Result<CompiledShaderId, String> {
        unsafe {
            gl::ShaderSource(self.as_uint(), 1, &source.as_ref().as_ptr(), ptr::null());
            gl::CompileShader(self.as_uint());
        }

        let mut status = gl::FALSE as GLint;

        unsafe {
            gl::GetShaderiv(self.as_uint(), gl::COMPILE_STATUS, &mut status);
        }

        if status != (gl::TRUE as GLint) {
            let mut len = 0;

            unsafe {
                gl::GetShaderiv(self.as_uint(), gl::INFO_LOG_LENGTH, &mut len);
            }

            let mut buf = Vec::with_capacity(len as usize);

            unsafe {
                buf.set_len((len as usize) - 1);
            }

            unsafe {
                gl::GetShaderInfoLog(
                    self.as_uint(),
                    len,
                    ptr::null_mut(),
                    buf.as_mut_ptr() as *mut GLchar
                );
            }

            Err(String::from_utf8(buf).expect("Shader info log is not utf8"))
        } else {
            Ok(CompiledShaderId(self))
        }
    }
}

impl Drop for ShaderId {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.as_uint());
        }
    }
}

impl AsRef<Self> for ShaderId {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

#[derive(Debug)]
pub struct CompiledShaderId(ShaderId);

impl CompiledShaderId {
    #[inline]
    pub unsafe fn as_uint(&self) -> GLuint {
        self.0.as_uint()
    }
}

impl AsRef<Self> for CompiledShaderId {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

#[cfg(test)]
mod tests {
    // TODO: Add tests.
}
