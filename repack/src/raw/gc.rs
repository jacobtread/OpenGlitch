//! Raw game cube types

use bitflags::bitflags;
use std::{mem::size_of, ptr::null_mut};
use swapbytes::SwapBytes;

use crate::st::{array_ptr, array_ptr_mut, fix_offset, try_fix, try_fix_array, CFSphere, Fixable};

/// GameCube RGBA color
#[derive(Debug, Clone, Copy, SwapBytes)]
pub struct GxColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

/// Game cube compute type, for our usage this
/// defines the position type of a vertex buffer
#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(u8)]
pub enum GxCompType {
    U8 = 0,
    S8 = 1,
    U16 = 2,
    S16 = 3,
    F32 = 4,
}

/// GameCube attr types, used for its position index types
#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(u8)]
pub enum GxAttrType {
    None = 0,
    Direct = 1,
    Index8 = 2,
    Index16 = 3,
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    #[repr(C)]
    pub struct GxVertexBufferFlags: u16 {
        const NONE     = 0x00;
        /// If skinned, the position and normal are presumed 48-bits each
        const SKINNED  = 0x01; // position and normal composed of s16's

        /// We assume fixed-point 16-bit normal, unless this flag is set:
        const NORM_NBT = 0x10;	// normal has binormal and tangent for bump-mapping
    }
}

impl SwapBytes for GxVertexBufferFlags {
    fn swap_bytes_mut(&mut self) {
        let value = self.bits().swap_bytes();
        *self = Self::from_bits_retain(value);
    }
}

static POS_FRACTIONAL_ADJUSTMENT: [f32; 16] = [
    1.,
    (1. / 2.),
    (1. / 4.),  //0.25f;
    (1. / 8.),  //0.125f;
    (1. / 16.), //0.0625f,
    (1. / 32.), //0.03125f,
    (1. / 64.),
    (1. / 128.),
    (1. / 256.),
    (1. / 512.),
    (1. / 1024.),
    (1. / 2048.),
    (1. / 4096.),
    (1. / 8192.),
    (1. / 16384.),
    (1. / 32768.),
];

/// UV data
///
/// 8 bit UV's do not have enough resolution.  16 bit seems to be fine
#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct GxSt {
    /// X dimension
    pub s: i16,
    /// Y Dimension
    pub t: i16,
}

/// Byte order needs to be fixable for these types
impl Fixable for GxSt {}

// Normal structure used for dynamic bump-mapping
pub struct GxNBT {
    /// Normal
    pub n: [i8; 3],
    /// Binormal
    pub b: [i8; 3],
    /// Tangents
    pub t: [i8; 4],
}

/// Game cube "vertex buffer" format
#[derive(Debug, SwapBytes)]
#[repr(C)]
pub struct GxVertexBuffer {
    /// Flags for this vertex buffer
    pub flags: GxVertexBufferFlags,
    /// Number of positions in this vertex buffer
    pub position_count: u16,
    /// Type of positions stored in this buffer
    position_type: GxCompType,
    /// Index type for positions
    position_index_type: GxAttrType,
    /// Byte size of each element in the position vector
    position_stride: u8,
    /// Number of bits in the fractional component of position
    position_fraction_bits: u8,

    /// Number of diffuse colors in this vertex buffer
    diffuse_count: u16,
    /// GameCube color index type
    color_index_type: GxAttrType,

    gc_vertex_format: u8,

    /// Pointer to the actual buffer data, shape is determined by `position_type`
    /// and the SKINNED flag
    position_buffer: *mut u8,
    /// Pointer to the array of difuse colors (if any) of length `diffuse_count`
    diffuse: *mut GxColor,
    /// Pointer to the ST (UV) data
    st: *mut GxSt,
    /// For bumpmapped objects, Pointer to the normal, binormal and tangents
    nbt: *mut GxNBT,
}

impl GxVertexBuffer {
    pub fn normalized(&self) -> Vec<GxPosF32> {
        let positions = self
            .positions()
            .expect("Vertex buffer positions are invalid");
        let adjust = *POS_FRACTIONAL_ADJUSTMENT
            .get(self.position_fraction_bits as usize)
            .expect("Missing fractional adjustment for vertex buffer");
        let mut out = Vec::with_capacity(self.position_count as usize);

        match positions {
            GxVertexBufferPosition::S8(values) => {
                values.iter().for_each(|value| {
                    out.push(GxPosF32 {
                        x: value.x as f32,
                        y: value.y as f32,
                        z: value.z as f32,
                    });
                });
            }
            GxVertexBufferPosition::S16(values) => {
                values.iter().for_each(|value| {
                    out.push(GxPosF32 {
                        x: value.x as f32,
                        y: value.y as f32,
                        z: value.z as f32,
                    });
                });
            }
            GxVertexBufferPosition::SkinnedS16(values) => {
                values.iter().for_each(|value| {
                    out.push(GxPosF32 {
                        x: value.position.x as f32,
                        y: value.position.y as f32,
                        z: value.position.z as f32,
                    });
                });
            }
            GxVertexBufferPosition::F32(values) => {
                values.iter().for_each(|value| {
                    out.push(*value);
                });
            }
        }

        out
    }

    pub fn get_position(&self, index: usize) -> Option<GxPosF32> {
        let positions = self
            .positions()
            .expect("Vertex buffer positions are invalid");
        let adjust = *POS_FRACTIONAL_ADJUSTMENT
            .get(self.position_fraction_bits as usize)
            .expect("Missing fractional adjustment for vertex buffer");

        // Multiply non float values by adjust

        match positions {
            GxVertexBufferPosition::S8(value) => {
                let value = value.get(index)?;

                Some(GxPosF32 {
                    x: value.x as f32 * adjust,
                    y: value.y as f32 * adjust,
                    z: value.z as f32 * adjust,
                })
            }
            GxVertexBufferPosition::S16(value) => {
                let value = value.get(index)?;

                Some(GxPosF32 {
                    x: value.x as f32 * adjust,
                    y: value.y as f32 * adjust,
                    z: value.z as f32 * adjust,
                })
            }
            GxVertexBufferPosition::SkinnedS16(value) => {
                let value = value.get(index)?;

                Some(GxPosF32 {
                    x: value.position.x as f32 * adjust,
                    y: value.position.y as f32 * adjust,
                    z: value.position.z as f32 * adjust,
                })
            }
            GxVertexBufferPosition::F32(value) => value.get(index).copied(),
        }
    }

    /// Access the positions array using the provided type
    fn positions_typed<T>(&self) -> Option<&[T]>
    where
        T: 'static,
    {
        unsafe { array_ptr(self.position_buffer.cast::<T>(), self.position_count) }
    }

    /// Access the positions array using the provided type
    fn positions_typed_mut<T>(&mut self) -> Option<&mut [T]>
    where
        T: 'static,
    {
        unsafe { array_ptr_mut(self.position_buffer.cast::<T>(), self.position_count) }
    }

    pub fn positions(&self) -> Option<GxVertexBufferPosition<'_>> {
        Some(match self.position_type {
            GxCompType::S8 => {
                let array: &[GxPosS8] = self.positions_typed()?;
                GxVertexBufferPosition::S8(array)
            }
            GxCompType::S16 => {
                if self.flags.contains(GxVertexBufferFlags::SKINNED) {
                    let array: &[GxSkinPosNorm] = self.positions_typed()?;
                    GxVertexBufferPosition::SkinnedS16(array)
                } else {
                    let array: &[GxPosS16] = self.positions_typed()?;
                    GxVertexBufferPosition::S16(array)
                }
            }

            GxCompType::F32 => {
                let array: &[GxPosF32] = self.positions_typed()?;
                GxVertexBufferPosition::F32(array)
            }

            _ => panic!("Unsupported vertex position data type"),
        })
    }

    pub fn positions_mut(&mut self) -> Option<GxVertexBufferPositionMut<'_>> {
        Some(match self.position_type {
            GxCompType::S8 => {
                let array: &mut [GxPosS8] = self.positions_typed_mut()?;
                GxVertexBufferPositionMut::S8(array)
            }
            GxCompType::S16 => {
                if self.flags.contains(GxVertexBufferFlags::SKINNED) {
                    let array: &mut [GxSkinPosNorm] = self.positions_typed_mut()?;
                    GxVertexBufferPositionMut::SkinnedS16(array)
                } else {
                    let array: &mut [GxPosS16] = self.positions_typed_mut()?;
                    GxVertexBufferPositionMut::S16(array)
                }
            }

            GxCompType::F32 => {
                let array: &mut [GxPosF32] = self.positions_typed_mut()?;
                GxVertexBufferPositionMut::F32(array)
            }

            _ => panic!("Unsupported vertex position data type"),
        })
    }

    /// Fixes the byte order for the position data stored within this buffer
    fn fix_position_data(&mut self) {
        if self.position_count == 0 {
            return;
        }

        // Get access to the position data
        if let Some(positions) = self.positions_mut() {
            // Swap the endianess of the position data
            match positions {
                GxVertexBufferPositionMut::S8(_values) => { /* Theres no need to swap i8's */ }
                GxVertexBufferPositionMut::S16(values) => values.swap_bytes_mut(),
                GxVertexBufferPositionMut::SkinnedS16(values) => values.swap_bytes_mut(),
                GxVertexBufferPositionMut::F32(values) => values.swap_bytes_mut(),
            }
        }
    }
}

impl Fixable for GxVertexBuffer {
    unsafe fn fix_offset(&mut self, ptr: *mut u8) {
        self.position_buffer = fix_offset(self.position_buffer, ptr);
        self.fix_position_data();

        // Value type for diffuse doesn't need to be fixed so just fix the pointer itself
        self.diffuse = fix_offset(self.diffuse, ptr);

        try_fix(&mut self.st, ptr);

        // Value type for nbt doesn't need to be fixed so just fix the pointer itself
        self.nbt = fix_offset(self.nbt, ptr);
    }
}

/// Position represented by 3 signed 8bit ints
#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct GxPosS8 {
    pub x: i8,
    pub y: i8,
    pub z: i8,
}

/// Position represented by 3 signed 16bit ints
#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct GxPosS16 {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

/// Position represented by 3 signed 16bit ints
/// and normal represented by 3 signed 16 bit ints
#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct GxSkinPosNorm {
    pub position: GxPosS16,
    pub normal: GxPosS16,
}

impl Fixable for GxSkinPosNorm {}

/// Position represented by 3 signed 32bit floats
#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(C)]
pub struct GxPosF32 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Variant specific vertex buffer positions
#[derive(Debug)]
pub enum GxVertexBufferPosition<'a> {
    S8(&'a [GxPosS8]),
    S16(&'a [GxPosS16]),
    SkinnedS16(&'a [GxSkinPosNorm]),
    F32(&'a [GxPosF32]),
}

/// Variant specific vertex buffer positions
#[derive(Debug)]
pub enum GxVertexBufferPositionMut<'a> {
    S8(&'a mut [GxPosS8]),
    S16(&'a mut [GxPosS16]),
    SkinnedS16(&'a mut [GxSkinPosNorm]),
    F32(&'a mut [GxPosF32]),
}

#[derive(Debug)]
pub enum GxVertexBufferIndex<'a> {
    Index8(&'a [u8]),
    Index16(&'a [u16]),
}
#[derive(Debug)]
pub enum GxVertexBufferIndexMut<'a> {
    Index8(&'a mut [u8]),
    Index16(&'a mut [u16]),
}

#[derive(Debug, SwapBytes)]
#[repr(C)]
pub struct GxMeshSkin {
    /// Number of skin translations
    translations_count: u16,
    /// Number of vert weights
    pub vert_matrix_counts: GxVertMatrixCounts,
    /// Skin translations
    translations: *mut GxTranslationDescription,
    /// Number of skinned vertices
    skinned_verts_count: u32,
    /// Pointer to the array of skinned vertices
    skinned_verts: *mut GxSkinPosNorm,
    /// Pointer to the array of weights (one to one correspondence with position)
    skin_weights: *mut GxSkinWeights,
}

impl Fixable for GxMeshSkin {
    unsafe fn fix_offset(&mut self, ptr: *mut u8) {
        try_fix_array(&mut self.translations, self.translations_count, ptr);
        try_fix_array(
            &mut self.skinned_verts,
            self.skinned_verts_count as usize,
            ptr,
        );

        self.skin_weights = fix_offset(self.skin_weights, ptr)
    }
}

impl GxMeshSkin {
    pub fn translations(&self) -> Option<&[GxTranslationDescription]> {
        unsafe { array_ptr(self.translations, self.translations_count) }
    }

    pub fn skinned_verts(&self) -> Option<&[GxSkinPosNorm]> {
        unsafe { array_ptr(self.skinned_verts, self.skinned_verts_count as usize) }
    }

    pub fn skin_weights(&self) -> Option<&[GxSkinWeights]> {
        unsafe { array_ptr(self.skin_weights, self.translations_count) }
    }
}

/// Counters for the number of verts weighted to the
/// different number of matrices
#[derive(Debug, SwapBytes)]
#[repr(C)]
pub struct GxVertMatrixCounts {
    // Number of verts weighted to 1 matrix
    pub td1: u16,
    // Number of verts weighted to 2 matrices
    pub td2: u16,
    // Number of verts weighted to 3 or 4 matrices
    pub td3_or_4: u16,
}

/// Structure that describes transforms for skin
#[derive(Debug, SwapBytes)]
#[repr(C)]
pub struct GxTranslationDescription {
    /// Number of matrix ids from `matrix_id`
    matrix_count: u8,
    /// Padding field
    _pad: u8,
    /// The number of vertices to affect
    pub vertex_count: u16,
    /// Ids of the matrix, only valid portion is `matrix_count` worth
    matrix_ids: [u8; 4],
}

impl Fixable for GxTranslationDescription {}

impl GxTranslationDescription {
    pub fn matrix_ids(&self) -> &[u8] {
        // Length clamped incase of invalid `matrix_count` being provided
        let length = self.matrix_count.min(4) as usize;
        &self.matrix_ids[..length]
    }
}

/// Skin weights
#[derive(Debug, SwapBytes)]
#[repr(C)]
pub struct GxSkinWeights {
    pub weights: [u8; 4],
}

/// GameCube mesh structure
#[derive(Debug, SwapBytes)]
#[repr(C)]
pub struct GxMesh {
    /// Set at runtime to a pointer of the base object (null and unused for this impl)
    _mesh: *mut (),
    /// Used only when nSegCount is 0
    pub at_rest_bound_sphere: CFSphere,
    /// Bitset, but options don't seem to be defined for this?
    pub flags: u8,
    /// Number of vertex buffers used by this mesh
    vertex_buffers_count: u8,
    /// Number of materials in this node
    pub material_count: u16,
    /// Pointer to array of vertex buffers
    vertex_buffers: *mut GxVertexBuffer,
    /// Pointer to the mesh skin, if there is one
    mesh_skin: *mut GxMeshSkin,
}

impl Fixable for GxMesh {
    unsafe fn fix_offset(&mut self, ptr: *mut u8) {
        self._mesh = null_mut();

        try_fix_array(&mut self.vertex_buffers, self.vertex_buffers_count, ptr);
        try_fix(&mut self.mesh_skin, ptr);
    }
}

impl GxMesh {
    pub fn vertex_buffers(&self) -> Option<&[GxVertexBuffer]> {
        unsafe { array_ptr(self.vertex_buffers, self.vertex_buffers_count) }
    }

    pub fn mesh_skin(&self) -> Option<&GxMeshSkin> {
        unsafe { self.mesh_skin.as_ref() }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    #[repr(C)]
    pub struct GxDisplayListContainerFlags: u8 {
        const NONE    = 0x00;
        const SKINNED = 0x01;
        const CONSTANT_COLOR = 0x02;
        const BUMPMAP = 0x04;
        const FACING_OPP_DIR_LIGHT =0x08;
        const STREAMING = 0x80;
    }
}

/// GameCube display list container
///
/// Not sure if I need this impl?
#[derive(Debug, SwapBytes)]
#[repr(C)]
pub struct GxDisplayListContainer {
    /// Flags for the display container
    #[sb(skip)]
    pub flags: GxDisplayListContainerFlags,
    /// Matrix index for this display list
    pub matrix_index: u8,
    /// ID for the LOD this display list is part of (0 is closest)
    pub lod_id: u8,

    pub part_id: u8,
    /// Number of stripped triangles in this display list
    pub strip_tri_count: u16,
    /// Number of list triangles in this display list
    pub list_tri_count: u16,
    /// Number of tri strips
    pub strip_count: u16,
    /// Number of tri lists
    pub list_count: u8,
    /// Index into the mesh's vertex buffers indicating which is used
    pub vb_index: u8,

    /// size of buffer
    size: u32,
    /// Buffer of data for the display list
    buffer: *mut (),

    pub constant_color: GxColor,
}

impl GxDisplayListContainer {}

impl Fixable for GxDisplayListContainer {
    unsafe fn fix_offset(&mut self, ptr: *mut u8) {
        if self.flags.contains(GxDisplayListContainerFlags::STREAMING) {
            // TODO: handle streaming.. if we actually need this structure?
            self.buffer = null_mut();
        } else {
            self.buffer = fix_offset(self.buffer, ptr);
        }
    }
}

/// GameCube platform specific mesh material data
///
/// Not sure if I need this impl?
#[derive(Debug, SwapBytes)]
#[repr(C)]
pub struct GxMeshMaterial {
    /// Pointer to array of display list containers for this material
    dl_containers: *mut GxDisplayListContainer,
    /// Number of display list containers used
    dl_containers_count: u16,
}

impl Fixable for GxMeshMaterial {
    unsafe fn fix_offset(&mut self, ptr: *mut u8) {
        try_fix_array(&mut self.dl_containers, self.dl_containers_count, ptr)
    }
}

impl GxMeshMaterial {
    pub fn dl_containers(&self) -> Option<&[GxDisplayListContainer]> {
        unsafe { array_ptr(self.dl_containers, self.dl_containers_count) }
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

#[derive(Debug, SwapBytes)]
#[repr(C)]
pub struct GxTexObj {
    // TODO: ...what actually goes here?
    _dummy: [u32; 8],
}

impl Fixable for GxTexObj {}
