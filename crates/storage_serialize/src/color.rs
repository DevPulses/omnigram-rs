//! `writeColor` / `readColor` — the packed storage form of a colour.
//!
//! Source of truth: `storage/serialize_common.cpp`.
//!
//! One `quint32`: red in the lowest byte, then green, blue, alpha. That is not
//! `ARGB32` and not Qt's `rgba()` order, so it is spelled out explicitly.

/// One stored colour.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Color {
    #[must_use]
    pub const fn new(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }

    /// `writeColor` — the packed value.
    #[must_use]
    pub const fn packed(self) -> u32 {
        (self.red as u32)
            | ((self.green as u32) << 8)
            | ((self.blue as u32) << 16)
            | ((self.alpha as u32) << 24)
    }

    /// `readColor` — the colour behind a packed value.
    #[must_use]
    pub const fn from_packed(value: u32) -> Self {
        Self {
            red: (value & 0xFF) as u8,
            green: ((value >> 8) & 0xFF) as u8,
            blue: ((value >> 16) & 0xFF) as u8,
            alpha: ((value >> 24) & 0xFF) as u8,
        }
    }
}
