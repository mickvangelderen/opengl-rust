struct WavefrontObj {
    material_filename: Option<String>,
    objects: Vec<Object>,
}

struct PosInObjSpace([f32; 3]);

struct PosInTexSpace([f32; 2]);

struct NorInObjSpace([f32; 3]);

struct Object {
    name: String,
    positions_in_obj_space: Vec<PosInObjSpace>,
    positions_in_tex_space: Vec<PosInTexSpace>,
    normals_in_obj_space: Vec<NorInObjSpace>,
    geometry: Vec<Group>
}

struct Group {
    material_name: Option<String>,
    pos_in_obj_space_indices: Vec<NonZero<u32>>,
    pos_in_tex_space_indices: Vec<Option<NonZero<u32>>>,
    nor_in_obj_space_indices: Vec<Option<NonZero<u32>>>,
    polygon_counts: Vec<u8>,
}
