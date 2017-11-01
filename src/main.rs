#![feature(nonzero)]

extern crate gl;
extern crate glutin;
extern crate core;

mod glw;

use glutin::GlContext;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use gl::types::*;
use glw::Shader;
use glw::ID;
use std::time;
use std::ptr;
use std::mem;

macro_rules! c_str {
    ($s:expr) => (
        concat!($s, "\0") as *const str as *const u8 as *const GLchar
    );
}

fn main() {
    let mut events_loop = glutin::EventsLoop::new();

    let gl_window = glutin::GlWindow::new(
        glutin::WindowBuilder::new()
            .with_title("rust-opengl")
            .with_dimensions(1024, 768),
        glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (3, 3)))
            .with_gl_profile(glutin::GlProfile::Core),
        &events_loop,
    ).unwrap();

    unsafe {
        gl_window.make_current().unwrap();
    }

    unsafe {
        gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);
        gl::ClearColor(0.5, 0.5, 0.5, 1.0);
    }

    let program = {
        let vertex_src = file_to_cstring("assets/vertex-shader.glsl").unwrap();
        let vertex_shader =
            glw::VertexShader::new(vertex_src).expect("Failed to compile vertex shader.");

        let fragment_src = file_to_cstring("assets/fragment-shader.glsl").unwrap();
        let fragment_shader =
            glw::FragmentShader::new(fragment_src).expect("Failed to compile fragment shader.");

        let program = glw::Program::new().unwrap();

        program.attach(&vertex_shader);
        program.attach(&fragment_shader);
        program.link().expect("Failed to link program.");
        program
    };

    let vertices: [GLfloat; 32] = [
        -0.5,
        0.5,
        0.0,

        1.0,
        0.0,
        0.0,

        // Top left
        0.0,
        1.0,

        0.5,
        0.5,
        0.0,

        0.0,
        1.0,
        0.0,

        // Top right
        1.0,
        1.0,

        -0.5,
        -0.5,
        0.0,

        0.0,
        0.0,
        1.0,

        // Bottom left
        0.0,
        0.0,

        0.5,
        -0.5,
        0.0,

        0.5,
        0.5,
        0.5,

        // Bottom right
        1.0,
        0.0,
    ];

    let indices: [GLuint; 6] = [0, 2, 3, 3, 1, 0];

    let va = glw::VertexArray::new().unwrap();
    let vb = glw::VertexBuffer::new().unwrap();
    let ve = glw::VertexBuffer::new().unwrap();

    let tex_data: [u8; 12] = [
        // Bottom left.
        0, 0, 0,
        // Bottom right.
        255, 255, 255,
        // Top left.
        255, 0, 0,
        // Top right.
        0, 255, 0,
    ];

    let stride = mem::size_of::<[GLfloat; 8]>() as GLint;

    unsafe {
        gl::BindVertexArray(va.id().as_uint());

        gl::BindBuffer(gl::ARRAY_BUFFER, vb.id().as_uint());

        gl::BufferData(
            gl::ARRAY_BUFFER,
            mem::size_of_val(&vertices) as GLsizeiptr,
            vertices.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW,
        );

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ve.id().as_uint());

        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            mem::size_of_val(&indices) as GLsizeiptr,
            indices.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            ptr::null(),
        );

        gl::EnableVertexAttribArray(0);

        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            mem::size_of::<[GLfloat; 3]>() as *const GLvoid,
        );

        gl::EnableVertexAttribArray(1);

        gl::VertexAttribPointer(
            2,
            2,
            gl::FLOAT,
            gl::FALSE,
            stride,
            mem::size_of::<[GLfloat; 6]>() as *const GLvoid,
        );

        gl::EnableVertexAttribArray(2);

        // Unnecessary.
        gl::BindBuffer(gl::ARRAY_BUFFER, 0 as GLuint);

        // Unnecessary.
        gl::BindVertexArray(0 as GLuint);
    }

    let mut tex_id: GLuint = 0;
    unsafe {
        gl::GenTextures(1, &mut tex_id);
        gl::BindTexture(gl::TEXTURE_2D, tex_id);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);

        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGB as GLint, 2, 2, 0, gl::RGB, gl::UNSIGNED_BYTE, tex_data.as_ptr() as *const GLvoid);
        gl::GenerateMipmap(gl::TEXTURE_2D);
    }

    unsafe {
        gl::UseProgram(program.id().as_uint());
        let tex_unif = gl::GetUniformLocation(program.id().as_uint(), c_str!("tex_color"));
        gl::Uniform1i(tex_unif, 0);
    }

    let mut running = true;
    let mut frame_count = 0;
    let mut last_fps_end = time::Instant::now();
    let mut last_frame_end = time::Instant::now();
    let mut green = 0f32;

    while running {

        let now = time::Instant::now();

        // Update FPS.
        frame_count += 1;
        let delta_fps = now.duration_since(last_fps_end);

        if delta_fps >= time::Duration::from_secs(1) {
            last_fps_end = now;
            println!("FPS: {}", frame_count);
            frame_count = 0;
        }

        // Update delta_frame.
        let delta_frame = now.duration_since(last_frame_end);
        last_frame_end = now;
        let delta_frame: f32 = (delta_frame.as_secs() as f32) +
            (delta_frame.subsec_nanos() as f32) * 1e-9;

        // Update background color.
        green = (green + delta_frame) % 1.0;

        // Process events.
        events_loop.poll_events(|event| match event {
            glutin::Event::WindowEvent { event, .. } => {
                match event {
                    glutin::WindowEvent::Closed => running = false,
                    glutin::WindowEvent::Resized(w, h) => {
                        gl_window.resize(w, h);
                        unsafe {
                            gl::Viewport(0, 0, w as i32, h as i32);
                        }
                    }
                    glutin::WindowEvent::KeyboardInput { input, .. } => {
                        match input.virtual_keycode {
                            Some(glutin::VirtualKeyCode::Escape) => running = false,
                            _ => (),
                        }
                    }
                    _ => (),
                }
            }
            glutin::Event::DeviceEvent { .. } => {}
            _ => (),
        });

        // Render.
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, tex_id);

            gl::UseProgram(program.id().as_uint());

            gl::BindVertexArray(va.id().as_uint());

            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
        }

        gl_window.swap_buffers().unwrap();
    }
}

fn file_to_cstring<P: AsRef<Path>>(path: P) -> std::io::Result<std::ffi::CString> {
    let file = File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    let string = std::ffi::CString::new(bytes)?;
    Ok(string)
}
