use binrw::{BinRead, FilePtr};

use super::types::{ApeSphere, ApeVec3f, FixedString, RawMatrix4x3f};

const FDATA_MESH_NAME_LENGTH: usize = 16;
const FDATA_MAX_LOD_MESH_COUNT: usize = 8;
const FDATA_BONE_NAME_LENGTH: usize = 32;

#[derive(Debug, BinRead)]
#[br(big)]
pub struct FMeshHeader {
    pub name: FixedString<FDATA_MESH_NAME_LENGTH>,

    pub bound_sphere: ApeSphere,
    pub bound_box_min: ApeVec3f,
    pub bound_box_max: ApeVec3f,

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

    pub segments_offset: u32,
    pub bones_offset: u32,
    pub lights_offset: u32,
    pub skeleton_index_array_offset: u32,
    pub materials_offset: u32,
    pub coll_tree_offset: u32,
    pub mesh_data_offset: u32,
}

#[derive(Debug, BinRead)]
#[br(big)]
pub struct FMesh {
    pub _a: u32,
}

const FDATA_VW_COUNT_PER_VTX: usize = 4;

#[derive(Debug, BinRead, Default)]
#[br(big)]
pub struct MeshSeg {
    pub bound_sphere: ApeSphere,
    pub bone_mtx_count: u8,
    pub bone_mtx_index: [u8; FDATA_VW_COUNT_PER_VTX],
}

#[derive(Debug, BinRead, Default)]
#[br(big)]
pub struct MeshBone {
    pub name: FixedString<FDATA_BONE_NAME_LENGTH>,
    pub at_rest_bone_to_model: RawMatrix4x3f,
    pub at_rest_model_to_bone: RawMatrix4x3f,
    pub at_rest_parent_to_bone: RawMatrix4x3f,
    pub at_rest_bone_to_parent: RawMatrix4x3f,
    pub segment_bounded_sphere: ApeSphere,
    pub skeleton: MeshSkeleton,
    pub flags: u8,
    #[br(pad_after = 3)]
    pub part_id: u8,
}

#[derive(Debug, BinRead, Default)]
#[br(big)]
pub struct MeshSkeleton {
    /// Bone index of this bone's parent (255 = no parent)
    pub parent_bone_index: u8,
    /// Number of children attached to this bone (0 = no children)
    pub child_bone_count: u8,
    /// Index into the array of bone indices (FMesh_t::pnSkeletonIndexArray) of where this bone's child index list begins
    pub child_array_start_index: u8,
}

#[derive(Debug, BinRead, Default)]
#[br(big)]
pub struct MeshLight {
    pub u: u32,
}

#[derive(Debug, BinRead, Default)]
#[br(big)]
pub struct MeshMaterial {
    pub u: u32,
}

#[cfg(test)]
mod test {
    use std::{fs::File, io::Seek, os::windows::fs::MetadataExt};

    use bevy::log::debug;
    use binrw::BinRead;

    use crate::formats::mesh::{FMeshHeader, MeshBone};

    use super::FMesh;

    #[test]
    fn test_load_mesh() {
        let mut file = File::open("data/ape/bridge01.ape").unwrap();
        let header: FMeshHeader = FMeshHeader::read(&mut file).unwrap();
        println!("Length: {}", file.metadata().unwrap().file_size());
        dbg!(&header);

        file.seek(std::io::SeekFrom::Start(header.bones_offset as u64));
        let bone: MeshBone = MeshBone::read(&mut file).unwrap();
        dbg!(&bone);
    }
}
