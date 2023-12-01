use swapbytes::SwapBytes;

use crate::st::{
    array_ptr, array_ptr_mut, fix_offset, try_fix, try_fix_array, CFSphere, CFVec3, Fixable,
};

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
    // Pointer to an array of index buffers (arrays of u16s)
    index_buffer: ArrayPtr<ArrayPtr<u16>>,
}

/// Type alias that shows a pointer is an array of values rather than
/// just a normal pointer to a single value
type ArrayPtr<T> = *mut T;

impl Fixable for DxMesh {
    unsafe fn fix_offset(&mut self, ptr: *mut u8) {
        try_fix_array(&mut self.vertex_buffers, self.vertex_buffer_count, ptr);
        // todo: fix col vertex buffer

        try_fix_array(&mut self.indicies_counts, self.index_buffer_count, ptr);

        self.index_buffer = fix_offset(self.index_buffer, ptr);

        if !self.index_buffer.is_null() {
            for i in 0..self.index_buffer_count {
                println!("At offset {}", i);
                let length = self.index_count(i as usize);
                let buffer = &mut *self.index_buffer.add(i as usize);

                try_fix_array(buffer, length, ptr);
            }
        }
    }
}
impl Fixable for u16 {}

impl DxMesh {
    pub fn vertex_buffers(&self) -> Option<&[DxVertexBufferDescriptor]> {
        unsafe { array_ptr(self.vertex_buffers, self.vertex_buffer_count) }
    }
    pub fn vertex_buffers_mut(&mut self) -> Option<&mut [DxVertexBufferDescriptor]> {
        unsafe { array_ptr_mut(self.vertex_buffers, self.vertex_buffer_count) }
    }

    /// Gets the number of indexes in the index buffer at the provided index
    pub fn index_count(&self, index: usize) -> u16 {
        unsafe { *self.indicies_counts.add(index) }
    }

    pub fn index_buffers(&self) -> Vec<&[u16]> {
        let mut out = Vec::with_capacity(self.index_buffer_count as usize);

        for i in 0..self.index_buffer_count {
            if let Some(buffer) = self.index_buffer(i as usize) {
                out.push(buffer);
            }
        }

        out
    }

    pub fn index_buffer(&self, index: usize) -> Option<&[u16]> {
        if self.index_buffer.is_null() {
            return None;
        }

        let length = self.index_count(index);
        let ptr = unsafe { *self.index_buffer.add(index) };

        unsafe { array_ptr(ptr, length) }
    }

    pub fn index_buffer_mut(&self, index: usize) -> Option<&mut [u16]> {
        if self.index_buffer.is_null() {
            return None;
        }

        let length = self.index_count(index);
        let ptr = unsafe { *self.index_buffer.add(index) };

        unsafe { array_ptr_mut(ptr, length) }
    }
}

/// GameCube attr types, used for its position index types
#[derive(Debug, Clone, Copy, SwapBytes)]
#[repr(i8)]
pub enum DxVertexBufferType {
    Shader = -1,

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
    info_index: DxVertexBufferType,
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
    vertex_buffer: *mut (), /* IDirect3DVertexBuffer8 */
}

impl DxVertexBufferDescriptor {
    pub fn positions(&mut self) -> Vec<[f32; 3]> {
        let mut out = Vec::new();

        match self.buffer_values().unwrap() {
            DxVertexBufferValues::N1C1T1(value) => {
                out.extend(value.iter().map(|value| value.position.clone()))
            }
            DxVertexBufferValues::N1C1T2(value) => {
                out.extend(value.iter().map(|value| value.position.clone()))
            }
            DxVertexBufferValues::N1W3C1T1(value) => {
                out.extend(value.iter().map(|value| value.position.clone()))
            }
            DxVertexBufferValues::N1W3C1T2(value) => {
                out.extend(value.iter().map(|value| value.position.clone()))
            }
            DxVertexBufferValues::TLC2T2(value) => {
                out.extend(value.iter().map(|value| value.position.clone()))
            }
            DxVertexBufferValues::C1(_) => todo!(),
            DxVertexBufferValues::C1T1(_) => todo!(),
        }

        out
    }

    pub fn buffer_values(&mut self) -> Option<DxVertexBufferValues> {
        match self.info_index {
            DxVertexBufferType::Shader => None,
            DxVertexBufferType::N1C1T1 => {
                let values = unsafe {
                    array_ptr_mut(self.vertex_buffer.cast(), self.vertex_count as usize)
                }?;
                Some(DxVertexBufferValues::N1C1T1(values))
            }
            DxVertexBufferType::N1C1T2 => {
                let values = unsafe {
                    array_ptr_mut(self.vertex_buffer.cast(), self.vertex_count as usize)
                }?;
                Some(DxVertexBufferValues::N1C1T2(values))
            }
            DxVertexBufferType::N1W3C1T1 => {
                let values = unsafe {
                    array_ptr_mut(self.vertex_buffer.cast(), self.vertex_count as usize)
                }?;
                Some(DxVertexBufferValues::N1W3C1T1(values))
            }
            DxVertexBufferType::N1W3C1T2 => {
                let values = unsafe {
                    array_ptr_mut(self.vertex_buffer.cast(), self.vertex_count as usize)
                }?;
                Some(DxVertexBufferValues::N1W3C1T2(values))
            }
            DxVertexBufferType::TLC2T2 => {
                let values = unsafe {
                    array_ptr_mut(self.vertex_buffer.cast(), self.vertex_count as usize)
                }?;
                Some(DxVertexBufferValues::TLC2T2(values))
            }
            DxVertexBufferType::C1 => {
                let values = unsafe {
                    array_ptr_mut(self.vertex_buffer.cast(), self.vertex_count as usize)
                }?;
                Some(DxVertexBufferValues::C1(values))
            }
            DxVertexBufferType::C1T1 => {
                let values = unsafe {
                    array_ptr_mut(self.vertex_buffer.cast(), self.vertex_count as usize)
                }?;
                Some(DxVertexBufferValues::C1T1(values))
            }
        }
    }
}

impl Fixable for DxVertexBufferDescriptor {
    unsafe fn fix_offset(&mut self, ptr: *mut u8) {
        self.lmuv_stream = fix_offset(self.lmuv_stream, ptr);
        self.basis_stream = fix_offset(self.basis_stream, ptr);
        self.lock_buf = fix_offset(self.lock_buf, ptr);
        self.vertex_buffer = fix_offset(self.vertex_buffer, ptr);
        self._link.fix_offset(ptr);
    }
}

#[derive(Debug, SwapBytes)]
#[repr(C)]
pub struct DxMeshMaterial {
    cluster: ArrayPtr<DxMeshCluster>,
    cluster_count: u32,
}

impl Fixable for DxMeshMaterial {
    unsafe fn fix_offset(&mut self, ptr: *mut u8) {
        try_fix_array(&mut self.cluster, self.cluster_count as usize, ptr);
    }
}

#[derive(Debug, SwapBytes)]
#[repr(C)]
pub struct DxMeshCluster {
    strip_count: u16,
    flags: u8,
    segment_index: u8,
    pub vertex_buffer_index: u8,
    pub index_buffer_index: u8,
    part_id: u8,
    lod_id: u8,

    push_buffer: *mut (),
    pub tri_list: DxMeshTriList,
    mesh_strip: ArrayPtr<DxMeshStrip>,
}

impl Fixable for DxMeshCluster {
    unsafe fn fix_offset(&mut self, ptr: *mut u8) {
        self.push_buffer = fix_offset(self.push_buffer, ptr);
        try_fix_array(&mut self.mesh_strip, self.strip_count, ptr);
    }
}

impl DxMeshCluster {
    pub fn mesh_strips(&self) -> Option<&[DxMeshStrip]> {
        unsafe { array_ptr(self.mesh_strip, self.strip_count) }
    }
}

#[derive(Debug, SwapBytes)]
#[repr(C)]
pub struct DxMeshTriList {
    // Number of single triangles
    pub tri_count: u16,
    // Starting vindex into DX Index Buffer
    pub start_vindex: u16,
    // Minimum vertex index referenced by this triangle list
    pub vtx_index_min: u16,
    // (Maximum vertex index referenced by this triangle list + 1) - nVtxIndexMin
    pub vtx_index_range: u16,
}

#[derive(Debug, SwapBytes)]
#[repr(C)]
pub struct DxMeshStrip {
    // Number of triangles in this strip
    pub tri_count: u8,
    // Alignment padding
    _pad: u8,
    // Starting vindex into DX Index Buffer
    pub start_vindex: u16,
    // Minimum vertex index referenced by this strip
    pub ctx_index_min: u16,
    // (Maximum vertex index referenced by this strip + 1) - nVtxIndexMin
    pub vtx_index_range: u16,
}

impl Fixable for DxMeshStrip {}

pub struct DxVertexBufferInfo {
    // D3D Flexible Vertex Format code (0=end of table)
    fvf: u32,
    // Number of bytes per vertex
    vtx_bytes: u8,
    // TRUE: vertex contains already transformed and lit data
    post_tl: bool,
    // Number of f32,f32,f32 normal fields
    normal_count: u8,
    // Number of f32 weights
    weight_count: u8,
    // Number of D3DCOLOR color fields
    color_count: u8,
    // Number of f32,f32 (S,T) texture coordinate pairs
    tc_count: u8,
    // Offset from beginning of vertex structure of the 3D coordinate
    offset_pos: u8,
    // Offset from beginning of vertex structure of the weight array
    offset_weight: u8,
    // Offset from beginning of vertex structure of the normal
    offset_normal: u8,
    // Offset from beginning of vertex structure of the color arrays
    offset_color: u8,
    // Offset from beginning of vertex structure of the texture coordinate pair array
    offset_tc: u8,
}
