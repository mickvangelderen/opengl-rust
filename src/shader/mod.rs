extern crate gl;
extern crate core;

pub mod specialization;

use core::nonzero::NonZero;
use gl::types::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ShaderKind {
    Compute = gl::COMPUTE_SHADER,
    Fragment = gl::FRAGMENT_SHADER,
    Geometry = gl::GEOMETRY_SHADER,
    Vertex = gl::VERTEX_SHADER,
    TesselationControl = gl::TESS_CONTROL_SHADER,
    TesselationEvaluation = gl::TESS_EVALUATION_SHADER,
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

    pub fn compile<T: AsRef<str>>(self, sources: &[T]) -> Result<CompiledShaderId, String> {
        let source_ptrs: Vec<*const GLchar> = sources
            .iter()
            .map(|source| source.as_ref().as_ptr() as *const GLchar)
            .collect();
        let source_lens: Vec<GLint> = sources
            .iter()
            .map(|source| source.as_ref().len() as GLint)
            .collect();

        unsafe {
            gl::ShaderSource(
                self.as_uint(),
                sources.len() as GLint,
                source_ptrs.as_ptr(),
                source_lens.as_ptr(),
            );
            gl::CompileShader(self.as_uint());
        }

        let status = unsafe {
            let mut status = gl::FALSE as GLint;
            gl::GetShaderiv(self.as_uint(), gl::COMPILE_STATUS, &mut status);
            status
        };

        if status != (gl::TRUE as GLint) {
            let capacity = unsafe {
                let mut capacity: GLint = 0;
                gl::GetShaderiv(self.as_uint(), gl::INFO_LOG_LENGTH, &mut capacity);
                assert!(capacity >= 0);
                capacity
            };

            let buffer = unsafe {
                let mut buffer: Vec<u8> = Vec::with_capacity(capacity as usize);
                let mut length: GLint = 0;
                gl::GetShaderInfoLog(
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
                "Shader info log is not utf8",
            ))
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

#[derive(Debug)]
pub struct CompiledShaderId(ShaderId);

impl CompiledShaderId {
    #[inline]
    pub unsafe fn as_uint(&self) -> GLuint {
        self.0.as_uint()
    }
}

impl AsRef<ShaderId> for CompiledShaderId {
    #[inline]
    fn as_ref(&self) -> &ShaderId {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    // TODO: Add tests.
}
