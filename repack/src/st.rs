use bitflags::bitflags;
use std::ops::{Deref, DerefMut};
use swapbytes::SwapBytes;

use crate::{
    raw::dx::{DxMesh, DxMeshMaterial},
    types::FixedString,
};

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
pub unsafe fn load_memory_struct<T>(buffer: Box<[u8]>) -> SafeBuffer<T>
where
    T: Sized + SwapBytes + Fixable,
{
    let ptr: *mut u8 = Box::into_raw(buffer).cast::<u8>();

    let mut buffer = SafeBuffer {
        // Cast the pointer type to the output type
        ptr: ptr.cast::<T>(),
    };

    let value_ref = &mut *buffer;

    value_ref.fix(ptr);

    buffer
}

/// Trait implemented by structures that need to fix their
/// pointer offsets
pub trait Fixable: SwapBytes {
    /// # Safety
    ///
    /// This is not safe, it relies on the values present in the compiled game
    /// assets being correct, that is the only assurance of correctness
    unsafe fn fix(&mut self, ptr: *mut u8) {
        #[cfg(not(target_endian = "little"))]
        {
            self.swap_bytes_mut();
        }

        self.fix_offset(ptr);
    }

    /// Fix up the pointers on the structure and fix any
    /// incorrect details
    ///
    /// # Safety
    ///
    /// This is not safe, it relies on the values present in the compiled game
    /// assets being correct, that is the only assurance of correctness
    unsafe fn fix_offset(&mut self, _ptr: *mut u8) {}
}

/// Pointers stored in the memory structure are indexed from
/// zero and need to be offset to the actual memory location
///
/// # Safety
///
/// This is not safe, it relies on the values present in the compiled game
/// assets being correct, that is the only assurance of correctness
///
/// ## Arguments
/// * offset - The original pointer stored in the structure (Just the offset)
/// * ptr    - The memory starting pointer to add the offset onto
pub unsafe fn fix_offset<T>(offset: *mut T, ptr: *mut u8) -> *mut T {
    // Don't offset null pointers
    if offset.is_null() {
        return offset;
    }

    ptr.byte_offset(offset as isize).cast()
}

/// Casts the provided array pointer to a slice of the
/// provided length, will return None if the pointer is
/// a null pointer
///
/// # Safety
///
/// This is not safe, it relies on the values present in the compiled game
/// assets being correct, that is the only assurance of correctness
pub unsafe fn array_ptr<T, L>(ptr: *const T, length: L) -> Option<&'static [T]>
where
    L: Into<usize>,
    T: 'static,
{
    if ptr.is_null() {
        return None;
    }

    let slice = std::slice::from_raw_parts(ptr, length.into());
    Some(slice)
}

/// Casts the provided array pointer to a mutable slice of the
/// provided length, will return None if the pointer is
/// a null pointer
///
/// # Safety
///
/// This is not safe, it relies on the values present in the compiled game
/// assets being correct, that is the only assurance of correctness
pub unsafe fn array_ptr_mut<T, L>(ptr: *mut T, length: L) -> Option<&'static mut [T]>
where
    L: Into<usize>,
    T: 'static,
{
    if ptr.is_null() {
        return None;
    }

    let slice = std::slice::from_raw_parts_mut(ptr, length.into());
    Some(slice)
}

/// Attempts to fix the offsets of the value on the other
/// side of the provided value pointer and the pointer itself
/// if the pointer is not null
///
/// # Safety
///
/// This is not safe, it relies on the values present in the compiled game
/// assets being correct, that is the only assurance of correctness
pub unsafe fn try_fix<T>(value: &mut *mut T, ptr: *mut u8)
where
    T: Fixable,
{
    // Fix the pointer itself
    *value = fix_offset(*value, ptr);

    // Try fix the value on the other side of the pointer
    if let Some(value) = unsafe { value.as_mut() } {
        value.fix(ptr)
    }
}

/// Attempts to fix the offsets of all the values in an array at `value` of
/// the provided `length` if the `value` pointer is not null
///
/// # Safety
///
/// This is not safe, it relies on the values present in the compiled game
/// assets being correct, that is the only assurance of correctness
pub unsafe fn try_fix_array<T, L>(value: &mut *mut T, length: L, ptr: *mut u8)
where
    T: Fixable,
    L: Into<usize> + Copy,
    T: 'static,
{
    // Fix the pointer itself
    *value = fix_offset(*value, ptr);

    // Try fix the elements
    if let Some(array) = array_ptr_mut(*value, length) {
        println!("Fixing array {} {}", ptr as usize, *value as usize);
        array.iter_mut().for_each(|value| value.fix(ptr));
    }
}

/// Attempts to fix the offsets of all the values in an array at `value` of
/// the provided `length` if the `value` pointer is not null
///
/// # Safety
///
/// This is not safe, it relies on the values present in the compiled game
/// assets being correct, that is the only assurance of correctness
pub unsafe fn try_ptr_fix_array<T, L>(value: &mut *mut *mut T, length: L, ptr: *mut u8)
where
    T: Fixable,
    L: Into<usize> + Copy,
    T: 'static,
{
    // Fix the pointer itself
    *value = fix_offset(*value, ptr);

    // Try fix the elements
    if let Some(array) = array_ptr_mut(*value, length) {
        array.iter_mut().for_each(|value| {
            try_fix(value, ptr);
        });
    }
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
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct FGCColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl SwapBytes for FGCColor {
    fn swap_bytes_mut(&mut self) {}
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
    segment_array: *mut FMeshSegment,
    /// Pointer to bone array (number of elements is nBoneCount) (NULL if nBoneCount is 0)
    bone_array: *mut FMeshBone,
    /// Pointer to light array (number of elements is nLightCount) (NULL if nLightCount is 0)
    light_array: *mut FMeshLight,
    /// Pointer to the skeleton index array used by FMeshBone_t::Skelton.nChildArrayStartIndex
    skeleton_index_array: *mut u8,
    /// Pointer to the array of materials
    material_array: *mut FMeshMaterial,

    /// Pointer to an array of the mesh collision data structures (1 per segment)
    collision_tree: *mut (), /* FkDOP_Tree_t */
    /// Texture layer ID array. Each slot matches up with a corresponding slot in each instance of this mesh.
    tex_layer_array: *mut FMeshTexLayerID,

    /// Pointer to implementation-specific object data
    mesh_is: *mut DxMesh,
}

impl Fixable for FMesh {
    unsafe fn fix_offset(&mut self, ptr: *mut u8) {
        self.segment_array = fix_offset(self.segment_array, ptr);
        self.bone_array = fix_offset(self.bone_array, ptr);
        self.light_array = fix_offset(self.light_array, ptr);
        self.skeleton_index_array = fix_offset(self.skeleton_index_array, ptr);
        self.collision_tree = fix_offset(self.collision_tree, ptr);
        println!("Premat");

        try_fix_array(&mut self.material_array, self.material_count, ptr);
        println!("Postmat");

        // TODO: Fixup coll tree

        try_fix_array(&mut self.tex_layer_array, self.tex_layer_id_count, ptr);

        try_fix(&mut self.mesh_is, ptr);
    }
}

impl FMesh {
    pub fn lod_distances(&self) -> &[f32] {
        &self.lod_distance[..self.lod_count as usize]
    }

    pub fn segments(&self) -> Option<&[FMeshSegment]> {
        unsafe { array_ptr(self.segment_array, self.segment_count) }
    }

    pub fn bones(&self) -> Option<&[FMeshBone]> {
        unsafe { array_ptr(self.bone_array, self.bone_count) }
    }

    pub fn lights(&self) -> Option<&[FMeshLight]> {
        unsafe { array_ptr(self.light_array, self.light_count) }
    }

    pub fn skeleton_index(&self, index: u8) -> Option<u8> {
        if self.skeleton_index_array.is_null() {
            return None;
        }

        let value = unsafe { *self.skeleton_index_array.offset(index as isize) };
        Some(value)
    }

    pub fn materials(&self) -> Option<&[FMeshMaterial]> {
        unsafe { array_ptr(self.material_array, self.material_count) }
    }

    pub fn tex_layers(&self) -> Option<&[FMeshTexLayerID]> {
        unsafe { array_ptr(self.tex_layer_array, self.tex_layer_id_count) }
    }
    pub fn collision_tree(&self) -> Option<&[()]> {
        unsafe { array_ptr(self.collision_tree, self.coll_tree_count) }
    }

    pub fn impl_specific(&self) -> Option<&DxMesh> {
        unsafe { self.mesh_is.as_ref() }
    }
    pub fn impl_specific_mut(&self) -> Option<&mut DxMesh> {
        unsafe { self.mesh_is.as_mut() }
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
    pub shader_light_registers: *mut u32,
    /// Pointer to shader's surface register array
    pub shader_surface_reigsters: *mut u32,
    /// Light Shader index for this material
    pub light_shader_index: u8,
    /// Specular Shader index for this material
    pub specular_shader_index: u8,
    /// Surface Shader index for this material
    pub surface_shader_index: u16,
    /// A mask that has bits set for each mesh part ID that uses it
    pub part_id_mask: u32,
    /// Pointer to the platform specific data for this material
    pub platform_data: *mut DxMeshMaterial,
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
    _dl_hash_key: u32,
}

impl Fixable for FMeshMaterial {
    unsafe fn fix_offset(&mut self, ptr: *mut u8) {
        println!("Shr");
        self.shader_light_registers = fix_offset(self.shader_light_registers, ptr);
        self.shader_surface_reigsters = fix_offset(self.shader_surface_reigsters, ptr);

        println!("Pre");
        try_fix(&mut self.platform_data, ptr);
        println!("Post");

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
    pub flip_palette: *mut *mut CFTexInst,
    pub scroll_st_per_second: CFVec2,
    pub uv_degree_rotation_per_second: f32,
    pub compressed_uv_rot_anchor: [u8; 2],
    _padding: [u8; 2],
}

impl Fixable for FMeshTexLayerID {
    unsafe fn fix_offset(&mut self, ptr: *mut u8) {
        self.flip_palette = fix_offset(self.flip_palette, ptr);

        if let Some(array) = array_ptr_mut(self.flip_palette, self.flip_page_count) {
            array.iter_mut().for_each(|value| try_fix(value, ptr))
        }
    }
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct CFTexInst {
    /// Pointer to TexDef to use
    pub tex_def: *mut FTexDef,
    /// Double buffer texture data for RenderTargets
    pub tex_buffer: [*mut FTexData; 2],
    pub buffer_index: u8,
    /// See FTEX_INSTFLAG_* for info
    pub flags: u32,
    /// 0.0f=normal, -1=bias by one smaller level, +1=bias by one larger level, etc.
    pub mipmap_bias: f32,
}

impl Fixable for CFTexInst {
    unsafe fn fix_offset(&mut self, ptr: *mut u8) {
        try_fix(&mut self.tex_def, ptr);

        self.tex_buffer
            .iter_mut()
            .for_each(|value| try_fix(value, ptr));
    }
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct FTexDef {
    pub tex_info: FTexInfo,
    pub tex_data: *mut FTexData,
}

impl Fixable for FTexDef {
    unsafe fn fix_offset(&mut self, ptr: *mut u8) {
        self.tex_info.fix_offset(ptr);
        try_fix(&mut self.tex_data, ptr);
    }
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct FTexInfo {
    /// ASCIIZ name of texture
    #[sb(skip)]
    pub name: FixedString<FDATA_TEXNAME_LENGTH>,
    /// Pointer to user-defined data
    pub user_data: *mut (),
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

impl Fixable for FTexInfo {
    unsafe fn fix_offset(&mut self, ptr: *mut u8) {
        self.user_data = fix_offset(self.user_data, ptr);
    }
}

#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct FLink {
    pub prev_link: *mut FLink,
    pub next_link: *mut FLink,
}

impl Fixable for FLink {
    unsafe fn fix_offset(&mut self, ptr: *mut u8) {
        self.prev_link = fix_offset(self.prev_link, ptr);
        self.next_link = fix_offset(self.next_link, ptr);
        // TODO: should I be fixing the values..?
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    #[repr(C)]
    pub struct FTexDataFlags: u8 {
        const NONE    = 0x00;
        // DXT texture
        const DXT = 0x01;
        // For streaming textures, this indicates that the texture is not in the cache
        const NOT_IN_CACHE = 0x02;
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
    // D3D texel format used for this texture
    pub d3d_fmt_color: u32,
    // D3D depth/stencil format (render targets only)
    pub d3d_fmt_depth: u32,
    // Set bits indicate which stages this texture is selected into (0=none)
    pub attached_stages: u32,
    // Approximate bytes consumed by this texture
    pub texture_bytes: u32,
    pub streaming_handle: *mut (),
    // Pointer to the image data for the D3D texture if load-in-place
    pub image_data: *mut (),
    // Pointer to D3D texture object IDirect3DTexture8
    pub d3d_texture: *mut (),
    // Pointer to D3D depth-stencil surface (NULL=none) IDirect3DSurface8
    pub d3d_depth_stencil: *mut (),
}

impl Fixable for FTexData {
    unsafe fn fix_offset(&mut self, ptr: *mut u8) {
        self.tex_def.fix_offset(ptr);
        self.link.fix_offset(ptr);

        self.streaming_handle = fix_offset(self.streaming_handle, ptr);
        self.image_data = fix_offset(self.image_data, ptr);
        self.d3d_texture = fix_offset(self.d3d_texture, ptr);
        self.d3d_depth_stencil = fix_offset(self.d3d_depth_stencil, ptr);
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
