extern crate core;
extern crate gl;

use gl::types::*;
use std::ffi;
use std::ptr;
use core::nonzero::NonZero;

pub trait Shader {
    fn new(source: ffi::CString) -> Result<Self, String>
    where
        Self: Sized;

    unsafe fn id(&self) -> NonZero<GLuint>;
}

macro_rules! impl_shader_type {
    ($XShaderID:ident, $XShader:ident, $SHADER_TYPE:path) => (
        pub struct $XShaderID(NonZero<GLuint>);

        impl $XShaderID {
            pub fn new() -> Option<Self> {
                NonZero::new(unsafe { gl::CreateShader($SHADER_TYPE) })
                    .map($XShaderID)
            }

            pub unsafe fn get(&self) -> NonZero<GLuint> {
                self.0
            }
        }

        impl Drop for $XShaderID {
            fn drop(&mut self) {
                unsafe { gl::DeleteShader((self.0).get()) };
            }
        }

        pub struct $XShader($XShaderID);

        impl Shader for $XShader {
            fn new(source: ffi::CString) -> Result<Self, String> {

                let id = $XShaderID::new().ok_or_else(
                    || String::from("Failed to create shader object.")
                )?;

                unsafe {
                    gl::ShaderSource(id.get().get(), 1, &source.as_ptr(), ptr::null());
                }

                unsafe {
                    gl::CompileShader(id.get().get());
                }

                let mut status = gl::FALSE as GLint;

                unsafe {
                    gl::GetShaderiv(id.get().get(), gl::COMPILE_STATUS, &mut status);
                }

                if status != (gl::TRUE as GLint) {
                    let mut len = 0;

                    unsafe {
                        gl::GetShaderiv(id.get().get(), gl::INFO_LOG_LENGTH, &mut len);
                    }

                    let mut buf = Vec::with_capacity(len as usize);

                    unsafe {
                        buf.set_len((len as usize) - 1);
                    }

                    unsafe {
                        gl::GetShaderInfoLog(
                            id.get().get(),
                            len,
                            ptr::null_mut(),
                            buf.as_mut_ptr() as *mut GLchar
                        );
                    }

                    Err(String::from_utf8(buf).expect("Shader info log is not utf8"))
                } else {
                    Ok($XShader(id))
                }
            }

            unsafe fn id(&self) -> NonZero<GLuint> {
                (self.0).get()
            }
        }
    )
}

impl_shader_type!(ComputeShaderID, ComputeShader, gl::COMPUTE_SHADER);
impl_shader_type!(FragmentShaderID, FragmentShader, gl::FRAGMENT_SHADER);
impl_shader_type!(GeometryShaderID, GeometryShader, gl::GEOMETRY_SHADER);
impl_shader_type!(
    TesselationControlShaderID,
    TesselationControlShader,
    gl::TESS_CONTROL_SHADER
);
impl_shader_type!(
    TesselationEvaluationShaderID,
    TesselationEvaluationShader,
    gl::TESS_EVALUATION_SHADER
);
impl_shader_type!(VertexShaderID, VertexShader, gl::VERTEX_SHADER);
