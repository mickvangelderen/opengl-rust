#![feature(nonzero)]
#![feature(stmt_expr_attributes)]

extern crate gl;
extern crate glutin;
extern crate core;
extern crate image;
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

// use shader::*;
use shader::specialization::*;
use program::*;
// use import::*;
// use palette::*;
use texture::*;
use vertex_buffer::*;
use vertex_array::*;
use viewport::*;

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
    const INITIAL_FOVY: Rad<GLfloat> = Rad(45.0 * 3.1415 / 180.0);
    const INITIAL_NEAR: GLfloat = 0.1;
    const INITIAL_FAR: GLfloat = 100.0;

    let mut viewport = Viewport::new(1024, 768);

    let mut events_loop = glutin::EventsLoop::new();

    let gl_window = glutin::GlWindow::new(
        glutin::WindowBuilder::new()
            .with_title("rust-opengl")
            .with_dimensions(
                viewport.width().abs() as u32,
                viewport.height().abs() as u32,
            ),
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
        let vertex_shader = VertexShaderId::new()
            .expect("Failed to acquire vertex shader id.")
            .compile(&[&vertex_src])
            .expect("Failed to compile vertex shader.");

        let fragment_src = file_to_string("assets/standard.frag").unwrap();
        let fragment_shader = FragmentShaderId::new()
            .expect("Failed to acquire fragment shader id.")
            .compile(&[&fragment_src])
            .expect("Failed to compile fragment shader.");

        let program = ProgramId::new().expect("Failed to acquire program id.");
        program
            .link(&[vertex_shader.as_ref(), fragment_shader.as_ref()])
            .expect("Failed to link program.")
    };

    let mesh = import::import_obj("assets/monster.obj").expect("Failed to import monster.obj");

    let va = VertexArrayId::new().unwrap();
    let vb = VertexBufferId::new().unwrap();
    let ve = VertexBufferId::new().unwrap();

    unsafe {
        va.bind();

        vb.bind(BufferTarget::ArrayBuffer);

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

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ve.as_uint());

        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            mem::size_of_val(&mesh.indices[..]) as GLsizeiptr,
            mesh.indices.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW,
        );
    }

    let tex_id: TextureId = {
        let img = image::open("assets/monster-diffuse.jpg").unwrap();

        let img = img.flipv().to_rgba();

        let mut tex_id = TextureId::new().unwrap();
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

            // Each row is expected to be padded to be a multiple of
            // GL_UNPACK_ALIGNMENT which is 4 by default. Here we set it to
            // 1 which means the rows will not be padded.
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

            gl::TexImage2D(
                gl::TEXTURE_2D, // Target
                0, // MIP map level
                gl::RGBA8 as GLint, // internal format
                img.width() as GLint, // width
                img.height() as GLint, // height
                0, // border, must be zero
                gl::RGBA, // format
                gl::UNSIGNED_BYTE, // component format
                img.as_ptr() as *const GLvoid, // data
            );

            gl::GenerateMipmap(gl::TEXTURE_2D);
        }

        tex_id
    };

    // Set up texture location for program.
    program.bind();
    unsafe {
        let loc = gl::GetUniformLocation(program.as_uint(), c_str!("tex_color"));
        gl::Uniform1i(loc, 0);
    }

    let light_program = {
        let vertex_shader = VertexShaderId::new()
            .unwrap()
            .compile(&[&file_to_string("assets/light.vert").unwrap()])
            .unwrap();
        let fragment_shader = FragmentShaderId::new()
            .unwrap()
            .compile(&[&file_to_string("assets/light.frag").unwrap()])
            .unwrap();
        let program = ProgramId::new().unwrap();
        program
            .link(&[vertex_shader.as_ref(), fragment_shader.as_ref()])
            .unwrap()
    };

    let light_mesh = import::import_obj("assets/icosphere-80.obj").expect("Failed to import obj");

    let light_vertex_array = VertexArrayId::new().unwrap();
    let light_vertex_buffer = VertexBufferId::new().unwrap();
    let light_elements_buffer = VertexBufferId::new().unwrap();

    unsafe {
        light_vertex_array.bind();

        gl::BindBuffer(gl::ARRAY_BUFFER, light_vertex_buffer.as_uint());

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
            light_elements_buffer.as_uint(),
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
                        viewport.update().width(w as GLsizei).height(h as GLsizei);
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

            program.bind();

            let pos_from_wld_to_cam_space = Matrix4::from(camera_rot.invert()) *
                Matrix4::from_translation(-camera_pos);

            let pos_from_cam_to_clp_space = Matrix4::from(PerspectiveFov {
                fovy: camera_fov,
                aspect: viewport.aspect(),
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

            va.bind();

            gl::DrawElements(
                gl::TRIANGLES,
                (3 * mesh.indices.len()) as GLsizei,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );

            light_program.bind();

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

            light_vertex_array.bind();

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
