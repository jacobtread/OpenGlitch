use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::null_mut,
};
use swapbytes::SwapBytes;

use crate::formats::types::FixedString;

const FDATA_MESH_NAME_LENGTH: usize = 16;
const FDATA_MAX_LOD_MESH_COUNT: usize = 8;
const FDATA_VW_COUNT_PER_VTX: usize = 4;
const FDATA_BONE_NAME_LENGTH: usize = 32;

/// Trait implemented by structures directly casted from
/// memory buffers
pub trait MemoryStructure: Sized + SwapBytes {
    /// Load the structure from the provided buffer pointer
    /// and length of the buffer
    ///
    /// # Safety
    ///
    /// Safe as long as the input data is not incorrect (Aka its unsafe)
    unsafe fn load(buffer: Box<[u8]>) -> SafeBuffer<Self> {
        let ptr: *mut u8 = Box::into_raw(buffer).cast::<u8>();

        let mut buffer = SafeBuffer {
            // Cast the pointer type to the output type
            ptr: ptr.cast::<Self>(),
        };

        let value_ref = &mut *buffer;

        // Fix up the data, if the system isn't big endian must
        // swap the value endianess to little endian
        #[cfg(not(target_endian = "big"))]
        {
            value_ref.swap_bytes_mut();
        }
        value_ref.fix(ptr);

        buffer
    }

    /// Fix up the pointers on the structure
    ///
    /// # Safety
    ///
    /// Safe as long as the input data is not incorrect (Aka its unsafe)
    unsafe fn fix(&mut self, ptr: *mut u8);
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct CFSphere {
    pub radius: f32,
    pub position: CFVec3,
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct CFVec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C, align(16))]
pub struct CFMtx43A {
    pub matrix: [[f32; 3]; 4],
}

/// Pointer type for 32bit pointers
#[derive(SwapBytes, Clone, Copy)]
#[repr(C)]
pub struct Ptr32 {
    value: u32,
}

impl Ptr32 {
    /// Offsets the pointer by the provided pointer
    pub fn offset(&mut self, ptr: *mut u8) {
        if self.value == 0 {
            return;
        }

        self.value += ptr as u32;
    }

    pub fn ptr_mut<T>(&self) -> *mut T {
        self.value as *mut T
    }

    pub fn ptr<T>(&self) -> *const T {
        self.value as *const T
    }
}

impl Debug for Ptr32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ptr({:#08x})", self.value)
    }
}

impl Display for Ptr32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ptr({:#08x})", self.value)
    }
}

///  FMesh_t - This is the base struct that holds the mesh geometry
#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
#[non_exhaustive]
pub struct FMesh {
    /// ASCIIZ name of this mesh
    #[sb(skip)]
    pub name: FixedString<FDATA_MESH_NAME_LENGTH>,
    // Bounds the mesh in model space (might not be valid for skinned models)
    pub bound_sphere: CFSphere,
    pub bound_box_min: CFVec3,
    pub bound_box_max: CFVec3,

    pub flags: u16,
    /// Or'ing of all MasterCollMasks in the collision trees
    pub mesh_coll_mask: u16,
    /// Number of non-void bones which are located at the beginning of pBoneArray
    pub used_bone_count: u8,
    /// The index into the FMeshBone_t array of the root bone (255 if this mesh has no bones)
    pub root_bone_index: i8,
    /// Number of bones in this model (0 if none)
    pub bone_count: u8,
    /// Number of segments in this object
    pub segment_count: u8,
    /// Number of entries in pTexLayerIDArray
    pub tex_layer_id_count: u8,
    /// Number of entries in pTexLayerIDArray that have their FMESH_TEXLAYERIDFLAG_USE_ST_INFO flag set
    pub tex_layer_id_count_st: u8,
    /// Number of entries in pTexLayerIDArray that have their FMESH_TEXLAYERIDFLAG_USE_FLIP_INFO flag set
    pub tex_layer_id_count_flip: u8,
    /// Number of lights attached to this mesh
    pub light_count: u8,
    /// Number of materials in the material array (aMtl)
    pub material_count: u8,
    /// Number of elements in the collision tree array
    pub coll_tree_count: u8,
    /// Number of LOD meshes for this object
    pub lod_count: u8,
    /// Bias added to the current LOD for generating shadows
    pub shadow_lod_bias: u8,

    pub lod_distance: [f32; FDATA_MAX_LOD_MESH_COUNT],

    /// Base of segment array with public information
    pub segment_array: Ptr32,
    /// Pointer to bone array (number of elements is nBoneCount) (NULL if nBoneCount is 0)
    pub bone_array: Ptr32,
    /// Pointer to light array (number of elements is nLightCount) (NULL if nLightCount is 0)
    pub light_array: Ptr32,
    /// Pointer to the skeleton index array used by FMeshBone_t::Skelton.nChildArrayStartIndex
    pub skeleton_index_array: Ptr32,
    /// Pointer to the array of materials
    pub material_array: Ptr32,

    /// Pointer to an array of the mesh collision data structures (1 per segment)
    pub collision_tree: Ptr32,
    /// Texture layer ID array. Each slot matches up with a corresponding slot in each instance of this mesh.
    pub tex_layer_array: Ptr32,

    /// Pointer to implementation-specific object data
    pub mesh_is: Ptr32,
}

impl MemoryStructure for FMesh {
    unsafe fn fix(&mut self, ptr: *mut u8) {
        self.segment_array.offset(ptr);
        self.bone_array.offset(ptr);
        self.light_array.offset(ptr);
        self.skeleton_index_array.offset(ptr);
        self.material_array.offset(ptr);
        self.collision_tree.offset(ptr);
        self.tex_layer_array.offset(ptr);
        self.mesh_is.offset(ptr);
    }
}

#[derive(Debug, Clone, Copy, SwapBytes)]
pub struct FMeshSegment {
    /// Bounds the segment in model space
    pub bound_sphere: CFSphere,
    /// Number of simultaneous bone matrices used for vertices within this segment (1=segmented, but not skinned)
    pub bone_mtx_count: u8,
    /// Index into object instance's bone matrix palette (255=none)
    pub bone_mtx_index: [u8; FDATA_VW_COUNT_PER_VTX],
}

impl FMesh {
    pub fn segments(&mut self) -> &[FMeshSegment] {
        unsafe { std::slice::from_raw_parts(self.segment_array.ptr(), self.segment_count as usize) }
    }
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct FMeshBone {
    #[sb(skip)]
    pub name: FixedString<FDATA_BONE_NAME_LENGTH>,
    pub at_rest_bone_to_model: CFMtx43A,
    pub at_rest_model_to_bone: CFMtx43A,
    pub at_rest_parent_to_bone: CFMtx43A,
    pub at_rest_bone_to_parent: CFMtx43A,
    pub segmented_bound_sphere: CFSphere,
    pub skeleton: FMeshSkeleton,
    pub flags: u8,
    pub part_id: u8,
    padding: [u8; 3],
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct FMeshSkeleton {
    /// Bone index of this bone's parent (255 = no parent)
    pub parent_bone_index: u8,
    /// Number of children attached to this bone (0 = no children)
    pub child_bone_count: u8,
    /// Index into the array of bone indices (FMesh_t::pnSkeletonIndexArray) of where this bone's child index list begins
    pub child_array_start_index: u8,
}

/// Safe wrapper around a type created from a buffer to
/// ensure the memory is cleaned up, can be derefered
/// to access the inner type
pub struct SafeBuffer<T> {
    ptr: *mut T,
}

impl<T> SafeBuffer<T> {}

impl<T> Drop for SafeBuffer<T> {
    fn drop(&mut self) {
        // Recreate and drop the underlying memory
        let buffer = unsafe { Box::from_raw(self.ptr.cast::<u8>()) };
        drop(buffer);
    }
}

impl<T> Deref for SafeBuffer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}

impl<T> DerefMut for SafeBuffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr }
    }
}

pub struct FDX8Mesh_t {}

#[cfg(test)]
mod test {
    use std::fs::File;

    use crate::formats::mesh::raw::{MemoryStructure, SafeBuffer};

    use super::FMesh;

    #[test]
    fn test_load_ape() {
        // Read entire file into a buffer
        let buffer = std::fs::read("data/ape/gcdggltch00.ape").unwrap();
        // Drop extra buffer capacity
        let buffer: Box<[u8]> = buffer.into_boxed_slice();

        println!("Buffer length {}", buffer.len());

        let mesh: SafeBuffer<FMesh> = unsafe { FMesh::load(buffer) };

        // let value = unsafe { *mesh.skeleton_index_array };

        dbg!(&*mesh);
    }
}
