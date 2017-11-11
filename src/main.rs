#![feature(nonzero)]

extern crate gl;
extern crate glutin;
extern crate core;
extern crate jpeg_decoder as jpeg;
extern crate cgmath;

#[macro_use(field_offset)]
extern crate simple_field_offset;

pub mod glw;
pub mod shader;
pub mod program;

use cgmath::prelude::*;
use cgmath::*;
use glutin::GlContext;
use std::path::Path;
use std::io::Read;
use gl::types::*;
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

fn duration_to_seconds(duration: time::Duration) -> f64 {
    let seconds = duration.as_secs() as f64;
    let nanoseconds = duration.subsec_nanos() as f64;
    seconds + nanoseconds * 1e-9
}

fn main() {
    const INITIAL_WIDTH: u32 = 1024;
    const INITIAL_HEIGHT: u32 = 768;
    const INITIAL_FOVY: Rad<GLfloat> = Rad(45.0 * 3.1415 / 180.0);
    const INITIAL_NEAR: GLfloat = 1.0;
    const INITIAL_FAR: GLfloat = 100.0;

    let mut events_loop = glutin::EventsLoop::new();

    let gl_window = glutin::GlWindow::new(
        glutin::WindowBuilder::new()
            .with_title("rust-opengl")
            .with_dimensions(INITIAL_WIDTH, INITIAL_HEIGHT),
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
        let vertex_shader = shader::specialization::VertexShaderId::new()
            .expect("Failed to acquire vertex shader id.")
            .compile(vertex_src)
            .expect("Failed to compile vertex shader.");

        let fragment_src = file_to_cstring("assets/fragment-shader.glsl").unwrap();
        let fragment_shader = shader::specialization::FragmentShaderId::new()
            .expect("Failed to acquire fragment shader id.")
            .compile(fragment_src)
            .expect("Failed to compile fragment shader.");

        let program = program::ProgramId::new()
            .expect("Failed to acquire program id.");

        program.attach(&vertex_shader);
        program.attach(&fragment_shader);
        program.link().expect("Failed to link program.")
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

    program.use_program();

    unsafe {
        let tex_unif = gl::GetUniformLocation(program.as_uint(), c_str!("tex_color"));
        gl::Uniform1i(tex_unif, 0);
    }

    let start = time::Instant::now();
    let mut running = true;
    let mut frame_count = 0;
    let mut last_fps_end = start;
    let mut last_frame_end = start;

    let mut move_up = false;
    let mut move_down = false;
    let mut move_left = false;
    let mut move_right = false;
    let mut move_forward = false;
    let mut move_backward = false;

    let mut camera_transform: Decomposed<Vector3<GLfloat>, Quaternion<GLfloat>> = Decomposed {
        scale: 1.0,
        disp: Vector3 { x: 0.0, y: 0.0, z: 10.0 },
        rot: Quaternion::one(),
    };

    let mut projection_transform = Matrix4::from(PerspectiveFov {
        fovy: INITIAL_FOVY,
        aspect: (INITIAL_WIDTH as GLfloat) / (INITIAL_HEIGHT as GLfloat),
        near: INITIAL_NEAR,
        far: INITIAL_FAR,
    });

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
        let delta_frame = duration_to_seconds(now.duration_since(last_frame_end)) as f32;
        last_frame_end = now;

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

                        projection_transform = Matrix4::from(PerspectiveFov {
                            fovy: INITIAL_FOVY,
                            aspect: (w as GLfloat) / (h as GLfloat),
                            near: INITIAL_NEAR,
                            far: INITIAL_FAR,
                        });
                    }

                    glutin::WindowEvent::KeyboardInput { input, .. } => {
                        let pressed = if let glutin::ElementState::Pressed = input.state {
                            true
                        } else {
                            false
                        };

                        match input.virtual_keycode {
                            Some(glutin::VirtualKeyCode::Escape) => running = false,
                            Some(glutin::VirtualKeyCode::W) => move_forward = pressed,
                            Some(glutin::VirtualKeyCode::S) => move_backward = pressed,
                            Some(glutin::VirtualKeyCode::A) => move_left = pressed,
                            Some(glutin::VirtualKeyCode::D) => move_right = pressed,
                            Some(glutin::VirtualKeyCode::Q) => move_up = pressed,
                            Some(glutin::VirtualKeyCode::Z) => move_down = pressed,
                            _ => (),
                        }
                    }
                    _ => (),
                }
            }
            glutin::Event::DeviceEvent { .. } => {}
            _ => (),
        });

        camera_transform.disp += Vector3 {
            x: move_right as u32 as GLfloat - move_left as u32 as GLfloat,
            y: move_up as u32 as GLfloat - move_down as u32 as GLfloat,
            z: move_backward as u32 as GLfloat - move_forward as u32 as GLfloat,
        } * delta_frame as f32;

        let angle = {
            let s = duration_to_seconds(now.duration_since(start));
            let rps = 1.0/16.0;
            Rad((s*std::f64::consts::PI*2.0*rps) as f32)
        };
        let x_shift = f32::cos(angle.0);
        let y_shift = f32::sin(angle.0);
        let model_transform = Matrix4::from_translation(Vector3::new(x_shift, y_shift, 0.0));

        // Render.
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, tex_id);

            program.use_program();

            {
                let loc = gl::GetUniformLocation(program.as_uint(), c_str!("model"));
                gl::UniformMatrix4fv(loc, 1, gl::FALSE, model_transform.as_ptr());
            }

            {
                let loc = gl::GetUniformLocation(program.as_uint(), c_str!("view"));
                let inverse_camera_transform = Matrix4::from(camera_transform.inverse_transform().unwrap());
                gl::UniformMatrix4fv(loc, 1, gl::FALSE, inverse_camera_transform.as_ptr());
            }

            {
                let loc = gl::GetUniformLocation(program.as_uint(), c_str!("projection"));
                gl::UniformMatrix4fv(loc, 1, gl::FALSE, projection_transform.as_ptr());
            }

            gl::BindVertexArray(va.id().as_uint());
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());

            {
                let loc = gl::GetUniformLocation(program.as_uint(), c_str!("model"));
                let model_transform: Matrix4<GLfloat> =
                    Matrix4::from_translation(Vector3::new(0.5, 0.0, 0.0)) *
                        Matrix4::from(Quaternion::from(
                            Euler::new(Rad::zero(), Rad::zero(), angle),
                        ));
                gl::UniformMatrix4fv(loc, 1, gl::FALSE, model_transform.as_ptr());
            }

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
