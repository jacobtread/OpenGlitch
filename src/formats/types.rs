use std::{
    ffi::CStr,
    fmt::{Debug, Display},
    io::{Read, Seek, SeekFrom},
    ops::{Deref, DerefMut},
};

use binrw::{file_ptr::IntoSeekFrom, BinRead, BinResult, Endian};

// Offset within the file that something can be found at
#[derive(Debug, BinRead, Default)]
pub struct PtrOffset(u32);

// CFMtx43
#[derive(Debug, BinRead, Default)]
pub struct RawMatrix4x3f {
    pub matrix: [[f32; 3]; 4],
}

// CFMtx44
#[derive(Debug, BinRead, Default)]
pub struct RawMatrix4x4f {
    pub matrix: [[f32; 4]; 4],
}

#[derive(Debug, BinRead, Default)]
pub struct RawVec3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Debug, BinRead, Default)]
pub struct RawVec2f {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, BinRead, Default)]
pub struct RawSphere {
    pub radius: f32,
    pub position: RawVec3f,
}

#[derive(Debug, BinRead, Default)]
pub struct RawColorRGBA {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

#[derive(Debug, BinRead, Default)]
pub struct RawColorRGB {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

#[derive(Debug, BinRead, Default)]
pub struct RawColorMotif {
    pub color: RawColorRGBA,
    pub modif_index: u32,
}

/// Null terminated string created from a fixed length
/// chunk of bytes
#[derive(BinRead, Clone, Copy)]
#[repr(C)]
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

#[derive(Debug)]
pub struct NullableFilePtr<T> {
    /// The file pointer
    pub ptr: u32,
    /// The value won't be read if the pointer is null
    pub value: Option<T>,
}

impl<T> BinRead for NullableFilePtr<T>
where
    T: BinRead,
{
    type Args<'a> = T::Args<'a>;

    fn read_options<R: Read + Seek>(
        reader: &mut R,
        endian: Endian,
        args: Self::Args<'_>,
    ) -> BinResult<Self> {
        let ptr = u32::read_options(reader, endian, ())?;

        let value = if ptr == 0 {
            None
        } else {
            let before = reader.stream_position()?;
            reader.seek(SeekFrom::Start(ptr as u64))?;
            let value = T::read_options(reader, endian, args);
            reader.seek(SeekFrom::Start(before))?;

            Some(value?)
        };

        Ok(NullableFilePtr { ptr, value })
    }
}

impl<T> Deref for NullableFilePtr<T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
