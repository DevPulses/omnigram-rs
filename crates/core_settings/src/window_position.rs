//! Window position and scale adjustments.
//!
//! Port of `tdesktop/Telegram/SourceFiles/core/core_settings.cpp:WindowPosition`.

use storage_serialize::{ByteArrayReader, ByteArrayWriter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WindowPosition {
    pub moncrc: i32,
    pub maximized: i32,
    pub scale: i32,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl WindowPosition {
    pub fn serialize(&self) -> Vec<u8> {
        let mut writer = ByteArrayWriter::with_capacity(7 * 4);
        writer.write_i32(self.x);
        writer.write_i32(self.y);
        writer.write_i32(self.w);
        writer.write_i32(self.h);
        writer.write_i32(self.moncrc);
        writer.write_i32(self.maximized);
        writer.write_i32(self.scale);
        writer.into_result()
    }

    pub fn deserialize(data: &[u8]) -> Option<Self> {
        let mut reader = ByteArrayReader::from_slice(data);
        let x = reader.read_i32()?;
        let y = reader.read_i32()?;
        let w = reader.read_i32()?;
        let h = reader.read_i32()?;
        let moncrc = reader.read_i32()?;
        let maximized = reader.read_i32()?;
        let scale = reader.read_i32()?;
        Some(Self {
            x,
            y,
            w,
            h,
            moncrc,
            maximized,
            scale,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_position_roundtrip() {
        let pos = WindowPosition {
            x: 100,
            y: 200,
            w: 800,
            h: 600,
            moncrc: 12345,
            maximized: 1,
            scale: 150,
        };
        let bytes = pos.serialize();
        assert_eq!(bytes.len(), 28);
        let decoded = WindowPosition::deserialize(&bytes).unwrap();
        assert_eq!(pos, decoded);
    }
}
