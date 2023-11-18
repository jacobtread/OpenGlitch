use binrw::{BinRead, FilePtr};
use bitflags::bitflags;

use super::types::{
    FixedString, NullableFilePtr, PtrOffset, RawColorMotif, RawColorRGB, RawColorRGBA,
    RawMatrix4x3f, RawSphere, RawVec2f, RawVec3f,
};

const FDATA_MESH_NAME_LENGTH: usize = 16;
const FDATA_MAX_LOD_MESH_COUNT: usize = 8;
const FDATA_BONE_NAME_LENGTH: usize = 32;
const FDATA_VW_COUNT_PER_VTX: usize = 4;
const FLIGHT_NAME_LENGTH: usize = 16;
const FLIGHT_TEXTURE_NAME_LENGTH: usize = 16;

#[derive(Debug, BinRead)]
#[br(big)]
pub struct FMeshHeader {
    pub name: FixedString<FDATA_MESH_NAME_LENGTH>,

    pub bound_sphere: RawSphere,
    pub bound_box_min: RawVec3f,
    pub bound_box_max: RawVec3f,

    pub flags: u16,
    pub mesh_coll_mask: u16,

    pub used_bone_count: u8,
    pub root_bone_index: u8,
    pub bone_count: u8,
    pub seg_count: u8,
    pub tex_layer_id_count: u8,
    pub tex_layer_id_count_st: u8,
    pub tex_layer_id_count_flip: u8,
    pub light_count: u8,
    pub material_count: u8,
    pub coll_tree_count: u8,
    pub lod_count: u8,
    pub shadow_lod_bias: u8,

    pub lod_distance: [f32; FDATA_MAX_LOD_MESH_COUNT],

    #[br(args { count: seg_count as usize })]
    pub segments: NullableFilePtr<Vec<MeshSeg>>,

    #[br(args { count: bone_count as usize })]
    pub bones: NullableFilePtr<Vec<MeshBone>>,

    #[br(args { count: light_count as usize })]
    pub lights: NullableFilePtr<Vec<MeshLight>>,

    pub skeleton_index_array: PtrOffset,

    #[br(args { count: material_count as usize })]
    pub materials: NullableFilePtr<Vec<MeshMaterial>>,

    pub coll_tree: PtrOffset,

    #[br(args { count: tex_layer_id_count as usize })]
    pub tex_layer_id_array: NullableFilePtr<Vec<MeshTexLayerID>>,

    pub mesh_data: NullableFilePtr<GCMesh>,
}

#[derive(Debug, BinRead)]
#[br(big)]
pub struct GCMesh {
    /// Pointer to the base object (Ignored as its always null)
    pub _mesh: PtrOffset,

    pub at_rest_bound_sphere: RawSphere,

    pub flags: u8,
    pub vb_count: u8,
    pub mtl_count: u16,

    #[br(args { count: vb_count as usize })]
    pub vertex_buffers: NullableFilePtr<Vec<GCVertexBuffer>>,

    pub mesh_skin: NullableFilePtr<GCMeshSkin>,
}

#[derive(Debug, BinRead)]
#[br(big)]
pub struct GCMeshSkin {
    pub trans_desc_count: u16,
    pub td1_mtx_count: u16,
    pub td2_mtx_count: u16,
    pub td3_or_4mtx_count: u16,
    #[br(args { count: trans_desc_count as usize })]
    pub trans_desc: NullableFilePtr<Vec<GCTransDesc>>,
    pub skinned_verts_count: u32,
    #[br(args { count: skinned_verts_count as usize })]
    pub skinned_verts: NullableFilePtr<Vec<GCSkinPosNorm>>,
    #[br(args { count: skinned_verts_count as usize })]
    pub weights: NullableFilePtr<Vec<GCWeights>>,
}

#[derive(Debug, BinRead)]
#[br(big)]
pub struct GCSkinPosNorm {
    pub position: [i16; 3],
    pub normal: [i16; 3],
}

#[derive(Debug, BinRead)]
#[br(big)]
pub struct GCWeights {
    pub weights: [u8; 4],
}

/// Structure that describes transforms for skin.
#[derive(Debug, BinRead)]
#[br(big)]
pub struct GCTransDesc {
    #[br(pad_after = 1)]
    pub matrix_count: u8,
    pub vert_count: u16,
    pub mtx_index: [u8; 4],
}

#[derive(Debug, BinRead)]
#[br(big)]
pub struct GCColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

// Normal structure used for dynamic bump-mapping
#[derive(Debug, BinRead)]
#[br(big)]
pub struct GCNBT8 {
    pub normal: [i8; 3],
    pub binormal: [i8; 3],
    pub tangents: [i8; 3],
}

/// 8 bit UV's do not have enough resolution.  16 bit seems to be fine
#[derive(Debug, BinRead)]
#[br(big)]
pub struct GCST16 {
    pub s: i16,
    pub t: i16,
}

bitflags! {
    #[derive(Debug, BinRead, Clone, Copy)]
    #[br(map = Self::from_bits_retain)]
    pub struct GCVertexBufferFlags: u16 {
        const NONE     = 0x00;
        const SKINNED  = 0x01;
        const NORM_NBT = 0x10;
    }
}

// GameCube "vertex buffer" format
#[derive(Debug, BinRead)]
#[br(big)]
pub struct GCVertexBuffer {
    pub flags: GCVertexBufferFlags,

    pub pos_count: u16,
    pub pos_type: u8,
    pub pos_idx_type: u8,
    pub pos_stride: u8,
    pub pos_frac: u8,

    pub diffuse_count: u16,
    pub color_idx_type: u8,

    pub vertex_format: u8,

    pub position: PtrOffset,
    #[br(args { count: diffuse_count as usize })]
    pub diffuse: NullableFilePtr<Vec<GCColor>>,
    pub st: NullableFilePtr<GCST16>,
    pub nbt: NullableFilePtr<GCNBT8>,
}

#[derive(Debug, BinRead)]
#[br(big)]
pub struct MeshSeg {
    pub bound_sphere: RawSphere,
    pub bone_mtx_count: u8,
    pub bone_mtx_index: [u8; FDATA_VW_COUNT_PER_VTX],
}

#[derive(Debug, BinRead)]
#[br(big)]
pub struct MeshBone {
    pub name: FixedString<FDATA_BONE_NAME_LENGTH>,
    pub at_rest_bone_to_model: RawMatrix4x3f,
    pub at_rest_model_to_bone: RawMatrix4x3f,
    pub at_rest_parent_to_bone: RawMatrix4x3f,
    pub at_rest_bone_to_parent: RawMatrix4x3f,
    pub segment_bounded_sphere: RawSphere,
    pub skeleton: MeshSkeleton,
    pub flags: u8,
    #[br(pad_after = 3)]
    pub part_id: u8,
}

#[derive(Debug, BinRead)]
#[br(big)]
pub struct MeshSkeleton {
    /// Bone index of this bone's parent (255 = no parent)
    pub parent_bone_index: u8,
    /// Number of children attached to this bone (0 = no children)
    pub child_bone_count: u8,
    /// Index into the array of bone indices (FMesh_t::pnSkeletonIndexArray) of where this bone's child index list begins
    pub child_array_start_index: u8,
}

#[derive(Debug, BinRead)]
#[br(big)]
pub struct MeshLight {
    pub name: FixedString<FLIGHT_NAME_LENGTH>,
    pub per_pixel_tex_name: FixedString<FLIGHT_TEXTURE_NAME_LENGTH>,
    pub corona_tex_name: FixedString<FLIGHT_TEXTURE_NAME_LENGTH>,
    pub flags: u32,

    pub light_id: u16,
    pub light_type: u8,
    /// Index into the parent model's bone (-1 if there is no parent bone)
    pub parent_bone_index: i8,

    /// Light intensity to be multiplied by each component (0.0f to 1.0f)
    pub intensity: f32,
    /// Light color motif (RGBA components range from 0.0f to 1.0f). Alpha is not used.
    pub motif: RawColorMotif,
    /// Light position and radius in model space (ignored for directional lights)
    pub influence: RawSphere,
    /// Light orientation in model space (or world space if not attached to an object).  Direction (away from source) is in m_vFront (dir and spot)
    pub orientation: RawMatrix4x3f,

    /// Spotlight inner full-angle in radians
    pub spot_inner_radians: f32,
    /// Spotlight outer full-angle in radians
    pub spot_outer_radians: f32,

    pub corona_scale: f32,
}

#[derive(Debug, BinRead)]
#[br(big)]
pub struct MeshMaterial {
    pub shader_light_registers: PtrOffset,
    pub shader_surface_registers: PtrOffset,

    pub light_shader_index: u8,
    pub specular_shader_index: u8,
    pub surface_shader_index: u16,

    pub part_id_mask: u32,
    pub platform_data: PtrOffset,

    pub lod_mask: u8,
    pub depth_bias_level: u8,
    pub base_st_sets: u8,
    pub light_map_st_sets: u8,
    pub tex_layer_id_index: [u8; 4],
    pub affect_angle: f32,
    pub comp_affect_normal: [i8; 3],
    pub affect_bone_id: i8,

    #[br(pad_after = 1)]
    pub compressed_radius: u8,

    pub material_flags: u16,
    pub draw_key: u32,
    pub material_tint: RawColorRGB,
    pub average_vert_pos: RawVec3f,
    pub dl_hash_key: u32,
}

#[derive(Debug, BinRead)]
#[br(big)]
pub struct MeshTexLayerID {
    pub tex_layer_id: u8,
    pub flags: u8,

    pub flip_page_count: u8,
    pub frames_per_flip: u8,
    pub flip_palette: PtrOffset,

    pub scroll_st_per_second: RawVec2f,
    pub uv_degree_rotation_per_second: f32,
    #[br(pad_after = 2)]
    pub compresed_uv_rot_anchor: [u8; 2],
}

#[cfg(test)]
mod test {
    use std::{fs::File, io::Seek, os::windows::fs::MetadataExt};

    use bevy::log::debug;
    use binrw::BinRead;

    use crate::formats::mesh::{FMeshHeader, MeshBone};

    #[test]
    fn test_load_mesh() {
        let mut file = File::open("data/ape/bridge01.ape").unwrap();
        let header: FMeshHeader = FMeshHeader::read(&mut file).unwrap();
        println!("Length: {}", file.metadata().unwrap().file_size());
        dbg!(&header);
    }
}
