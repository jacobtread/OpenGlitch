use bitflags::bitflags;
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
const FLIGHT_NAME_LENGTH: usize = 16;
const FLIGHT_TEXTURE_NAME_LENGTH: usize = 16;
const FDATA_TEXNAME_LENGTH: usize = 16;

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
#[repr(C)]
pub struct CFVec2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C, align(16))]
pub struct CFMtx43A {
    pub matrix: [[f32; 3]; 4],
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct CFMtx43 {
    pub matrix: [[f32; 3]; 4],
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct CFColorRGB {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct CFColorRGBA {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct CFColorMotif {
    pub color: CFColorRGBA,
    pub motif_index: u32,
}

/// Pointer type for 32bit pointers
#[derive(SwapBytes, Clone, Copy)]
#[repr(C)]
pub struct Ptr<T> {
    value: u32,
    #[sb(skip)]
    type_data: PhantomData<*const T>,
}

impl<T> Ptr<T> {
    /// Offsets the pointer by the provided pointer
    pub fn offset(&mut self, ptr: *mut u8) {
        if self.value == 0 {
            return;
        }

        self.value += ptr as u32;
    }

    pub fn ptr_mut(&self) -> *mut T {
        self.value as *mut T
    }

    pub fn ptr(&self) -> *const T {
        self.value as *const T
    }
}

impl<T> Deref for Ptr<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr() }
    }
}

impl<T> DerefMut for Ptr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.ptr_mut() }
    }
}

impl<T> Debug for Ptr<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ptr({:#08x})", self.value)
    }
}

impl<T> Display for Ptr<T> {
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
    pub segment_array: Ptr<FMeshSegment>,
    /// Pointer to bone array (number of elements is nBoneCount) (NULL if nBoneCount is 0)
    pub bone_array: Ptr<FMeshBone>,
    /// Pointer to light array (number of elements is nLightCount) (NULL if nLightCount is 0)
    pub light_array: Ptr<FMeshLight>,
    /// Pointer to the skeleton index array used by FMeshBone_t::Skelton.nChildArrayStartIndex
    pub skeleton_index_array: Ptr<u8>,
    /// Pointer to the array of materials
    pub material_array: Ptr<FMeshMaterial>,

    /// Pointer to an array of the mesh collision data structures (1 per segment)
    pub collision_tree: Ptr<() /* FkDOP_Tree_t */>,
    /// Texture layer ID array. Each slot matches up with a corresponding slot in each instance of this mesh.
    pub tex_layer_array: Ptr<FMeshTexLayerID>,

    /// Pointer to implementation-specific object data
    pub mesh_is: Ptr<()>,
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
    pub fn segments(&self) -> &[FMeshSegment] {
        unsafe { std::slice::from_raw_parts(self.segment_array.ptr(), self.segment_count as usize) }
    }

    pub fn bones(&self) -> &[FMeshBone] {
        unsafe { std::slice::from_raw_parts(self.bone_array.ptr(), self.bone_count as usize) }
    }

    pub fn lights(&self) -> &[FMeshLight] {
        unsafe { std::slice::from_raw_parts(self.light_array.ptr(), self.light_count as usize) }
    }

    pub fn skeleton_index(&self, index: u8) -> u8 {
        let array: *const u8 = self.skeleton_index_array.ptr();
        unsafe { *array.offset(index as isize) }
    }

    pub fn materials(&self) -> &[FMeshMaterial] {
        unsafe {
            std::slice::from_raw_parts(self.material_array.ptr(), self.material_count as usize)
        }
    }

    pub fn tex_layers(&self) -> &[FMeshTexLayerID] {
        unsafe {
            std::slice::from_raw_parts(self.tex_layer_array.ptr(), self.tex_layer_id_count as usize)
        }
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

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct FMeshLight {
    // ASCIIZ name of the light
    #[sb(skip)]
    pub name: FixedString<FDATA_BONE_NAME_LENGTH>,

    // texture that is projected by this light.  MAKE SURE YOU NULL TERMINATE THIS, EVEN IF YOU DON"T WANT A TEXTURE
    #[sb(skip)]
    pub per_pixel_tex_name: FixedString<FLIGHT_TEXTURE_NAME_LENGTH>,
    // texture used to create the corona.  MAKE SURE YOU NULL TERMINATE THIS, EVEN IF YOU DON"T WANT A TEXTURE
    #[sb(skip)]
    pub corona_tex_name: FixedString<FLIGHT_TEXTURE_NAME_LENGTH>,

    // See FLIGHT_FLAG_* for info
    pub flags: u32,
    // LIGHT ID SHOULD BE SET TO 0xffff, UNLESS BEING SET FROM WITHIN A TOOL
    pub light_id: u16,
    // Light type (see FLightType_e for info)
    pub light_type: u8,
    // Index into the parent model's bone (-1 if there is no parent bone)
    pub parent_bone_index: i8,

    /// Light intensity to be multiplied by each component (0.0f to 1.0f)
    pub intensity: f32,
    /// Light color motif (RGBA components range from 0.0f to 1.0f). Alpha is not used.
    pub motif: CFColorMotif,
    /// Light position and radius in model space (ignored for directional lights)
    pub influence: CFSphere,
    /// Light orientation in model space (or world space if not attached to an object).  Direction (away from source) is in m_vFront (dir and spot)
    pub orientation: CFMtx43,

    /// Spotlight inner full-angle in radians
    pub spot_inner_radians: f32,
    /// Spotlight outer full-angle in radians
    pub spot_outer_radians: f32,

    pub corona_scale: f32,
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct FMeshMaterial {
    /// Pointer to shader's lighting register array
    pub shader_light_registers: Ptr<u32>,
    /// Pointer to shader's surface register array
    pub shader_surface_reigsters: Ptr<u32>,
    /// Light Shader index for this material
    pub light_shader_index: u8,
    /// Specular Shader index for this material
    pub specular_shader_index: u8,
    /// Surface Shader index for this material
    pub surface_shader_index: u16,
    /// A mask that has bits set for each mesh part ID that uses it
    pub part_id_mask: u32,
    /// Pointer to the platform specific data for this material
    pub platform_data: Ptr<()>,
    /// A bit mask that identifies all of the LOD that use this material
    pub lod_mask: u8,
    /// 0=normal, 1=appear in front of 0, 2=appear in front of 1, etc. (negative values not allowed)
    pub depth_bias_level: u8,
    pub base_st_sets: u8,
    pub light_map_st_sets: u8,
    /// Array of texture layer indices used by this material (255=empty slot) (fill lower elements first)
    /// Indices are into FMesh_t::pTexLayerIDArray[]
    pub tex_layer_id_index: [u8; 4],
    /// cos of angle of affect for angular emissive or angular translucency
    pub affect_angle: f32,
    /// Compressed affect normal used for determining material angle to camera (mult by 1/64)
    pub compressed_affect_normal: [i8; 3],
    /// Bone ID used to transform the affect angle
    pub affect_bone_id: i8,
    /// The radius of the material verts from the vAverageVertPos in model space represented as a percentage of
    /// the mesh's bounding sphere through a unit float compressed to a u8 (multiply by (1/255) * mesh BoundSphere_MS.m_fRadius)
    pub compressed_radius: u8,
    _pad: u8,
    /// Material flags (see FMESH_MTLFLAG_* for info)
    pub mtl_flags: u16,
    /// Key used by the engine to indicate that this material has already been submitted for drawing during the current viewport render
    pub draw_key: u32,
    /// Tint to be applied to the material
    pub material_tint: CFColorRGB,
    /// Average of the position of all verts using this material
    pub average_vert_pos: CFVec3,
    /// Hash key used in display list rendering (only valid in game)
    pub dl_hash_key: u32,
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct FMeshTexLayerID {
    pub tex_layer_id: u8,
    pub flags: u8,
    pub flip_page_count: u8,
    pub frames_per_flip: u8,
    pub flip_palette: Ptr<Ptr<CFTexInst>>,
    pub scroll_st_per_second: CFVec2,
    pub uv_degree_rotation_per_second: f32,
    pub compressed_uv_rot_anchor: [u8; 2],
    _padding: [u8; 2],
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct CFTexInst {
    /// Pointer to TexDef to use
    pub tex_def: Ptr<FTexDef>,
    /// Double buffer texture data for RenderTargets
    pub tex_buffer: Ptr<[FTexData; 2]>,
    pub buffer_index: u8,
    /// See FTEX_INSTFLAG_* for info
    pub flags: u32,
    /// 0.0f=normal, -1=bias by one smaller level, +1=bias by one larger level, etc.
    pub mipmap_bias: f32,
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct FTexDef {
    pub tex_info: FTexInfo,
    pub tex_data: Ptr<FTexData>,
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct FTexInfo {
    /// ASCIIZ name of texture
    #[sb(skip)]
    pub name: FixedString<FDATA_TEXNAME_LENGTH>,
    /// Pointer to user-defined data
    pub user_data: Ptr<()>,
    /// Texel format (See FTexFmt_e)
    pub tex_fmt: u8,
    /// Palette format (See FTexPalFmt_e)
    pub pal_fmt: u8,
    /// See FTEX_FLAG_* for info
    pub flags: u8,
    /// Number of LODs. 1=not mipmapped. >1 for mipmapped images
    pub lod_count: u8,
    /// For render targets, this is the number of bits in the stencil buffer
    pub render_target_stencil_bit_count: u8,
    /// For render targets, this is the number of bits in the depth buffer
    pub render_target_depth_bit_count: u8,

    _reserved: [u8; 2],
    /// Number of texels across of largest LOD image (always a power of 2)
    pub texels_across: u16,
    ///  Number of texels down of largest LOD image (always a power of 2)
    pub texels_down: u16,
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct FLink {
    pub prev_link: Ptr<FLink>,
    pub next_link: Ptr<FLink>,
}

const GX_TF_CTF: u32 = 0x20;
const GX_TF_ZTF: u32 = 0x10;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(u32)]
pub enum GxTexFmt {
    TF_I4 = 0x0,
    TF_I8 = 0x1,
    TF_IA4 = 0x2,
    TF_IA8 = 0x3,
    TF_RGB565 = 0x4,
    TF_RGB5A3 = 0x5,
    TF_RGBA8 = 0x6,
    TF_CMPR = 0xE,

    CTF_R4 = GX_TF_CTF,
    CTF_RA4 = 0x2 | GX_TF_CTF,
    CTF_RA8 = 0x3 | GX_TF_CTF,
    CTF_YUVA8 = 0x6 | GX_TF_CTF,
    CTF_A8 = 0x7 | GX_TF_CTF,
    CTF_R8 = 0x8 | GX_TF_CTF,
    CTF_G8 = 0x9 | GX_TF_CTF,
    CTF_B8 = 0xA | GX_TF_CTF,
    CTF_RG8 = 0xB | GX_TF_CTF,
    CTF_GB8 = 0xC | GX_TF_CTF,

    TF_Z8 = 0x1 | GX_TF_ZTF,
    TF_Z16 = 0x3 | GX_TF_ZTF,
    TF_Z24X8 = 0x6 | GX_TF_ZTF,

    CTF_Z4 = GX_TF_ZTF | GX_TF_CTF,
    CTF_Z8M = 0x9 | GX_TF_ZTF | GX_TF_CTF,
    CTF_Z8L = 0xA | GX_TF_ZTF | GX_TF_CTF,
    CTF_Z16L = 0xC | GX_TF_ZTF | GX_TF_CTF,
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    #[repr(C)]
    pub struct FTexDataFlags: u8 {
        const NONE    = 0x00;
        // Texture is created at runtime
        const RUNTIME = 0x01;
    }
}

impl SwapBytes for FTexDataFlags {
    fn swap_bytes_mut(&mut self) {
        let value = self.bits().swap_bytes();
        *self = FTexDataFlags::from_bits_retain(value);
    }
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct FTexData {
    /// Public texture definition
    pub tex_def: FTexDef,
    /// Link to other texture resources
    pub link: FLink,
    /// bitset of FGCTEXFLAGS_NONE, FGCTEXFLAGS_RUNTIME
    pub flags: FTexDataFlags,
    // LOD count
    pub lod_count: u8,
    // texture width
    pub width: u16,
    // texture height
    pub height: u16,
    // GC texel format used for this texture
    pub gc_tex_fmt: GxTexFmt,
    // Pointer to the GC platform texture object
    pub gc_tex_obj: Ptr<GCTexObj /* GCTexObj */>,
    // Set bits indicate which stages this texture is selected into (0=none)
    pub attached_stages: u32,
    // Approximate bytes consumed by this texture
    pub texture_bytes: u32,
    // Pointer to raw texture data
    pub raw_texture: Ptr<()>,
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct GCTexObj {
    _dummy: [u32; 8],
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
