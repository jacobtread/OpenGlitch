use bevy::render::color::Color;
use bevy_rapier3d::parry::bounding_volume::{Aabb, BoundingSphere};
use nalgebra::Matrix4x3;

use super::mesh_raw::{LightType, MeshBoneFlags, MeshLightFlags};

#[derive(Debug)]
pub struct Mesh {
    pub name: String,
    pub bound_sphere: BoundingSphere,
    pub bound: Aabb,
    pub flags: u16,
    pub mesh_coll_mask: u16,
    pub root_bone_index: u16,

    pub used_bone_count: u16,

    pub segments: Vec<MeshSegment>,
    pub bones: Vec<MeshBone>,
    pub lights: Vec<MeshLight>,
}

#[derive(Debug)]
pub struct MeshBone {
    pub name: String,
    pub at_rest_bone_to_model: Matrix4x3<f32>,
    pub at_rest_model_to_bone: Matrix4x3<f32>,
    pub at_rest_parent_to_bone: Matrix4x3<f32>,
    pub at_rest_bone_to_parent: Matrix4x3<f32>,
    pub segment_bounded_sphere: BoundingSphere,
    pub flags: MeshBoneFlags,
    pub part_id: u8,
}

type BoneMatrixIndex = u8;

#[derive(Debug)]
pub struct MeshSegment {
    pub bounding_sphere: BoundingSphere,
    pub matrix_indicies: Vec<BoneMatrixIndex>,
}

#[derive(Debug)]
pub struct MeshLight {
    pub name: String,
    pub per_pixel_tex_name: String,
    pub corona_tex_name: String,
    pub flags: MeshLightFlags,
    pub light_id: u16,
    pub light_type: LightType,
    pub parent_bone_index: i8,
    pub indensity: f32,
    pub motif: Color,
    pub influence: BoundingSphere,
    pub orientation: Matrix4x3<f32>,
    pub spot_inner_radians: f32,
    pub spot_outer_radians: f32,
    pub corona_scale: f32,
}
