extern crate gl;

use gl::types::*;
use cgmath::*;
use std::path::Path;
use std::io::Read;
use std::io;
use std::fs;
use super::palette;

#[derive(Debug)]
pub struct Mesh {
    pub elements: Vec<VertexData>,
    pub indices: Vec<[GLuint; 3]>,
}

#[derive(PartialEq, Copy, Clone, Debug)]
#[repr(C)]
struct TriangleElement {
    vertex_position_index: GLuint,
    texture_position_index: GLuint,
    vertex_normal_index: GLuint,
}

#[test]
fn triangle_element_has_expected_layout() {
    assert_eq!(0, field_offset!(TriangleElement, vertex_position_index));
    assert_eq!(4, field_offset!(TriangleElement, texture_position_index));
    assert_eq!(8, field_offset!(TriangleElement, vertex_normal_index));
    assert_eq!(12, ::std::mem::size_of::<TriangleElement>());
}

#[derive(Debug)]
#[repr(C)]
pub struct VertexData {
    pub vertex_position: Vector3<GLfloat>,
    pub texture_position: Vector2<GLfloat>,
    pub vertex_normal: Vector3<GLfloat>,
}

#[test]
fn vertex_data_has_expected_layout() {
    assert_eq!(00, field_offset!(VertexData, vertex_position));
    assert_eq!(12, field_offset!(VertexData, texture_position));
    assert_eq!(20, field_offset!(VertexData, vertex_normal));
    assert_eq!(32, ::std::mem::size_of::<VertexData>());
}

pub fn import_obj<P: AsRef<Path>>(path: P) -> io::Result<Mesh> {
    let mut file = fs::File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut vertex_positions: Vec<Vector3<GLfloat>> = Vec::new();
    let mut vertex_normals: Vec<Vector3<GLfloat>> = Vec::new();
    let mut texture_positions: Vec<Vector2<GLfloat>> = Vec::new();

    let mut triangles = Vec::new();

    for line in contents.lines() {
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("v") => {
                // Parse "v 0.397429 3.307064 0.397429"
                vertex_positions.push(Vector3::new(
                    parts.next().unwrap().parse().unwrap(),
                    parts.next().unwrap().parse().unwrap(),
                    parts.next().unwrap().parse().unwrap(),
                ));
            }
            Some("vn") => {
                // Parse "vn 0.6133 -0.1738 0.7705"
                vertex_normals.push(Vector3::new(
                    parts.next().unwrap().parse().unwrap(),
                    parts.next().unwrap().parse().unwrap(),
                    parts.next().unwrap().parse().unwrap(),
                ));
            }
            Some("vt") => {
                // Parse "vt 0.532019 0.081125"
                texture_positions.push(Vector2::new(
                    parts.next().unwrap().parse().unwrap(),
                    parts.next().unwrap().parse().unwrap(),
                ));
            }
            Some("f") => {
                // Parse "f 1/1 2/2 4/3" (triangle with mesh/tex indices).
                let mut tri: [TriangleElement; 3] = unsafe { ::std::mem::uninitialized() };

                for i in 0..tri.len() {
                    let mut ind = parts.next().unwrap().split('/');
                    tri[i] = TriangleElement {
                        vertex_position_index: ind.next().unwrap().parse::<GLuint>().unwrap() - 1,
                        texture_position_index: ind.next().unwrap().parse::<GLuint>().unwrap() - 1,
                        vertex_normal_index: ind.next().unwrap().parse::<GLuint>().unwrap() - 1,
                    }
                }

                triangles.push(tri);
            }
            _ => (),
        }
    }

    let elements: Vec<_> = triangles.iter().flat_map(|tri| tri).collect();

    let palette::Palette {
        elements: element_palette,
        indices: element_indices,
    } = palette::Palette::<_, GLuint>::naive(&elements, |a, b| a == b);

    let element_palette = element_palette
        .iter()
        .map(|element| {
            VertexData {
                vertex_position: vertex_positions[element.vertex_position_index as usize],
                texture_position: texture_positions[element.texture_position_index as usize],
                vertex_normal: vertex_normals[element.vertex_normal_index as usize],
            }
        })
        .collect();

    let element_indices = unsafe {
        // Beware: quality programming incoming.
        let l = element_indices.len();
        assert!(l % 3 == 0);
        let mut i = ::std::mem::transmute::<Vec<GLuint>, Vec<[GLuint; 3]>>(element_indices);
        i.set_len(l / 3);
        i
    };

    Ok(Mesh {
        elements: element_palette,
        indices: element_indices,
    })
}
