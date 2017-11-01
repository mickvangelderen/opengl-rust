extern crate core;
extern crate gl;

use gl::types::*;
use std::ffi;
use std::ptr;
use core::nonzero::NonZero;

type ValidID = NonZero<GLuint>;

pub trait ID {
    unsafe fn as_uint(&self) -> GLuint;
}

pub struct VertexBufferID(ValidID);

impl VertexBufferID {
    pub fn new() -> Option<Self> {
        NonZero::new(unsafe {
            let mut id: GLuint = 0;
            gl::GenBuffers(1, &mut id);
            id
        }).map(VertexBufferID)
    }
}

impl ID for VertexBufferID {
    unsafe fn as_uint(&self) -> GLuint {
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

    pub fn bind(&self) -> () {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.id().as_uint());
            // FIXME: Check for errors.
        }
    }
}

pub struct ProgramID(ValidID);

impl ProgramID {
    pub fn new() -> Option<Self> {
        NonZero::new(unsafe { gl::CreateProgram() }).map(ProgramID)
    }
}

impl ID for ProgramID {
    unsafe fn as_uint(&self) -> GLuint {
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

    pub fn attach<T: Shader>(&self, shader: &T) {
        unsafe {
            gl::AttachShader(self.id().as_uint(), shader.id().as_uint());
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

pub trait Shader {
    type ShaderID: ID;

    fn new(source: ffi::CString) -> Result<Self, String>
    where
        Self: Sized;

    fn id(&self) -> &Self::ShaderID;
}

macro_rules! impl_shader_type {
    ($XShaderID:ident, $XShader:ident, $SHADER_TYPE:path) => (
        pub struct $XShaderID(ValidID);

        impl $XShaderID {
            pub fn new() -> Option<Self> {
                NonZero::new(unsafe { gl::CreateShader($SHADER_TYPE) })
                    .map($XShaderID)
            }
        }

        impl ID for $XShaderID {
            unsafe fn as_uint(&self) -> GLuint {
                (self.0).get()
            }
        }

        impl Drop for $XShaderID {
            fn drop(&mut self) {
                unsafe { gl::DeleteShader((self.0).get()) };
            }
        }

        pub struct $XShader($XShaderID);

        impl Shader for $XShader {
            type ShaderID = $XShaderID;

            fn new(source: ffi::CString) -> Result<Self, String> {

                let id = $XShaderID::new().ok_or_else(
                    || String::from("Failed to create shader object.")
                )?;

                unsafe {
                    gl::ShaderSource(id.as_uint(), 1, &source.as_ptr(), ptr::null());
                }

                unsafe {
                    gl::CompileShader(id.as_uint());
                }

                let mut status = gl::FALSE as GLint;

                unsafe {
                    gl::GetShaderiv(id.as_uint(), gl::COMPILE_STATUS, &mut status);
                }

                if status != (gl::TRUE as GLint) {
                    let mut len = 0;

                    unsafe {
                        gl::GetShaderiv(id.as_uint(), gl::INFO_LOG_LENGTH, &mut len);
                    }

                    let mut buf = Vec::with_capacity(len as usize);

                    unsafe {
                        buf.set_len((len as usize) - 1);
                    }

                    unsafe {
                        gl::GetShaderInfoLog(
                            id.as_uint(),
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

            fn id(&self) -> &Self::ShaderID {
                &self.0
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
