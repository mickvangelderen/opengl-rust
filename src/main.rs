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

    let vertices: [GLfloat; 9] = [-0.5, -0.5, 0.0, 0.5, -0.5, 0.0, 0.0, 0.7, 0.0];

    let vb = glw::VertexBuffer::new().unwrap();

    vb.bind();

    unsafe {
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<GLfloat>()) as GLsizeiptr,
            std::mem::transmute(&vertices[0]),
            gl::STATIC_DRAW,
        );
    }

    let program = {
        let vertex_src = file_to_cstring("assets/vertex-shader.glsl").unwrap();
        let vertex_shader =
            glw::VertexShader::new(vertex_src).expect("Failed to compile vertex shader.");

        let fragment_src = file_to_cstring("assets/fragment-shader.glsl").unwrap();
        let fragment_shader =
            glw::VertexShader::new(fragment_src).expect("Failed to compile fragment shader.");

        let program = glw::Program::new().unwrap();

        program.attach(&vertex_shader);
        program.attach(&fragment_shader);
        program.link().unwrap();
        program
    };

    unsafe {
        gl::UseProgram(program.id().as_uint())
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
        let delta_frame: f32 = (delta_frame.as_secs() as f32) + (delta_frame.subsec_nanos() as f32)*1e-9;

        // Updates background color.
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

        unsafe {
            gl::ClearColor(0.0, green, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
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
