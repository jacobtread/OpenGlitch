use std::{
    ffi::CStr,
    fmt::{Debug, Display},
};

use binrw::{BinRead, BinResult};
use nalgebra::{ArrayStorage, Matrix4x3};

// CFMtx43
#[derive(Debug, BinRead, Default)]
#[repr(C)]
#[br(big)]
pub struct RawMatrix4x3f {
    pub matrix: [[f32; 3]; 4],
}

// CFMtx44
#[derive(Debug, BinRead, Default)]
#[repr(C)]
#[br(big)]
pub struct RawMatrix4x4f {
    pub matrix: [[f32; 4]; 4],
}

#[derive(Debug, BinRead, Default)]
#[repr(C)]
#[br(big)]
pub struct ApeVec3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, BinRead, Default)]
#[repr(C)]
#[br(big)]
pub struct ApeSphere {
    pub radius: f32,
    pub position: ApeVec3f,
}

/// Null terminated string created from a fixed length
/// chunk of bytes
#[derive(BinRead)]
#[repr(C)]
#[br(big)]
pub struct FixedString<const LENGTH: usize> {
    bytes: [u8; LENGTH],
}

impl<const LENGTH: usize> Default for FixedString<LENGTH> {
    fn default() -> Self {
        Self {
            bytes: [0u8; LENGTH],
        }
    }
}

impl<const LENGTH: usize> FixedString<LENGTH> {
    pub fn as_cstr(&self) -> &CStr {
        CStr::from_bytes_until_nul(&self.bytes).expect("Fixed string missing null byte")
    }

    pub fn as_string(&self) -> String {
        self.as_cstr().to_string_lossy().to_string()
    }
}

impl<const LENGTH: usize> Debug for FixedString<LENGTH> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.as_cstr();
        Debug::fmt(value, f)
    }
}

impl<const LENGTH: usize> Display for FixedString<LENGTH> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.as_string();
        Display::fmt(&value, f)
    }
}
