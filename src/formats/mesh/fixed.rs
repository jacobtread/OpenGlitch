use bevy_rapier3d::parry::bounding_volume::{Aabb, BoundingSphere, BoundingVolume};

#[derive(Debug)]
pub struct FMesh {
    pub name: String,
    pub bound_sphere: BoundingSphere,
    pub bound_box: Aabb,
    pub flags: u16,
    pub mesh_coll_mask: u16,
    pub used_bone_count: u8,
    pub root_bone_index: i8,
    pub shadow_lod_bias: u8,
    pub lod_distance: Vec<f32>,
}
