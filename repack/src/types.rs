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
