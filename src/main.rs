#![feature(nonzero)]
#![feature(stmt_expr_attributes)]

extern crate gl;
extern crate glutin;
extern crate core;
extern crate jpeg_decoder as jpeg;
extern crate cgmath;

#[macro_use(field_offset)]
extern crate simple_field_offset;

pub mod shader;
pub mod program;
pub mod import;
pub mod palette;
pub mod texture;
pub mod vertex_buffer;
pub mod vertex_array;
pub mod viewport;

use vertex_buffer::VertexBuffer;
use vertex_array::VertexArray;

use cgmath::prelude::*;
use cgmath::*;
use glutin::GlContext;
use std::path::Path;
use std::io::Read;
use gl::types::*;
use std::time;
use std::mem;
use std::io;
use std::fs;

macro_rules! c_str {
    ($s:expr) => (
        concat!($s, "\0") as *const str as *const u8 as *const GLchar
    );
}

#[allow(unused_macros)]
macro_rules! print_expr {
    ($e:expr) => {
        println!("{}: {:#?}", stringify!($e), $e)
    }
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
    const INITIAL_NEAR: GLfloat = 0.1;
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
        gl::Enable(gl::DEPTH_TEST);
    }

    let program = {
        let vertex_src = file_to_string("assets/standard.vert").unwrap();
        let vertex_shader = shader::specialization::VertexShaderId::new()
            .expect("Failed to acquire vertex shader id.")
            .compile(&[vertex_src])
            .expect("Failed to compile vertex shader.");

        let fragment_src = file_to_string("assets/standard.frag").unwrap();
        let fragment_shader = shader::specialization::FragmentShaderId::new()
            .expect("Failed to acquire fragment shader id.")
            .compile(&[fragment_src])
            .expect("Failed to compile fragment shader.");

        let program = program::ProgramId::new().expect("Failed to acquire program id.");

        program.attach(&vertex_shader);
        program.attach(&fragment_shader);
        program.link().expect("Failed to link program.")
    };

    let mesh = import::import_obj("assets/monster.obj").expect("Failed to import monster.obj");

    let va = VertexArray::new().unwrap();
    let vb = VertexBuffer::new().unwrap();
    let ve = VertexBuffer::new().unwrap();

    unsafe {
        gl::BindVertexArray(va.id().as_uint());

        gl::BindBuffer(gl::ARRAY_BUFFER, vb.id().as_uint());

        gl::BufferData(
            gl::ARRAY_BUFFER,
            mem::size_of_val(&mesh.elements[..]) as GLsizeiptr,
            mesh.elements.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<import::VertexData>() as GLsizei,
            field_offset!(import::VertexData, vertex_position) as *const GLvoid,
        );
        gl::EnableVertexAttribArray(0);

        gl::VertexAttribPointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<import::VertexData>() as GLsizei,
            field_offset!(import::VertexData, texture_position) as *const GLvoid,
        );
        gl::EnableVertexAttribArray(1);

        gl::VertexAttribPointer(
            2,
            3,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<import::VertexData>() as GLsizei,
            field_offset!(import::VertexData, vertex_normal) as *const GLvoid,
        );
        gl::EnableVertexAttribArray(2);

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ve.id().as_uint());

        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            mem::size_of_val(&mesh.indices[..]) as GLsizeiptr,
            mesh.indices.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW,
        );

        // This would be a mistake.
        // gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0 as GLuint);

        // Unnecessary.
        gl::BindBuffer(gl::ARRAY_BUFFER, 0 as GLuint);

        // Unnecessary.
        gl::BindVertexArray(0 as GLuint);
    }

    let tex_id: texture::TextureId = {
        let file = fs::File::open("assets/monster-diffuse.jpg").unwrap();
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

        let mut tex_id = texture::TextureId::new().unwrap();
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, tex_id.as_uint());

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MIN_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as GLint,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_MAG_FILTER,
                gl::LINEAR_MIPMAP_LINEAR as GLint,
            );

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

    // Set up texture location for program.
    program.use_program();
    unsafe {
        let loc = gl::GetUniformLocation(program.as_uint(), c_str!("tex_color"));
        gl::Uniform1i(loc, 0);
    }

    let light_program = {
        let vertex_shader = shader::specialization::VertexShaderId::new()
            .unwrap()
            .compile(&[file_to_string("assets/light.vert").unwrap()])
            .unwrap();
        let fragment_shader = shader::specialization::FragmentShaderId::new()
            .unwrap()
            .compile(&[file_to_string("assets/light.frag").unwrap()])
            .unwrap();
        let program = program::ProgramId::new().unwrap();
        program.attach(&vertex_shader);
        program.attach(&fragment_shader);
        program.link().unwrap()
    };

    let light_mesh = import::import_obj("assets/icosphere-80.obj").expect("Failed to import obj");

    let light_vertex_array = VertexArray::new().unwrap();
    let light_vertex_buffer = VertexBuffer::new().unwrap();
    let light_elements_buffer = VertexBuffer::new().unwrap();

    unsafe {
        gl::BindVertexArray(light_vertex_array.id().as_uint());

        gl::BindBuffer(gl::ARRAY_BUFFER, light_vertex_buffer.id().as_uint());

        gl::BufferData(
            gl::ARRAY_BUFFER,
            mem::size_of_val(&light_mesh.elements[..]) as GLsizeiptr,
            light_mesh.elements.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<import::VertexData>() as GLsizei,
            field_offset!(import::VertexData, vertex_position) as *const GLvoid,
        );
        gl::EnableVertexAttribArray(0);

        gl::BindBuffer(
            gl::ELEMENT_ARRAY_BUFFER,
            light_elements_buffer.id().as_uint(),
        );

        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            mem::size_of_val(&light_mesh.indices[..]) as GLsizeiptr,
            light_mesh.indices.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW,
        );
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

    let mut camera_pos: Vector3<GLfloat> = Vector3::new(0.0, 4.0, 10.0);
    let mut camera_pitch: Rad<GLfloat> = Rad(0.0);
    let mut camera_yaw: Rad<GLfloat> = Rad(0.0);
    let mut camera_fov = INITIAL_FOVY;
    let mut camera_aspect = (INITIAL_WIDTH as GLfloat) / (INITIAL_HEIGHT as GLfloat);

    while running {

        let now = time::Instant::now();

        let delta_start = duration_to_seconds(now.duration_since(start)) as f32;

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

        let mut mouse_dx: f32 = 0.0;
        let mut mouse_dy: f32 = 0.0;
        let mut mouse_dscroll: f32 = 0.0;

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

                        camera_aspect = (w as GLfloat) / (h as GLfloat);
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
            glutin::Event::DeviceEvent { device_id, event, .. } => {
                match event {
                    glutin::DeviceEvent::Added => println!("Added device {:?}", device_id),
                    glutin::DeviceEvent::Removed => println!("Removed device {:?}", device_id),
                    glutin::DeviceEvent::Motion { axis, value } => {
                        match axis {
                            0 => mouse_dx += value as f32,
                            1 => mouse_dy += value as f32,
                            3 => mouse_dscroll += value as f32,
                            _ => (),
                        }
                    }
                    _ => (),
                }
            }
            _ => (),
        });

        let camera_dpos = Quaternion::from_axis_angle(Vector3::unit_y(), -camera_yaw) *
            Vector3 {
                x: move_right as u32 as GLfloat - move_left as u32 as GLfloat,
                y: move_up as u32 as GLfloat - move_down as u32 as GLfloat,
                z: move_backward as u32 as GLfloat - move_forward as u32 as GLfloat,
            };
        let camera_positional_velocity: GLfloat = 2.0;
        camera_pos += camera_positional_velocity * (delta_frame as f32) * camera_dpos;

        let camera_angular_velocity: GLfloat = 0.001;
        camera_yaw += Rad(mouse_dx) * camera_angular_velocity;
        camera_pitch += Rad(mouse_dy) * camera_angular_velocity;

        if camera_pitch > Rad::turn_div_4() {
            camera_pitch = Rad::turn_div_4();
        } else if camera_pitch < -Rad::turn_div_4() {
            camera_pitch = -Rad::turn_div_4();
        }

        let camera_zoom_velocity: GLfloat = 0.10;
        camera_fov += Rad(mouse_dscroll) * camera_zoom_velocity * (delta_frame as f32);
        if camera_fov > Deg(80.0).into() {
            camera_fov = Deg(80.0).into()
        } else if camera_fov < Deg(10.0).into() {
            camera_fov = Deg(10.0).into()
        }

        let camera_rot = Quaternion::from_axis_angle(Vector3::unit_y(), -camera_yaw) *
            Quaternion::from_axis_angle(Vector3::unit_x(), -camera_pitch);

        // Render.
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, tex_id.as_uint());

            program.use_program();

            let pos_from_wld_to_cam_space = Matrix4::from(camera_rot.invert()) *
                Matrix4::from_translation(-camera_pos);

            let pos_from_cam_to_clp_space = Matrix4::from(PerspectiveFov {
                fovy: camera_fov,
                aspect: camera_aspect,
                near: INITIAL_NEAR,
                far: INITIAL_FAR,
            });

            let light_pos_in_wld_space = Quaternion::from_angle_y(Deg(delta_start * 90.0))
                .rotate_vector(Vector3::new(3.0, 2.0, 0.0));

            let pos_from_obj_to_wld_space =
                Matrix4::from_translation(Vector3::new(-1.0, 0.0, 0.0)) *
                    Matrix4::from_angle_y(Deg(delta_start * 20.0));

            let pos_from_obj_to_cam_space = pos_from_wld_to_cam_space * pos_from_obj_to_wld_space;
            let pos_from_obj_to_clp_space = pos_from_cam_to_clp_space * pos_from_obj_to_cam_space;

            {
                let loc =
                    gl::GetUniformLocation(program.as_uint(), c_str!("pos_from_obj_to_cam_space"));
                gl::UniformMatrix4fv(loc, 1, gl::FALSE, pos_from_obj_to_cam_space.as_ptr());
            }

            {
                // FIXME: Create 3x3 matrix instead of 4x4. We don't care about translation.
                let nor_from_obj_to_cam_space =
                    pos_from_obj_to_cam_space.invert().unwrap().transpose();
                let loc =
                    gl::GetUniformLocation(program.as_uint(), c_str!("nor_from_obj_to_cam_space"));
                gl::UniformMatrix4fv(loc, 1, gl::FALSE, nor_from_obj_to_cam_space.as_ptr());
            }

            {
                let loc =
                    gl::GetUniformLocation(program.as_uint(), c_str!("pos_from_obj_to_clp_space"));
                gl::UniformMatrix4fv(loc, 1, gl::FALSE, pos_from_obj_to_clp_space.as_ptr());
            }

            {
                let light_pos_in_cam_space =
                    (pos_from_wld_to_cam_space * light_pos_in_wld_space.extend(1.0)).truncate();
                let loc =
                    gl::GetUniformLocation(program.as_uint(), c_str!("light_pos_in_cam_space"));
                gl::Uniform3fv(loc, 1, light_pos_in_cam_space.as_ptr());
            }

            gl::BindVertexArray(va.id().as_uint());
            gl::DrawElements(
                gl::TRIANGLES,
                (3 * mesh.indices.len()) as GLsizei,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );

            light_program.use_program();

            let pos_from_obj_to_wld_space = Matrix4::from_translation(light_pos_in_wld_space) *
                Matrix4::from_scale(0.2);
            let pos_from_obj_to_cam_space = pos_from_wld_to_cam_space * pos_from_obj_to_wld_space;
            let pos_from_obj_to_clp_space = pos_from_cam_to_clp_space * pos_from_obj_to_cam_space;

            {
                let loc = gl::GetUniformLocation(
                    light_program.as_uint(),
                    c_str!("pos_from_obj_to_clp_space"),
                );
                gl::UniformMatrix4fv(loc, 1, gl::FALSE, pos_from_obj_to_clp_space.as_ptr());
            }

            gl::BindVertexArray(light_vertex_array.id().as_uint());
            gl::DrawElements(
                gl::TRIANGLES,
                (3 * light_mesh.indices.len()) as GLsizei,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        }

        gl_window.swap_buffers().unwrap();
    }
}

fn file_to_string<P: AsRef<Path>>(path: P) -> std::io::Result<String> {
    let mut file = io::BufReader::new(fs::File::open(path)?);
    let mut string = String::new();
    file.read_to_string(&mut string)?;
    Ok(string)
}
