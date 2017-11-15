extern crate gl;

use gl::types::*;
use cgmath::*;
use std::path::Path;
use std::io::Read;
use std::io;
use std::fs;

#[derive(Debug)]
pub struct Mesh {
    pub data: Vec<[VertexData; 3]>,
}

#[derive(Debug)]
#[repr(C, packed)]
struct TriangleElement {
    xyz_index: GLuint,
    uv_index: GLuint,
}

#[derive(Debug)]
#[repr(C, packed)]
pub struct VertexData {
    pub xyz: Vector3<GLfloat>,
    pub uv: Vector2<GLfloat>,
}

pub fn import_obj<P: AsRef<Path>>(path: P) -> io::Result<Mesh> {
    let mut file = fs::File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut xyzs = Vec::new();
    let mut uvs = Vec::new();
    let mut triangles = Vec::new();

    for line in contents.lines() {
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("v") => {
                // Parse "v 0.397429 3.307064 0.397429"
                let xyz: Vector3<GLfloat> = Vector3::new(
                    parts.next().unwrap().parse().unwrap(),
                    parts.next().unwrap().parse().unwrap(),
                    parts.next().unwrap().parse().unwrap(),
                );
                xyzs.push(xyz);
            }
            Some("vt") => {
                // Parse "vt 0.532019 0.081125"
                let uv: Vector2<GLfloat> = Vector2::new(
                    parts.next().unwrap().parse().unwrap(),
                    parts.next().unwrap().parse().unwrap(),
                );
                uvs.push(uv);
            }
            Some("f") => {
                // Parse "f 1/1 2/2 4/3" (triangle with mesh/tex indices).
                let mut tri: [TriangleElement; 3] = unsafe { ::std::mem::uninitialized() };

                for i in 0..tri.len() {
                    let mut ind = parts.next().unwrap().split('/');
                    tri[i] = TriangleElement {
                        xyz_index: ind.next().unwrap().parse::<GLuint>().unwrap() - 1,
                        uv_index: ind.next().unwrap().parse::<GLuint>().unwrap() - 1,
                    }
                }

                triangles.push(tri);
            }
            _ => (),
        }
    }

    let data: Vec<_> = triangles
        .iter()
        .map(|triangle| {
            let el_to_data = |&TriangleElement {
                                  xyz_index,
                                  uv_index,
                              }| {
                VertexData {
                    xyz: xyzs[xyz_index as usize],
                    uv: uvs[uv_index as usize],
                }
            };
            [
                el_to_data(&triangle[0]),
                el_to_data(&triangle[1]),
                el_to_data(&triangle[2]),
            ]
        })
        .collect();

    Ok(Mesh { data })
}
