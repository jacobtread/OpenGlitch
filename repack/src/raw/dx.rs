use swapbytes::SwapBytes;

use crate::st::{fix_offset, try_fix, try_fix_array, CFSphere, CFVec3, Fixable};

/// Directx8 mesh definition
#[derive(Debug, SwapBytes)]
#[repr(C)]
pub struct DxMesh {
    /// See FDX8MESH_FLAG_* for info
    pub flags: u16,
    /// Number of vertex buffers used by this mesh
    vertex_buffer_count: u8,
    /// Number of index buffers used by this mesh
    index_buffer_count: u8,
    /// The address offset for the temporary portion of the file when loaded (this portion is converted to DX resources).
    pub disposable_offset: u32,
    /// Used only when nSegCount is 0
    pub at_rest_bound_sphere: CFSphere,
    /// Set at runtime to a pointer of the base object (null and unused for this impl)
    _mesh: *mut (),
    /// Array of vertex buffer descriptors
    vertex_buffers: *mut DxVertexBufferDescriptor,
    /// Array of Collision vertex buffers
    coll_vertex_buffer: *mut *mut CFVec3,
    /// Array for number of indices used by this mesh in each IB
    indicies_counts: *mut u16,
    // Pointer to an array of index buffers (array of u16s)
    index_buffer: *mut *mut (),
}

impl Fixable for DxMesh {
    unsafe fn fix_offset(&mut self, ptr: *mut u8) {
        try_fix_array(&mut self.vertex_buffers, self.vertex_buffer_count, ptr);
        // todo: fix col vertex buffer

        try_fix_array(&mut self.indicies_counts, self.index_buffer_count, ptr)
    }
}
impl Fixable for u16 {}

/// GameCube attr types, used for its position index types
#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(u8)]
pub enum DxVertexBufferType {
    // 1 normal 1 color 1 TC
    N1C1T1 = 0,
    // 1 normal 1 color 2 TCs
    N1C1T2 = 1,

    // 1 normal 3 weights 1 color 1 TC
    N1W3C1T1 = 2,
    // 1 normal 3 weights 1 color 2 TCs
    N1W3C1T2 = 3,

    // Post-transformed-and-lit vertex: 2 color, 2 TCs
    TLC2T2 = 4,

    // 1 color
    C1 = 5,
    // 1 color 1 TC
    C1T1 = 6,
}

pub enum DxVertexBufferValues<'a> {
    // 1 normal 1 color 1 TC
    N1C1T1(&'a mut [N1C1T1]),
    // 1 normal 1 color 2 TCs
    N1C1T2(&'a mut [N1C1T2]),
    // 1 normal 3 weights 1 color 1 TC
    N1W3C1T1(&'a mut [N1W3C1T1]),
    // 1 normal 3 weights 1 color 2 TCs
    N1W3C1T2(&'a mut [N1W3C1T2]),
    // Post-transformed-and-lit vertex: 2 color, 2 TCs
    TLC2T2(&'a mut [TLC2T2]),
    // 1 color
    C1(&'a mut [C1]),
    // 1 color 1 TC
    C1T1(&'a mut [C1T1]),
}

#[derive(Debug, SwapBytes)]
#[repr(C)]
pub struct N1C1T1 {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub diffuse_rgba: u32,
    pub st_0: [f32; 2],
}

#[derive(Debug, SwapBytes)]
#[repr(C)]
pub struct N1C1T2 {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub diffuse_rgba: u32,
    pub st_0: [f32; 2],
    pub st_1: [f32; 2],
}

#[derive(Debug, SwapBytes)]
#[repr(C)]
pub struct N1W3C1T1 {
    pub position: [f32; 3],
    pub weight: [f32; 3],
    pub normal: [f32; 3],
    pub diffuse_rgba: u32,
    pub st_0: [f32; 2],
}

#[derive(Debug, SwapBytes)]
#[repr(C)]
pub struct N1W3C1T2 {
    pub position: [f32; 3],
    pub weight: [f32; 3],
    pub normal: [f32; 3],
    pub diffuse_rgba: u32,
    pub st_0: [f32; 2],
    pub st_1: [f32; 2],
}

#[derive(Debug, SwapBytes)]
#[repr(C)]
pub struct TLC2T2 {
    pub position: [f32; 3],
    pub rhw: f32,
    pub diffuse_rgba: u32,
    pub specular_rgba: u32,
    pub st_0: [f32; 2],
    pub st_1: [f32; 2],
}

#[derive(Debug, SwapBytes)]
#[repr(C)]
pub struct C1 {
    pub color: [f32; 3],
}

#[derive(Debug, SwapBytes)]
#[repr(C)]
pub struct C1T1 {
    pub color: [f32; 3],
    pub diffuse_rgba: u32,
    pub st_0: [f32; 2],
}

#[derive(Debug, SwapBytes)]
#[repr(C)]
pub struct FLink {
    prev_link: *mut FLink,
    next_link: *mut FLink,
}

impl Fixable for FLink {
    unsafe fn fix_offset(&mut self, ptr: *mut u8) {
        try_fix(&mut self.prev_link, ptr);
        try_fix(&mut self.next_link, ptr);
    }
}

#[derive(Debug, SwapBytes)]
#[repr(C)]
pub struct DxVertexBufferDescriptor {
    // Link to other VBs
    _link: FLink,
    // Number of vertices in this DX vertex buffer
    vertex_count: u32,
    // Number of bytes per vertex
    bytes_per_vertex: u16,
    // Number of f32,f32 (S,T) texture coordinate pairs used for lightmaps, per vertex
    lmtc_count: u16,
    // Pointer to the stream of lightmap UV's
    lmuv_stream: *mut (),
    // Pointer to the stream of basis vectors.
    basis_stream: *mut (),
    // Index into FDX8VB_InfoTable[] of the entry that describes this VB format (-1=shader)
    info_index: i8,
    // TRUE=this VB is dynamic
    dynamic: bool,
    // TRUE=software vertex processing
    software_vp: u8,
    // TRUE=this VB is locked
    locked: u8,
    // Set when Lock() is called to the memory address that can be filled with data
    lock_buf: *mut (),
    // Used to restore lock state if we lose the device
    lock_offset: u32,
    // Used to restore lock state if we lose the device
    lock_bytes: u32,
    // Handle to the vertex shader this VB is currently attached to (or FVF code if nInfoIndex is not -1)
    vertex_shader: u32,
    // Pointer to the actual DX vertex buffers
    dvertex_buffer: *mut (), /* IDirect3DVertexBuffer8 */
}

impl DxVertexBufferDescriptor {
    pub fn buffer_values() {}
}

impl Fixable for DxVertexBufferDescriptor {
    unsafe fn fix_offset(&mut self, ptr: *mut u8) {
        self.lmuv_stream = fix_offset(self.lmuv_stream, ptr);
        self.basis_stream = fix_offset(self.basis_stream, ptr);
        self.lock_buf = fix_offset(self.lock_buf, ptr);
        self.dvertex_buffer = fix_offset(self.dvertex_buffer, ptr);
        self._link.fix_offset(ptr);
    }
}
