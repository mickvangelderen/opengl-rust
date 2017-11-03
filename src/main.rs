#![feature(nonzero)]

extern crate gl;
extern crate glutin;
extern crate core;
extern crate jpeg_decoder as jpeg;
extern crate cgmath;

#[macro_use(field_offset)]
extern crate simple_field_offset;

mod glw;

use cgmath::Vector2;
use cgmath::Vector3;
use glutin::GlContext;
use std::path::Path;
use std::io::Read;
use gl::types::*;
use glw::Shader;
use glw::ID;
use std::time;
use std::ptr;
use std::mem;
use std::io;
use std::fs;

macro_rules! c_str {
    ($s:expr) => (
        concat!($s, "\0") as *const str as *const u8 as *const GLchar
    );
}

#[repr(C, packed)]
struct VertexData {
    position: Vector3<GLfloat>,
    color: Vector3<GLfloat>,
    tex_coords: Vector2<GLfloat>,
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

    let vertices: [VertexData; 4] = [
        VertexData {
            position: Vector3::new(-0.5, 0.5, 0.0),
            color: Vector3::new(1.0, 0.0, 0.0),
            tex_coords: Vector2::new(0.0, 1.0),
        },
        VertexData {
            position: Vector3::new(0.5, 0.5, 0.0),
            color: Vector3::new(0.0, 1.0, 0.0),
            tex_coords: Vector2::new(1.0, 1.0),
        },
        VertexData {
            position: Vector3::new(-0.5, -0.5, 0.0),
            color: Vector3::new(0.0, 0.0, 1.0),
            tex_coords: Vector2::new(0.0, 0.0),
        },
        VertexData {
            position: Vector3::new(0.5, -0.5, 0.0),
            color: Vector3::new(0.5, 0.5, 0.5),
            tex_coords: Vector2::new(1.0, 0.0),
        },
    ];

    let indices: [GLuint; 6] = [0, 2, 3, 3, 1, 0];

    let va = glw::VertexArray::new().unwrap();
    let vb = glw::VertexBuffer::new().unwrap();
    let ve = glw::VertexBuffer::new().unwrap();

    let stride = mem::size_of::<VertexData>() as GLint;
    let (position_offset, color_offset, tex_coords_offset) = field_offset!(VertexData, (position, color, tex_coords), *const GLvoid);

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

        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, position_offset);
        gl::EnableVertexAttribArray(0);

        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, stride, color_offset);
        gl::EnableVertexAttribArray(1);

        gl::VertexAttribPointer(2, 2, gl::FLOAT, gl::FALSE, stride, tex_coords_offset);
        gl::EnableVertexAttribArray(2);

        // Unnecessary.
        gl::BindBuffer(gl::ARRAY_BUFFER, 0 as GLuint);

        // Unnecessary.
        gl::BindVertexArray(0 as GLuint);
    }

    let tex_id: GLuint = {
        let file = fs::File::open("assets/bricks-grey.jpg").unwrap();
        let buf_file = io::BufReader::new(file);
        let mut decoder = jpeg::Decoder::new(buf_file);
        let tex_data = decoder.decode().expect("Failed to decode jpeg.");
        let tex_info = decoder.info().unwrap();
        // Flip the texture.
        let tex_data = {
            let w = tex_info.width as usize;
            let h = tex_info.height as usize;
            let mut buffer = Vec::with_capacity(w * h * 3);
            unsafe {
                buffer.set_len(w * h * 3);
            }
            for r in 0..h {
                for c in 0..w {
                    for b in 0..3 {
                        let in_i = (r * w + c) * 3 + b;
                        let out_i = ((h - 1 - r) * w + c) * 3 + b;
                        buffer[out_i] = tex_data[in_i];
                    }
                }
            }
            buffer
        };

        let mut tex_id = 0;
        unsafe {
            gl::GenTextures(1, &mut tex_id);
            gl::BindTexture(gl::TEXTURE_2D, tex_id);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);


            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGB as GLint,
                tex_info.width as GLint,
                tex_info.height as GLint,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                tex_data.as_ptr() as *const GLvoid,
            );
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }

        tex_id
    };

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
    let mut mix_val = 0.5;

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
                            Some(glutin::VirtualKeyCode::Up) => mix_val += 0.1,
                            Some(glutin::VirtualKeyCode::Down) => mix_val -= 0.1,
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

            {
                let loc = gl::GetUniformLocation(program.id().as_uint(), c_str!("mix_val"));
                gl::Uniform1f(loc, mix_val);
            }

            gl::BindVertexArray(va.id().as_uint());

            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
        }

        gl_window.swap_buffers().unwrap();
    }
}

fn file_to_cstring<P: AsRef<Path>>(path: P) -> std::io::Result<std::ffi::CString> {
    let file = fs::File::open(path)?;
    let mut reader = io::BufReader::new(file);
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    let string = std::ffi::CString::new(bytes)?;
    Ok(string)
}
