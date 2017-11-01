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
        gl::ClearColor(0.0, 1.0, 0.0, 1.0);
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

    let vertices: [GLfloat; 12] = [
        -0.5,
        0.5,
        0.0,
        0.5,
        0.5,
        0.0,
        -0.5,
        -0.5,
        0.0,
        0.5,
        -0.5,
        0.0,
    ];

    let indices: [GLuint; 6] = [0, 2, 3, 3, 1, 0];

    let va = glw::VertexArray::new().unwrap();
    let vb = glw::VertexBuffer::new().unwrap();
    let ve = glw::VertexBuffer::new().unwrap();

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
            mem::size_of::<[GLfloat; 3]>() as GLint,
            ptr::null(),
        );

        gl::EnableVertexAttribArray(0);

        // Unnecessary.
        gl::BindBuffer(gl::ARRAY_BUFFER, 0 as GLuint);

        // Unnecessary.
        gl::BindVertexArray(0 as GLuint);
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
            gl::ClearColor(0.0, green, 0.0, 1.0);

            gl::Clear(gl::COLOR_BUFFER_BIT);

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
