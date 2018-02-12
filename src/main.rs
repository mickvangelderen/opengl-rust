#![feature(nonzero)]
#![feature(stmt_expr_attributes)]
#![feature(non_exhaustive)]

extern crate cgmath;
extern crate core;
extern crate gl;
extern crate glutin;
extern crate image;

#[macro_use(field_offset)]
extern crate simple_field_offset;

pub mod camera;
pub mod shader;
pub mod framebuffer;
pub mod renderbuffer;
pub mod program;
pub mod import;
pub mod palette;
pub mod texture;
pub mod vertex_buffer;
pub mod vertex_array;
pub mod viewport;

#[macro_use]
pub mod string;

#[macro_use]
pub mod debug;

use camera::*;
// use shader::*;
use shader::specialization::*;
use framebuffer::*;
use renderbuffer::*;
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

struct LightColor {
    ambient: Vector3<f32>,
    diffuse: Vector3<f32>,
    specular: Vector3<f32>,
}

struct LightAttenuation {
    constant: f32,
    linear: f32,
    quadratic: f32,
}

struct PointLight {
    color: LightColor,
    attenuation: LightAttenuation,
    position: Vector3<f32>,
}

// struct DirectionalLight {
//     color: LightColor,
//     attenuation: LightAttenuation,
//     direction: Vector3<f32>,
// }

// struct SpotLight {
//     color: LightColor,
//     attenuation: LightAttenuation,
//     position: Vector3<f32>,
//     direction: Vector3<f32>,
//     inner_angle: Rad<f32>,
//     outer_angle: Rad<f32>,
// }

impl PointLight {
    fn set_standard_program_uniforms(
        &self,
        program: &LinkedProgramId,
        index: usize,
        pos_from_wld_to_cam_space: &Matrix4<f32>,
    ) {
        unsafe {
            let pos_in_cam_space =
                (pos_from_wld_to_cam_space * self.position.extend(1.0)).truncate();
            let name: String = format!("point_lights[{}].pos_in_cam_space\0", index);
            let loc = gl::GetUniformLocation(program.as_uint(), name.as_ptr() as *const GLchar);
            gl::Uniform3f(
                loc,
                pos_in_cam_space.x,
                pos_in_cam_space.y,
                pos_in_cam_space.z,
            );
        }

        unsafe {
            let name: String = format!("point_lights[{}].ambient\0", index);
            let loc = gl::GetUniformLocation(program.as_uint(), name.as_ptr() as *const GLchar);
            gl::Uniform3f(
                loc,
                self.color.ambient.x,
                self.color.ambient.y,
                self.color.ambient.z,
            );
        }

        unsafe {
            let name: String = format!("point_lights[{}].diffuse\0", index);
            let loc = gl::GetUniformLocation(program.as_uint(), name.as_ptr() as *const GLchar);
            gl::Uniform3f(
                loc,
                self.color.diffuse.x,
                self.color.diffuse.y,
                self.color.diffuse.z,
            );
        }

        unsafe {
            let name: String = format!("point_lights[{}].specular\0", index);
            let loc = gl::GetUniformLocation(program.as_uint(), name.as_ptr() as *const GLchar);
            gl::Uniform3f(
                loc,
                self.color.specular.x,
                self.color.specular.y,
                self.color.specular.z,
            );
        }

        unsafe {
            let name: String = format!("point_lights[{}].attenuation_constant\0", index);
            let loc = gl::GetUniformLocation(program.as_uint(), name.as_ptr() as *const GLchar);
            gl::Uniform1f(loc, self.attenuation.constant);
        }

        unsafe {
            let name: String = format!("point_lights[{}].attenuation_linear\0", index);
            let loc = gl::GetUniformLocation(program.as_uint(), name.as_ptr() as *const GLchar);
            gl::Uniform1f(loc, self.attenuation.linear);
        }

        unsafe {
            let name: String = format!("point_lights[{}].attenuation_quadratic\0", index);
            let loc = gl::GetUniformLocation(program.as_uint(), name.as_ptr() as *const GLchar);
            gl::Uniform1f(loc, self.attenuation.quadratic);
        }
    }
}

fn duration_to_seconds(duration: time::Duration) -> f64 {
    let seconds = duration.as_secs() as f64;
    let nanoseconds = duration.subsec_nanos() as f64;
    seconds + nanoseconds * 1e-9
}

fn main() {
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

    gl::load_with(|symbol| gl_window.get_proc_address(symbol) as *const _);

    let mut texture_unit_slot = TextureUnitSlot {};
    let mut program_slot = ProgramSlot {};
    let mut framebuffer_slot = FramebufferSlot {};

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

    let mesh = import::import_obj("assets/crate.obj").expect("Failed to import crate.obj");

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

    let diffuse_texture_id: TextureId = {
        let img = image::open("assets/crate_diffuse.png").unwrap();

        let img = img.flipv().to_rgba();

        let mut diffuse_texture_id = TextureId::new().unwrap();
        unsafe {
            // Each row is expected to be padded to be a multiple of
            // GL_UNPACK_ALIGNMENT which is 4 by default. Here we set it to
            // 1 which means the rows will not be padded.

            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

            texture_unit_slot
                .active_texture(TextureUnit::TextureUnit0)
                .texture_target_2d
                .bind(&diffuse_texture_id)
                .min_filter(TextureFilter::LinearMipmapLinear)
                .mag_filter(TextureFilter::LinearMipmapLinear)
                .wrap_s(gl::REPEAT as GLint)
                .wrap_t(gl::REPEAT as GLint)
                .image_2d(
                    0,                             // MIP map level
                    gl::RGBA8 as GLint,            // internal format
                    img.width() as GLint,          // width
                    img.height() as GLint,         // height
                    gl::RGBA,                      // format
                    gl::UNSIGNED_BYTE,             // component format
                    img.as_ptr() as *const GLvoid, // data
                )
                .generate_mipmap();
        }

        diffuse_texture_id
    };

    let specular_texture_id: TextureId = {
        let img = image::open("assets/crate_specular.png").unwrap();

        let img = img.flipv().to_rgba();

        let mut specular_texture_id = TextureId::new().unwrap();
        unsafe {
            // Each row is expected to be padded to be a multiple of
            // GL_UNPACK_ALIGNMENT which is 4 by default. Here we set it to
            // 1 which means the rows will not be padded.
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);

            texture_unit_slot
                .active_texture(TextureUnit::TextureUnit0)
                .texture_target_2d
                .bind(&specular_texture_id)
                .min_filter(TextureFilter::LinearMipmapLinear)
                .mag_filter(TextureFilter::LinearMipmapLinear)
                .wrap_s(gl::REPEAT as GLint)
                .wrap_t(gl::REPEAT as GLint)
                .image_2d(
                    0,                             // MIP map level
                    gl::RGBA8 as GLint,            // internal format
                    img.width() as GLint,          // width
                    img.height() as GLint,         // height
                    gl::RGBA,                      // format
                    gl::UNSIGNED_BYTE,             // component format
                    img.as_ptr() as *const GLvoid, // data
                )
                .generate_mipmap();
        }

        specular_texture_id
    };

    program_slot
        .bind(&program)
        // Set texture units.
        .set_uniform_1i(&program.uniform_location(static_cstr!("material.diffuse")), 0)
        .set_uniform_1i(&program.uniform_location(static_cstr!("material.specular")), 1)
        // Set shininess.
        .set_uniform_1f(&program.uniform_location(static_cstr!("material.shininess")), 64.0);

    // Point Lights.

    let mut point_lights = [
        PointLight {
            position: Vector3::new(0.0, 0.0, 0.0),
            color: LightColor {
                ambient: Vector3::new(0.1, 0.1, 0.1),
                diffuse: Vector3::new(1.0, 1.0, 1.0),
                specular: Vector3::new(1.0, 1.0, 1.0),
            },
            attenuation: LightAttenuation {
                constant: 1.0,
                linear: 0.03,
                quadratic: 0.01,
            },
        },
        PointLight {
            position: Vector3::new(3.0, 0.0, 0.0),
            color: LightColor {
                ambient: Vector3::new(0.1, 0.1, 0.1),
                diffuse: Vector3::new(1.0, 0.2, 0.2),
                specular: Vector3::new(1.0, 0.2, 0.2),
            },
            attenuation: LightAttenuation {
                constant: 1.0,
                linear: 0.03,
                quadratic: 0.01,
            },
        },
        PointLight {
            position: Vector3::new(0.0, 3.0, 0.0),
            color: LightColor {
                ambient: Vector3::new(0.1, 0.1, 0.1),
                diffuse: Vector3::new(0.2, 1.0, 0.2),
                specular: Vector3::new(0.2, 1.0, 0.2),
            },
            attenuation: LightAttenuation {
                constant: 1.0,
                linear: 0.03,
                quadratic: 0.01,
            },
        },
        PointLight {
            position: Vector3::new(0.0, 0.0, 3.0),
            color: LightColor {
                ambient: Vector3::new(0.1, 0.1, 0.1),
                diffuse: Vector3::new(0.2, 0.2, 1.0),
                specular: Vector3::new(0.2, 0.2, 1.0),
            },
            attenuation: LightAttenuation {
                constant: 1.0,
                linear: 0.03,
                quadratic: 0.01,
            },
        },
    ];

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

        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, light_elements_buffer.as_uint());

        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            mem::size_of_val(&light_mesh.indices[..]) as GLsizeiptr,
            light_mesh.indices.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW,
        );
    }

    // Create a framebuffer to render to.
    let main_fb = FramebufferId::new().unwrap();

    let main_fb_tex = TextureId::new().unwrap();

    unsafe {
        texture_unit_slot
            .active_texture(TextureUnit::TextureUnit0)
            .texture_target_2d
            .bind(&main_fb_tex)
            .min_filter(TextureFilter::Nearest)
            .mag_filter(TextureFilter::Nearest)
            .wrap_s(gl::CLAMP_TO_EDGE as GLint)
            .wrap_t(gl::CLAMP_TO_EDGE as GLint)
            .image_2d(
                0,                 // MIP map level
                gl::RGB8 as GLint, // internal format
                viewport.width(),
                viewport.height(),
                gl::RGB,           // format
                gl::UNSIGNED_BYTE, // component format
                std::ptr::null(),  // data
            );

        let _bound_fb = framebuffer_slot.bind(FramebufferTarget::Framebuffer, &main_fb);

        gl::FramebufferTexture2D(
            FramebufferTarget::Framebuffer as GLenum,
            FramebufferAttachment::color(0) as GLenum,
            TextureTarget2d::as_enum(),
            main_fb_tex.as_uint(),
            0,
        );
    }

    let main_fb_depth_stencil = RenderBufferId::new().unwrap();
    main_fb_depth_stencil.bind(RenderBufferTarget::RenderBuffer);

    unsafe {
        gl::RenderbufferStorage(
            RenderBufferTarget::RenderBuffer as GLenum,
            RenderBufferInternalFormat::DEPTH24_STENCIL8 as GLenum,
            viewport.width(),
            viewport.height(),
        );
        gl::FramebufferRenderbuffer(
            FramebufferTarget::Framebuffer as GLenum,
            gl::DEPTH_STENCIL_ATTACHMENT,
            RenderBufferTarget::RenderBuffer as GLenum,
            main_fb_depth_stencil.as_uint(),
        );
    }

    match unsafe { gl::CheckFramebufferStatus(FramebufferTarget::Framebuffer as GLenum) } {
        gl::FRAMEBUFFER_COMPLETE => {}
        _ => {
            panic!("Framebuffer not complete");
        }
    }

    let post_vao = VertexArrayId::new().unwrap();
    let post_vbo = VertexBufferId::new().unwrap();
    let post_vertex_data: [GLfloat; 16] = [
        -1.0, -1.0, 0.0, 0.0, // vertex coordinates, texture coordinates
        1.0, -1.0, 1.0, 0.0, //
        -1.0, 1.0, 0.0, 1.0, //
        1.0, 1.0, 1.0, 1.0, //
    ];
    let post_program = {
        let vertex_shader = VertexShaderId::new()
            .unwrap()
            .compile(&[&file_to_string("assets/post.vert").unwrap()])
            .unwrap();
        let fragment_shader = FragmentShaderId::new()
            .unwrap()
            .compile(&[&file_to_string("assets/post.frag").unwrap()])
            .unwrap();
        let program = ProgramId::new().unwrap();
        program
            .link(&[vertex_shader.as_ref(), fragment_shader.as_ref()])
            .unwrap()
    };

    program_slot
        .bind(&post_program)
        .set_uniform_1f(
            &post_program.uniform_location(static_cstr!("dx")),
            1.0 / viewport.width() as f32,
        )
        .set_uniform_1f(
            &post_program.uniform_location(static_cstr!("dy")),
            1.0 / viewport.height() as f32,
        );

    unsafe {
        post_vao.bind();

        post_vbo.bind(BufferTarget::ArrayBuffer);

        gl::BufferData(
            BufferTarget::ArrayBuffer as GLenum,
            mem::size_of_val(&post_vertex_data[..]) as GLsizeiptr,
            post_vertex_data.as_ptr() as *const GLvoid,
            gl::STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,                                         // attribute index
            2,                                         // count
            gl::FLOAT,                                 // type
            gl::FALSE,                                 // normalize
            mem::size_of::<[GLfloat; 4]>() as GLsizei, // stride
            0 as *const GLvoid,                        // offset
        );
        gl::EnableVertexAttribArray(0);

        gl::VertexAttribPointer(
            1,
            2,
            gl::FLOAT,
            gl::FALSE,
            mem::size_of::<[GLfloat; 4]>() as GLsizei, // stride
            mem::size_of::<[GLfloat; 2]>() as *const GLvoid, // offset
        );
        gl::EnableVertexAttribArray(1);
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

    let mut camera = Camera {
        position: Vector3::new(0.0, 4.0, 10.0),
        pitch: Rad(0.0),
        yaw: Rad(0.0),
        fov: Rad::from(Deg(45.0)),
        positional_velocity: 2.0,
        angular_velocity: 0.06,
        zoom_velocity: 0.10,
    };

    let mut has_focus = true;

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
        events_loop.poll_events(|event| {
            use glutin::Event::*;
            match event {
                WindowEvent { event, .. } => {
                    use glutin::WindowEvent::*;
                    use glutin::ElementState::*;
                    match event {
                        Closed => running = false,
                        Resized(w, h) => {
                            gl_window.resize(w, h);
                            viewport.update().width(w as GLsizei).height(h as GLsizei);

                            // Update framebuffer color texture size.
                            unsafe {
                                texture_unit_slot
                                    .active_texture(TextureUnit::TextureUnit0)
                                    .texture_target_2d
                                    .bind(&main_fb_tex)
                                    .image_2d(
                                        0,                 // MIP map level
                                        gl::RGB8 as GLint, // internal format
                                        viewport.width(),
                                        viewport.height(),
                                        gl::RGB,           // format
                                        gl::UNSIGNED_BYTE, // component format
                                        std::ptr::null(),  // data
                                    );
                            }

                            // Update framebuffer depth+stencil size.
                            unsafe {
                                // TODO(mickvangelderen): Is this
                                // required at all for
                                // RenderbufferStorage?
                                let _bound_fb =
                                    framebuffer_slot.bind(FramebufferTarget::Framebuffer, &main_fb);
                                main_fb_depth_stencil.bind(RenderBufferTarget::RenderBuffer);

                                gl::RenderbufferStorage(
                                    RenderBufferTarget::RenderBuffer as GLenum,
                                    RenderBufferInternalFormat::DEPTH24_STENCIL8 as GLenum,
                                    viewport.width(),
                                    viewport.height(),
                                );
                            }

                            // Update uniforms dependent on viewport size.
                            program_slot
                                .bind(&post_program)
                                .set_uniform_1f(
                                    &post_program.uniform_location(static_cstr!("dx")),
                                    1.0 / viewport.width() as f32,
                                )
                                .set_uniform_1f(
                                    &post_program.uniform_location(static_cstr!("dy")),
                                    1.0 / viewport.height() as f32,
                                );
                        }

                        KeyboardInput { input, .. } => {
                            let pressed = if let Pressed = input.state {
                                true
                            } else {
                                false
                            };

                            use glutin::VirtualKeyCode::*;
                            match input.virtual_keycode {
                                Some(Escape) => {
                                    if has_focus {
                                        running = false;
                                    }
                                }
                                Some(W) => move_forward = pressed,
                                Some(S) => move_backward = pressed,
                                Some(A) => move_left = pressed,
                                Some(D) => move_right = pressed,
                                Some(Q) => move_up = pressed,
                                Some(Z) => move_down = pressed,
                                _ => (),
                            }
                        }
                        Focused(state) => {
                            has_focus = state;
                        }
                        _ => (),
                    }
                }
                DeviceEvent {
                    device_id, event, ..
                } => {
                    use glutin::DeviceEvent::*;
                    match event {
                        Added => println!("Added device {:?}", device_id),
                        Removed => println!("Removed device {:?}", device_id),
                        Motion { axis, value } => match axis {
                            0 => mouse_dx += value as f32,
                            1 => mouse_dy += value as f32,
                            3 => mouse_dscroll += value as f32,
                            _ => (),
                        },
                        _ => (),
                    }
                }
                _ => (),
            }
        });

        if has_focus {
            camera.update(&CameraUpdate {
                delta_time: delta_frame as f32,
                delta_position: Vector3 {
                    x: move_right as u32 as GLfloat - move_left as u32 as GLfloat,
                    y: move_up as u32 as GLfloat - move_down as u32 as GLfloat,
                    z: move_backward as u32 as GLfloat - move_forward as u32 as GLfloat,
                },
                delta_yaw: Rad(mouse_dx),
                delta_pitch: Rad(mouse_dy),
                delta_scroll: mouse_dscroll,
            });
        }

        point_lights[0].position = Quaternion::from_angle_y(Deg(delta_start * 90.0))
            .rotate_vector(Vector3::new(3.0, 2.0, 0.0));

        let pos_from_wld_to_cam_space = camera.pos_from_wld_to_cam_space();

        let pos_from_cam_to_clp_space = Matrix4::from(PerspectiveFov {
            fovy: camera.fov,
            aspect: viewport.aspect(),
            near: INITIAL_NEAR,
            far: INITIAL_FAR,
        });

        // Render.
        unsafe {
            let _bound_fb = framebuffer_slot.bind(FramebufferTarget::Framebuffer, &main_fb);
            gl::ClearColor(0.7, 0.8, 0.9, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::Enable(gl::DEPTH_TEST);

            texture_unit_slot
                .active_texture(TextureUnit::TextureUnit0)
                .texture_target_2d
                .bind(&diffuse_texture_id);

            texture_unit_slot
                .active_texture(TextureUnit::TextureUnit1)
                .texture_target_2d
                .bind(&specular_texture_id);

            let _bound_program = program_slot.bind(&program);

            {
                let pos_from_obj_to_wld_space = Matrix4::from_translation(Vector3::zero())
                    * Matrix4::from_angle_y(Deg(delta_start * 20.0));

                let pos_from_obj_to_cam_space =
                    pos_from_wld_to_cam_space * pos_from_obj_to_wld_space;
                let pos_from_obj_to_clp_space =
                    pos_from_cam_to_clp_space * pos_from_obj_to_cam_space;
                {
                    let loc = gl::GetUniformLocation(
                        program.as_uint(),
                        gl_str!("pos_from_obj_to_cam_space"),
                    );
                    gl::UniformMatrix4fv(loc, 1, gl::FALSE, pos_from_obj_to_cam_space.as_ptr());
                }

                {
                    // FIXME: Create 3x3 matrix instead of 4x4. We don't care about translation.
                    let nor_from_obj_to_cam_space =
                        pos_from_obj_to_cam_space.invert().unwrap().transpose();
                    let loc = gl::GetUniformLocation(
                        program.as_uint(),
                        gl_str!("nor_from_obj_to_cam_space"),
                    );
                    gl::UniformMatrix4fv(loc, 1, gl::FALSE, nor_from_obj_to_cam_space.as_ptr());
                }

                {
                    let loc = gl::GetUniformLocation(
                        program.as_uint(),
                        gl_str!("pos_from_obj_to_clp_space"),
                    );
                    gl::UniformMatrix4fv(loc, 1, gl::FALSE, pos_from_obj_to_clp_space.as_ptr());
                }
            }

            // Set light uniforms.
            for (i, light) in point_lights.iter().enumerate() {
                light.set_standard_program_uniforms(&program, i, &pos_from_wld_to_cam_space);
            }

            va.bind();

            gl::DrawElements(
                gl::TRIANGLES,
                (3 * mesh.indices.len()) as GLsizei,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            );
        }

        unsafe {
            // Draw point lights.
            let _bound_program = program_slot.bind(&light_program);

            light_vertex_array.bind();

            for light in point_lights.iter() {
                let pos_from_obj_to_wld_space =
                    Matrix4::from_translation(light.position) * Matrix4::from_scale(0.2);
                let pos_from_obj_to_cam_space =
                    pos_from_wld_to_cam_space * pos_from_obj_to_wld_space;
                let pos_from_obj_to_clp_space =
                    pos_from_cam_to_clp_space * pos_from_obj_to_cam_space;

                {
                    let loc = gl::GetUniformLocation(
                        light_program.as_uint(),
                        gl_str!("pos_from_obj_to_clp_space"),
                    );
                    gl::UniformMatrix4fv(loc, 1, gl::FALSE, pos_from_obj_to_clp_space.as_ptr());
                }

                {
                    let loc = gl::GetUniformLocation(light_program.as_uint(), gl_str!("color"));
                    gl::Uniform3f(
                        loc,
                        light.color.diffuse.x,
                        light.color.diffuse.y,
                        light.color.diffuse.z,
                    );
                }

                gl::DrawElements(
                    gl::TRIANGLES,
                    (3 * light_mesh.indices.len()) as GLsizei,
                    gl::UNSIGNED_INT,
                    std::ptr::null(),
                );
            }
        }

        unsafe {
            // Render offscreen buffer.
            let _bound_fb =
                framebuffer_slot.bind(FramebufferTarget::Framebuffer, &DEFAULT_FRAMEBUFFER_ID);
            // gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);
            // gl::ClearColor(0.0, 1.0, 0.0, 1.0);
            // gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::Disable(gl::DEPTH_TEST);
            let _bound_program = program_slot.bind(&post_program);
            post_vao.bind();

            texture_unit_slot
                .active_texture(TextureUnit::TextureUnit0)
                .texture_target_2d
                .bind(&main_fb_tex);

            gl::DrawArrays(gl::TRIANGLE_STRIP, 0 as GLint, 4 as GLsizei);
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
