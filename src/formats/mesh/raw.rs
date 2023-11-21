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

/// Load the structure from the provided buffer pointer
/// and length of the buffer
///
/// # Safety
///
/// Safe as long as the input data is not incorrect (Aka its unsafe)
unsafe fn load_memory_struct<T>(buffer: Box<[u8]>) -> SafeBuffer<T>
where
    T: Sized + SwapBytes + FixOffsets,
{
    let ptr: *mut u8 = Box::into_raw(buffer).cast::<u8>();

    let mut buffer = SafeBuffer {
        // Cast the pointer type to the output type
        ptr: ptr.cast::<T>(),
    };

    let value_ref = &mut *buffer;

    // Fix up the data, if the system isn't big endian must
    // swap the value endianess to little endian
    #[cfg(not(target_endian = "big"))]
    {
        value_ref.swap_bytes_mut();
    }
    value_ref.fix_offsets(ptr);

    buffer
}

/// Trait implemented by structures that need to fix their
/// pointer offsets
pub trait FixOffsets {
    /// Fix up the pointers on the structure
    fn fix_offsets(&mut self, ptr: *mut u8);
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
pub struct FGCColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
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

impl<T> Ptr<T>
where
    T: FixOffsets,
{
    pub fn offset_and_fix(&mut self, ptr: *mut u8) {
        self.offset(ptr);
        self.try_fix_offsets(ptr);
    }

    pub fn try_fix_offsets(&mut self, ptr: *mut u8) {
        if let Some(value) = self.try_deref_mut() {
            value.fix_offsets(ptr);
        }
    }

    pub fn offset_and_fix_slice(&mut self, length: usize, ptr: *mut u8) {
        self.offset(ptr);
        self.try_fix_slice_offsets(length, ptr);
    }

    pub fn try_fix_slice_offsets(&mut self, length: usize, ptr: *mut u8) {
        if let Some(value) = self.try_as_slice_mut(length) {
            value.iter_mut().for_each(|value| value.fix_offsets(ptr));
        }
    }
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

    pub fn try_deref(&self) -> Option<&T> {
        if self.is_null() {
            return None;
        }
        let value = unsafe { &*(self.value as *const T) };
        Some(value)
    }

    pub fn try_deref_mut(&self) -> Option<&mut T> {
        if self.is_null() {
            return None;
        }
        let value = unsafe { &mut *(self.value as *mut T) };
        Some(value)
    }

    pub fn is_null(&self) -> bool {
        self.value == 0
    }

    /// Treats the pointer as a slice of the provided length
    pub fn as_slice(&self, length: usize) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.ptr(), length) }
    }

    /// Treats the pointer as a mutable slice of the provided length
    pub fn as_slice_mut(&mut self, length: usize) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.ptr_mut(), length) }
    }

    /// Treats the pointer as a slice of the provided length
    pub fn try_as_slice(&self, length: usize) -> Option<&[T]> {
        if self.is_null() {
            return None;
        }
        Some(unsafe { std::slice::from_raw_parts(self.ptr(), length) })
    }

    /// Treats the pointer as a mutable slice of the provided length
    pub fn try_as_slice_mut(&mut self, length: usize) -> Option<&mut [T]> {
        if self.is_null() {
            return None;
        }
        Some(unsafe { std::slice::from_raw_parts_mut(self.ptr_mut(), length) })
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
    bone_count: u8,
    /// Number of segments in this object
    segment_count: u8,
    /// Number of entries in pTexLayerIDArray
    tex_layer_id_count: u8,

    /// Number of entries in pTexLayerIDArray that have their FMESH_TEXLAYERIDFLAG_USE_ST_INFO flag set
    _tex_layer_id_count_st: u8,
    /// Number of entries in pTexLayerIDArray that have their FMESH_TEXLAYERIDFLAG_USE_FLIP_INFO flag set
    _tex_layer_id_count_flip: u8,

    /// Number of lights attached to this mesh
    light_count: u8,
    /// Number of materials in the material array (aMtl)
    material_count: u8,
    /// Number of elements in the collision tree array
    coll_tree_count: u8,
    /// Number of LOD meshes for this object
    lod_count: u8,
    /// Bias added to the current LOD for generating shadows
    pub shadow_lod_bias: u8,

    pub lod_distance: [f32; FDATA_MAX_LOD_MESH_COUNT],

    /// Base of segment array with public information
    segment_array: Ptr<FMeshSegment>,
    /// Pointer to bone array (number of elements is nBoneCount) (NULL if nBoneCount is 0)
    bone_array: Ptr<FMeshBone>,
    /// Pointer to light array (number of elements is nLightCount) (NULL if nLightCount is 0)
    light_array: Ptr<FMeshLight>,
    /// Pointer to the skeleton index array used by FMeshBone_t::Skelton.nChildArrayStartIndex
    skeleton_index_array: Ptr<u8>,
    /// Pointer to the array of materials
    material_array: Ptr<FMeshMaterial>,

    /// Pointer to an array of the mesh collision data structures (1 per segment)
    collision_tree: Ptr<() /* FkDOP_Tree_t */>,
    /// Texture layer ID array. Each slot matches up with a corresponding slot in each instance of this mesh.
    tex_layer_array: Ptr<FMeshTexLayerID>,

    /// Pointer to implementation-specific object data
    mesh_is: Ptr<FGCMesh>,
}

impl FixOffsets for FMesh {
    fn fix_offsets(&mut self, ptr: *mut u8) {
        self.segment_array.offset(ptr);
        self.bone_array.offset(ptr);
        self.light_array.offset(ptr);
        self.skeleton_index_array.offset(ptr);
        self.collision_tree.offset(ptr);

        self.material_array
            .offset_and_fix_slice(self.material_count as usize, ptr);

        // TODO: Fixup coll tree

        self.tex_layer_array
            .offset_and_fix_slice(self.tex_layer_id_count as usize, ptr);

        self.mesh_is.offset_and_fix(ptr);
    }
}

impl FMesh {
    pub fn lod_distances(&self) -> &[f32] {
        &self.lod_distance[..self.lod_count as usize]
    }

    pub fn segments(&self) -> &[FMeshSegment] {
        self.segment_array.as_slice(self.segment_count as usize)
    }

    pub fn bones(&self) -> &[FMeshBone] {
        self.bone_array.as_slice(self.bone_count as usize)
    }

    pub fn lights(&self) -> &[FMeshLight] {
        self.light_array.as_slice(self.light_count as usize)
    }

    pub fn skeleton_index(&self, index: u8) -> u8 {
        let array: *const u8 = self.skeleton_index_array.ptr();
        unsafe { *array.offset(index as isize) }
    }

    pub fn materials(&self) -> &[FMeshMaterial] {
        self.material_array.as_slice(self.material_count as usize)
    }

    pub fn tex_layers(&self) -> &[FMeshTexLayerID] {
        self.tex_layer_array
            .as_slice(self.tex_layer_id_count as usize)
    }
    pub fn collision_tree(&self) -> &[()] {
        self.collision_tree.as_slice(self.coll_tree_count as usize)
    }

    pub fn impl_specific(&self) -> Option<&FGCMesh> {
        self.mesh_is.try_deref()
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
    pub platform_data: Ptr<FGCMeshMaterial>,
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

impl FixOffsets for FMeshMaterial {
    fn fix_offsets(&mut self, ptr: *mut u8) {
        self.shader_light_registers.offset(ptr);
        self.shader_surface_reigsters.offset(ptr);

        self.platform_data.offset_and_fix(ptr);

        // TODO: Fixup registers

        // TODO: Fix hash key
    }
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct FMeshTexLayerID {
    pub tex_layer_id: u8,
    pub flags: u8,
    pub flip_page_count: u8,
    pub frames_per_flip: u8,
    // Pointer to flip palette (array of CFTexInst pointers). Number of palette entries is nFlipPageCount.
    pub flip_palette: Ptr<Ptr<CFTexInst>>,
    pub scroll_st_per_second: CFVec2,
    pub uv_degree_rotation_per_second: f32,
    pub compressed_uv_rot_anchor: [u8; 2],
    _padding: [u8; 2],
}

impl FixOffsets for FMeshTexLayerID {
    fn fix_offsets(&mut self, ptr: *mut u8) {
        self.flip_palette.offset(ptr);
        if let Some(value) = self
            .flip_palette
            .try_as_slice_mut(self.flip_page_count as usize)
        {
            value.iter_mut().for_each(|value| {
                value.offset(ptr);
                value.try_fix_offsets(ptr);
            })
        }
    }
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct CFTexInst {
    /// Pointer to TexDef to use
    pub tex_def: Ptr<FTexDef>,
    /// Double buffer texture data for RenderTargets
    pub tex_buffer: [Ptr<FTexData>; 2],
    pub buffer_index: u8,
    /// See FTEX_INSTFLAG_* for info
    pub flags: u32,
    /// 0.0f=normal, -1=bias by one smaller level, +1=bias by one larger level, etc.
    pub mipmap_bias: f32,
}

impl FixOffsets for CFTexInst {
    fn fix_offsets(&mut self, ptr: *mut u8) {
        self.tex_def.offset_and_fix(ptr);

        self.tex_buffer
            .iter_mut()
            .for_each(|value| value.offset_and_fix(ptr));
    }
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct FTexDef {
    pub tex_info: FTexInfo,
    pub tex_data: Ptr<FTexData>,
}

impl FixOffsets for FTexDef {
    fn fix_offsets(&mut self, ptr: *mut u8) {
        self.tex_info.fix_offsets(ptr);
        self.tex_data.offset_and_fix(ptr);
    }
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

impl FixOffsets for FTexInfo {
    fn fix_offsets(&mut self, ptr: *mut u8) {
        self.user_data.offset(ptr);
    }
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct FLink {
    pub prev_link: Ptr<FLink>,
    pub next_link: Ptr<FLink>,
}

impl FixOffsets for FLink {
    fn fix_offsets(&mut self, ptr: *mut u8) {
        self.prev_link.offset(ptr);
        self.next_link.offset(ptr);
    }
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
    pub gc_tex_obj: Ptr<GCTexObj>,
    // Set bits indicate which stages this texture is selected into (0=none)
    pub attached_stages: u32,
    // Approximate bytes consumed by this texture
    pub texture_bytes: u32,
    // Pointer to raw texture data
    pub raw_texture: Ptr<()>,
}

impl FixOffsets for FTexData {
    fn fix_offsets(&mut self, ptr: *mut u8) {
        self.tex_def.fix_offsets(ptr);
        self.link.fix_offsets(ptr);
        self.gc_tex_obj.offset(ptr);
        self.raw_texture.offset(ptr);
    }
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct GCTexObj {
    _dummy: [u32; 8],
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct FGCMesh {
    // Pointer to platform-independent base object (Always null unless set when loaded)
    _mesh: Ptr<FMesh>,

    /// Used only when nSegCount is 0
    pub at_rest_bound_sphere: CFSphere,
    ///See FGCMESH_FLAG_* for info
    pub flags: u8,
    // Number of vertex buffers used by this mesh
    vb_count: u8,
    // Number of materials in this node
    pub mtl_count: u16,
    // Array of vertex buffer descriptors
    vb: Ptr<FGCVB>,
    // Pointer to the mesh skin, if there is one
    pub mesh_skin: Ptr<FGCMeshSkin>,
}

impl FixOffsets for FGCMesh {
    fn fix_offsets(&mut self, ptr: *mut u8) {
        self.vb.offset_and_fix_slice(self.vb_count as usize, ptr);
        self.mesh_skin.offset_and_fix(ptr);
    }
}

impl FGCMesh {
    pub fn vertex_buffers(&self) -> &[FGCVB] {
        self.vb.as_slice(self.vb_count as usize)
    }

    pub fn vertex_buffers_mut(&mut self) -> &mut [FGCVB] {
        self.vb.as_slice_mut(self.vb_count as usize)
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    #[repr(C)]
    pub struct FGCDLContFlags: u8 {
        const NONE    = 0x00;
        const SKINNED = 0x01;
        const CONSTANT_COLOR = 0x02;
        const BUMPMAP = 0x04;
        const FACING_OPP_DIR_LIGHT =0x08;
        const STREAMING = 0x80;
    }
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct FGCDLCont {
    // See display list flags, above
    #[sb(skip)]
    pub flags: FGCDLContFlags,
    // Matrix index for this display list
    pub matrix_idc: u8,
    // ID for the LOD this display list is part of (0 is closest)
    pub lod_id: u8,
    pub part_id: u8,
    // Number of stripped triangles in this display list
    pub strip_tri_count: u16,
    // Number of list triangles in this display list
    pub list_tri_count: u16,
    // Number of tri strips
    pub strip_count: u16,
    // Number of tri lists
    pub list_count: u8,
    // Index into the mesh's vertex buffers indicating which
    pub vb_index: u8,
    // size of buffer
    pub size: u32,
    pub buffer: Ptr<()>,
    pub constant_color: FGCColor,
}

impl FixOffsets for FGCDLCont {
    fn fix_offsets(&mut self, ptr: *mut u8) {
        self.buffer.offset(ptr)
    }
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct FGCMeshMaterial {
    // Array of display list containers for this material
    dl_container: Ptr<FGCDLCont>,
    // Number of display list containers used
    dl_cont_count: u16,
}

impl FixOffsets for FGCMeshMaterial {
    fn fix_offsets(&mut self, ptr: *mut u8) {
        self.dl_container.offset(ptr);
        if let Some(value) = self.display_containers_mut() {
            value.iter_mut().for_each(|value| {
                if value.flags.contains(FGCDLContFlags::STREAMING) {
                    // TODO: handle streaming
                } else {
                    value.buffer.offset(ptr);
                }
            });
        }
    }
}

impl FGCMeshMaterial {
    #[inline]
    pub fn display_containers(&self) -> Option<&[FGCDLCont]> {
        self.dl_container.try_as_slice(self.dl_cont_count as usize)
    }

    #[inline]
    pub fn display_containers_mut(&mut self) -> Option<&mut [FGCDLCont]> {
        self.dl_container
            .try_as_slice_mut(self.dl_cont_count as usize)
    }
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct FGCMeshSkin {
    // Number of skin translations
    pub tran_desc_count: u16,
    // Number of verts weighted to 1 matrix
    pub td1_mtx_count: u16,
    // Number of verts weighted to 2 matrices
    pub td2_mtx_count: u16,
    // Number of verts weighted to 3 or 4 matrices
    pub td3_or_4mtx_count: u16,
    // Pointer to the array of skin translations descriptions
    pub trans_desc: Ptr<FGCTransDesc>,
    // Number of skinned Verts
    pub skinned_verts_count: u32,
    // Pointer to the array of skinned verts
    pub skinned_verts: Ptr<FGCSkinPosNorm>,
    // Pointer to the array of weights (one to one correspondence with position)
    pub skin_weights: Ptr<FGCWeights>,
}

impl FixOffsets for FGCMeshSkin {
    fn fix_offsets(&mut self, ptr: *mut u8) {
        self.trans_desc.offset(ptr);
        self.skinned_verts.offset(ptr);
        self.skin_weights.offset(ptr);
    }
}

impl FGCMeshSkin {
    pub fn trans_desc(&self) -> &[FGCTransDesc] {
        self.trans_desc.as_slice(self.tran_desc_count as usize)
    }

    pub fn skinned_verts(&self) -> &[FGCSkinPosNorm] {
        self.skinned_verts
            .as_slice(self.skinned_verts_count as usize)
    }
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct FGCTransDesc {
    pub matrix_count: u8,
    _pad: u8,
    pub vert_count: u16,
    pub mtx_ids: [u8; 4],
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct FGCSkinPosNorm {
    pub position: [i16; 3],
    pub normal: [i16; 3],
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct FGCWeights {
    pub weights: [u8; 4],
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    #[repr(C)]
    pub struct FGCVBFlags: u16 {
        const NONE     = 0x00;
        /// If skinned, the position and normal are presumed 48-bits each
        const SKINNED  = 0x01; // position and normal composed of s16's
        /// We assume fixed-point 16-bit normal, unless this flag is set:
        const NORM_NBT = 0x10;	// normal has binormal and tangent for bump-mapping
    }
}

impl SwapBytes for FGCVBFlags {
    fn swap_bytes_mut(&mut self) {
        let value = self.bits().swap_bytes();
        *self = Self::from_bits_retain(value);
    }
}

// 8 bit UV's do not have enough resolution.  16 bit seems to be fine
#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct FGCST16 {
    pub s: i16,
    pub t: i16,
}

// Normal structure used for dynamic bump-mapping
#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct FGCNBT8 {
    pub n: [i8; 3],
    pub b: [i8; 3],
    pub t: [i8; 3],
}

/// GameCube "vertex buffer" format
#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct FGCVB {
    pub flags: FGCVBFlags,
    // Number of positions in this vertex buffer
    pub pos_count: u16,
    // GC pos type (GX_F32, GX_S16, or GX_S8)
    pub pos_type: u8,
    // GC position index type (GX_INDEX8 or GX_INDEX16)
    pub pos_idx_type: u8,
    // Byte size of the position vector
    pub pos_stride: u8,
    // Number of bits in the fractional component of position
    pub pos_frac: u8,

    // Number of unique diffuse colors in this vertex buffer
    pub diffuse_count: u16,
    // GC color index type (GX_INDEX8 or GX_INDEX16)
    pub color_idx_type: u8,

    pub gc_vertex_format: u8,

    // Pointer to the position data (in the case of skinned, position and normal)
    pub position: Ptr<()>,
    // Pointer to the diffuse color data (if any)
    pub diffuse: Ptr<FGCColor>,
    // Pointer to the ST data
    pub st: Ptr<FGCST16>,
    // For bumpmapped objects, Pointer to the normal, binormal and tangents
    pub nbt: Ptr<FGCNBT8>,
}

impl FixOffsets for FGCVB {
    fn fix_offsets(&mut self, ptr: *mut u8) {
        self.position.offset(ptr);
        self.nbt.offset(ptr);
        self.diffuse.offset(ptr);
        self.st.offset(ptr);
    }
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

    use crate::formats::mesh::raw::{load_memory_struct, SafeBuffer};

    use super::FMesh;

    #[test]
    fn test_load_ape() {
        // Read entire file into a buffer
        let buffer = std::fs::read("data/ape/gcdggltch00.ape").unwrap();
        // Drop extra buffer capacity
        let buffer: Box<[u8]> = buffer.into_boxed_slice();

        println!("Buffer length {}", buffer.len());

        let mesh: SafeBuffer<FMesh> = unsafe { load_memory_struct::<FMesh>(buffer) };

        // let value = unsafe { *mesh.skeleton_index_array };

        dbg!(&*mesh);
    }
}
