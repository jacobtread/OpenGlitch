use bevy::math::Vec3;
use bevy::render::primitives::Sphere;
use binrw::{prelude::BinResult, BinRead};
use bitflags::bitflags;
use std::mem::size_of;
use std::{
    fmt::{Debug, Display},
    io::prelude::{Read, Seek},
};

/// String created from a fixed length chunk of bytes where
/// the remaining length is padded with zeros
pub struct PaddedString<const SIZE: usize>(String);

impl<const SIZE: usize> Debug for PaddedString<SIZE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl<const SIZE: usize> Display for PaddedString<SIZE> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl<const SIZE: usize> BinRead for PaddedString<SIZE> {
    type Args<'a> = ();

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        _endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let mut buffer = [0u8; SIZE];
        reader.read_exact(&mut buffer)?;

        let bytes = if let Some(end_index) = buffer.iter().position(|value| 0.eq(value)) {
            &buffer[..end_index]
        } else {
            // String takes up entire size
            &buffer
        };
        let value = String::from_utf8_lossy(bytes);

        Ok(PaddedString(value.to_string()))
    }
}

#[cfg(test)]
mod test {
    use binrw::BinRead;
    use std::fs::File;

    use super::PasmHeader;

    #[test]
    fn test_load_ape() {
        let mut file = File::open("data/ape/bridge1.ape").unwrap();

        let header: PasmHeader = PasmHeader::read(&mut file).unwrap();
        dbg!(&header);
    }
}

#[derive(BinRead, Debug)]
#[br(little)]
pub struct MtxOrientation {
    #[br(count = 12)]
    pub orientation: Vec<f32>,
}

/// Pasm file versioning
#[derive(BinRead, Debug)]
#[br(little)]
pub struct PasmHeaderVersion {
    pub sub: i8,
    pub minor: i8,
    pub major: i8,
    pub platform: i8,
}

/// Header for the pasm file format (.ape / .wld)
#[derive(BinRead, Debug)]
#[br(little, magic = b"FANG")]
pub struct PasmHeader {
    /// Scene name padded with null bytes
    pub scene_name: PaddedString<16>,

    pub wld: i32,
    pub num_bones: i16,
    pub num_cells: i16,
    pub num_lights: i16,
    pub num_vis_portals: i16,
    pub num_objects: i16,
    pub num_fog: i16,
    pub num_segments: i16,
    pub num_shapes: i16,

    // Struct sizing
    pub size_of_bone_struct: i16,
    pub size_of_light_struct: i16,
    pub size_of_object_struct: i16,
    pub size_of_fog_struct: i16,
    pub size_of_segment_struct: i16,
    pub size_of_material_struct: i16,
    pub size_of_vert_struct: i16,
    pub size_of_vert_index_struct: i16,
    #[br(pad_after = 66)]
    pub size_of_shape_struct: i16,
}

/// Bones
#[derive(BinRead, Debug)]
#[br(little)]
pub struct PasmBone {
    pub bone_name: PaddedString<32>,
    pub flags: i32,
    pub bone_index: i32,
    pub parent_index: i32,
    pub mtx_orientation: MtxOrientation,

    pub num_children: i32,
    #[br(count = num_children, pad_after = 16, pad_size_to = 64)]
    pub child_indices: Vec<u8>,
}

/// Bone weight
#[derive(BinRead, Debug)]
#[br(little)]
pub struct PasmWeight {
    pub bone_index: f32,
    #[br(pad_after = 16)]
    pub weight: f32,
}

#[derive(BinRead, Debug)]
#[br(little, repr = u32)]
pub enum PasmLightType {
    Spot = 0,
    Omni = 1,
    Dir = 2,
    Ambient = 3,
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct PasmLightFlags: u32 {
        /// Disregard the light's rgb and only use the motif's color
        const DONT_USE_RGB           = 0x00000001;
        /// Light the object that the light is attached to
        const LIGHT_SELF             = 0x00000002;
        /// Lights attached to this object don't light the terrain
        const DONT_LIGHT_TERRAIN     = 0x00000004;
        /// This light casts a projection on the environment
        const PER_PIXEL              = 0x00000008;
        /// This light will only be used in the lightmap portion of PASM and will not be exported to the engine.
        const LIGHTMAP_ONLY_LIGHT    = 0x00000010;
        /// This light is to be used for generating lightmaps (If it is not dynamic, it can be discarded prior to the engine)
        const LIGHTMAP_LIGHT         = 0x00000020;
        /// This light will generate its own unique lightmap in the lightmapping phase (it must also have a unique m_nLightID)
        const UNIQUE_LIGHTMAP        = 0x00000040;
        /// This light has a corona
        const CORONA                 = 0x00000080;
        /// Fade the corona as the camera gets closer.
        const CORONA_PROXYFADE       = 0x00000080;
        /// This light will cast shadows
        const CAST_SHADOWS           = 0x00000200;
        /// This light will not affect static objects
        const DYNAMIC_ONLY           = 0x00000400;
        /// For per-pixel lights that have a projected texture.
        const MESH_MUST_BE_PER_PIXEL = 0x00000800;
    }
}

impl BinRead for PasmLightFlags {
    type Args<'a> = ();
    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: binrw::Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let value: u32 = u32::read_options(reader, endian, args)?;
        Ok(PasmLightFlags::from_bits_retain(value))
    }
}

#[derive(BinRead, Debug)]
#[br(little)]
pub struct PasmLight {
    pub light_type: PasmLightType,
    pub light_name: PaddedString<16>,
    #[br(count = 4)]
    pub sphere: Vec<f32>,

    pub direction: PasmLightDirection,
    pub color: PasmColor,
    pub intensity: f32,

    pub spot_inner_angle: f32,
    pub spot_outer_angle: f32,

    pub flags: PasmLightFlags,
    pub motif_id: i32,

    pub corona_scale: f32,
    pub mtx_orientation: MtxOrientation,

    pub corona_texture: PaddedString<16>,
    pub per_pixel_texture: PaddedString<16>,

    pub light_id: i16,

    #[br(pad_after = 30)]
    pub parent_bone_name: PaddedString<32>,
}

/// sRGB color
#[derive(BinRead, Debug)]
#[br(little)]
pub struct PasmColor {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

#[derive(BinRead, Debug)]
#[br(little)]
pub struct PasmColor4 {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

#[derive(BinRead, Debug)]
#[br(little)]
pub struct PasmLightDirection {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(BinRead, Debug)]
#[br(little)]
pub struct PasmObject {
    #[br(pad_after = 4)]
    pub object_name: PaddedString<12>,

    pub flags: u32,

    pub mtx_orientation: MtxOrientation,

    pub user_data_length: u32,
    pub cull_distance: u32,
    pub parent_index: u32,

    #[br(pad_after = 20)]
    pub tint_rgb: PasmColor,

    #[br(count = user_data_length)]
    pub user_data: Vec<u8>,
}

#[derive(BinRead, Debug)]
#[br(little)]
pub enum PasmShapeType {
    #[br(magic = 0u32)]
    Sphere {
        #[br(pad_after = 12)]
        radius: f32,
    },

    #[br(magic = 1u32)]
    Cylinder {
        radius: f32,
        #[br(pad_after = 8)]
        height: f32,
    },

    #[br(magic = 2u32)]
    Box {
        #[br(pad_before = 4)]
        length: f32,
        width: f32,
        height: f32,
    },

    #[deprecated = "Deprecated baking into .wld, replaced by .cam files"]
    #[br(magic = 3u32)]
    Camera {
        fov: f32,
        frames: i32,
        offset_to_frames: i32,
        offset_to_string: i32,
    },

    #[deprecated = "Deprecated, superseded by sound_ambient_* gamedata in entities"]
    #[br(magic = 4u32)]
    Speaker {
        radius: f32,
        #[br(pad_after = 8)]
        unit_volume: f32,
    },

    /// Functionally equivalent to StartPoint
    #[br(magic = 5u32)]
    SpawnPoint {
        #[br(pad_size_to = 16)]
        _padding: (),
    },

    /// Functionally equivalent to StartPoint
    #[br(magic = 6u32)]
    StartPoint {
        #[br(pad_size_to = 16)]
        _padding: (),
    },

    #[deprecated = "AIRooms module deprecated, treated by PASM as APE_SHAPE_TYPE_BOX"]
    #[br(magic = 7u32)]
    Room {
        length: f32,
        width: f32,
        height: f32,
        room_id: i32,
    },

    #[deprecated = "Deprecated, treated by PASM as APE_SHAPE_TYPE_BOX"]
    #[br(magic = 8u32)]
    Arena {
        length: f32,
        width: f32,
        #[br(pad_after = 4)]
        height: f32,
    },

    #[deprecated = "Unimplimented in max exporter and treated by PASM as APE_SHAPE_TYPE_BOX"]
    #[br(magic = 9u32)]
    ParticleBox {
        length: f32,
        width: f32,
        #[br(pad_after = 4)]
        height: f32,
    },

    #[deprecated = "Unimplimented in max exporter and treated by PASM as APE_SHAPE_TYPE_SPHERE"]
    #[br(magic = 10u32)]
    ParticleSphere {
        #[br(pad_after = 12)]
        radius: f32,
    },

    #[deprecated = "Unimplimented in max exporter and treated by PASM as APE_SHAPE_TYPE_CYLINDER"]
    #[br(magic = 11u32)]
    ParticleCylinder {
        radius: f32,
        #[br(pad_after = 8)]
        height: f32,
    },

    #[br(magic = 12u32)]
    Spline {
        num_pts: i32,
        closed: i32,
        #[br(pad_after = 4)]
        num_segments: i32,
    },
}

/// Included at start of userData in PASMShape.userData when typeData = APE_SHAPE_TYPE_SPLINE
#[derive(BinRead, Debug)]
#[br(little)]
pub struct PasmSplinePt {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(BinRead, Debug)]
#[br(little)]
pub struct PasmShape {
    pub ty: PasmShapeType,
    pub mtx_orientation: MtxOrientation,
    pub user_data_length: u32,
    #[br(pad_after = 12)]
    pub parent_index: u32,
    #[br(count = user_data_length)]
    pub user_data: Vec<u8>,
}

#[derive(BinRead, Debug)]
#[br(little)]
pub struct PasmVec3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(BinRead, Debug)]
#[br(little)]
pub struct PasmVec2f {
    pub x: f32,
    pub y: f32,
}

//
#[derive(BinRead, Debug)]
#[br(little)]
pub struct PasmVisEdge {
    pub vert_index_1: i16,
    pub vert_index_2: i16,

    pub num_face: u32,

    pub face_index_1: i16,
    #[br(pad_after = 4)]
    pub face_index_2: i16,
}

#[derive(BinRead, Debug)]
#[br(little)]
pub struct PasmVisFace {
    pub degree: i32,
    #[br(count = 6)]
    pub vert_indices: Vec<i16>,
    #[br(count = 6)]
    pub edge_indices: Vec<i16>,

    pub normal: PasmVec3f,
    #[br(pad_after = 4)]
    pub centroid: PasmVec3f,
}

/// Visibilty Portal definition for designer placed sightline planes between volumes
#[derive(BinRead, Debug)]
#[br(little)]
pub struct PasmVisPortal {
    pub name: PaddedString<32>,

    #[br(count = 4)]
    pub corners: Vec<PasmVec3f>,

    pub normal: PasmVec3f,
    pub centroid: PasmVec3f,

    #[br(pad_after = 16)]
    pub flags: u32,
}

#[derive(BinRead, Debug)]
#[br(little)]
pub struct PasmSphere {
    #[br(count = 4)]
    pub sphere: Vec<f32>,
}

#[derive(BinRead, Debug)]
#[br(little)]
pub struct PasmCell {
    pub cell_name: PaddedString<32>,

    pub num_verts: u32,
    #[br(count = num_verts, pad_size_to = 12 /* Size of PasmVec3f */ * 156)]
    pub vis_verts: Vec<PasmVec3f>,

    pub num_edges: u32,
    #[br(count = num_edges, pad_size_to = 16 /* Size of PasmVisEdge */ * 79)]
    pub vis_edges: Vec<PasmVisEdge>,

    pub num_faces: u32,
    #[br(count = num_faces, pad_size_to = 56 /* Size of PasmVisFace */ * 26)]
    pub vis_faces: Vec<PasmVisFace>,

    pub sphere: PasmSphere,

    #[br(pad_after = 16)]
    pub flags: u32,
}

#[derive(BinRead, Debug)]
#[br(little)]
pub struct PasmVolume {
    pub num_cells: u32,
    #[br(count = num_cells, pad_size_to = 4672 /* Size of PasmCell */ * 16)]
    pub cells: Vec<PasmCell>,

    pub sphere: PasmSphere,
    #[br(pad_after = 16)]
    pub flags: u32,
}

#[derive(BinRead, Debug)]
#[br(little)]
pub struct PasmLightRgbi {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub i: f32,
}

#[derive(BinRead, Debug)]
#[br(little)]
pub struct PasmCommands {
    pub sort: i32,
    pub order_num: i32,
    pub shader_num: i32,
    pub emissive_motif_id: i32,
    pub specular_motif_id: i32,
    pub diffuse_motif_id: i32,
    pub use_emissive_color: i32,
    pub use_specular_color: i32,
    pub use_diffuse_color: i32,
    pub num_text_frames: i32,
    pub frames_per_second: f32,
    pub delta_u_per_second: f32,
    pub delta_v_per_second: f32,
    pub z_tug_value: i32,
    pub id: i8,
    pub no_coll: i8,
    pub coll_id: i8,
    pub flags: u8,
    pub coll_mask: u16,
    pub react_type: i16,
    #[br(pad_after = 2)]
    pub surface_type: i16,
    pub tint_rgb: PasmColor,
    pub light_rgbi: PasmLightRgbi,
    pub bump_map_tile_factor: f32,
    pub detail_map_tile_factor: f32,
    pub detail_uv_rotation_per_second: f32,
    #[br(pad_after = 12)]
    pub rotate_uv_around: PasmVec2f,
}

#[derive(BinRead, Debug)]
#[br(little, repr = u32)]
pub enum PasmLayerIndex {
    Diffuse = 0,
    SpecularMask = 1,
    EmissiveMask = 2,
    AlphaMask = 3,
    Bump = 4,
    Detail = 5,
    Environment = 6,
    Unused1 = 7,
    Unused2 = 8,
    Unused3 = 9,
    Max = 10,
}

/// A Material is made up of either 1 or 2 layers, base and layer1 respectfully
#[derive(BinRead, Debug)]
#[br(little)]
pub struct PasmLayer {
    pub textured: i32,
    #[br(count = 10)]
    pub tex_name: Vec<PaddedString<16>>,

    pub unit_alpha_multiplier: f32,
    pub draw_as_wire: i8,
    pub two_sided: i8,
    pub tile_u: i8,
    pub tile_v: i8,
    pub specular_rgb: PasmColor,
    pub illum_rgb: PasmColor,
    pub diffuse_rgb: PasmColor,
    pub shininess: f32,
    pub shin_str: f32,

    #[br(pad_after = 36)]
    pub star_commands: PasmCommands,
}

#[derive(BinRead, Debug)]
#[br(little)]
pub struct PasmMaterial {
    pub layer_count: i32,

    #[br(count = layer_count, pad_size_to = 380 /* Size of PasmLayer */ * 4)]
    pub layers: Vec<PasmLayer>,

    pub first_index: i32,
    pub num_indicies: i32,
    pub star_commands: PasmCommands,
    pub lod_index: i16,
    pub affect_angle: i16,
    #[br(pad_after = 24)]
    pub flags: i32,
}

#[derive(BinRead, Debug)]
#[br(little)]
pub struct PasmVert {
    pub pos: PasmVec3f,
    pub norm: PasmVec3f,
    pub color: PasmColor4,
    #[br(count = 4)]
    pub uvs: Vec<PasmVec2f>,

    pub num_weights: i32,
    #[br(count = 4, pad_size_to = 24 /* Size of PasmWeight */ * 4, pad_after = 16)]
    pub weights: Vec<PasmWeight>,
}

#[derive(BinRead, Debug)]
#[br(little)]
pub struct PasmVertIndex {
    #[br(pad_after = 16)]
    pub vert_index: i32,
}

#[derive(BinRead, Debug)]
#[br(little)]
pub struct PasmSegment {
    pub mesh_name: PaddedString<16>,
    pub skinned: i32,
    pub num_materials: i32,
    pub num_verts: i32,
    #[br(pad_after = 16)]
    pub num_indicies: i32,

    #[br(count = num_materials)]
    pub materials: Vec<PasmMaterial>,
    #[br(count = num_verts)]
    pub vertices: Vec<PasmVert>,
    #[br(count = num_indicies)]
    pub indicies: Vec<PasmVertIndex>,
}
