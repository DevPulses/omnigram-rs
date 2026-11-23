//! ImageLocation serialization ported from Telegram Desktop.
//!
//! Source: `storage/serialize_peer.cpp`.

use storage_serialize::{ByteArrayReader, ByteArrayWriter};

pub const MODERN_IMAGE_LOCATION_TAG: i32 = i32::MIN;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ImageLocation {
    pub width: i32,
    pub height: i32,
    pub dc: i32,
    pub volume: u64,
    pub local: i32,
    pub secret: u64,
    pub file_reference: Vec<u8>,
}

impl ImageLocation {
    pub fn write(&self, writer: &mut ByteArrayWriter) {
        writer.write_i32(MODERN_IMAGE_LOCATION_TAG);
        // Serialize internal fields
        let mut inner = ByteArrayWriter::new();
        inner.write_i32(self.width);
        inner.write_i32(self.height);
        inner.write_i32(self.dc);
        inner.write_u64(self.volume);
        inner.write_i32(self.local);
        inner.write_u64(self.secret);
        inner.write_byte_array(Some(&self.file_reference));
        writer.write_byte_array(Some(inner.result()));
    }

    pub fn read(reader: &mut ByteArrayReader, _stream_version: i32) -> Option<Self> {
        let tag_or_width = reader.read_i32()?;
        if tag_or_width == MODERN_IMAGE_LOCATION_TAG {
            let serialized = reader.read_byte_array()?;
            let mut inner = ByteArrayReader::from_slice(&serialized);
            let width = inner.read_i32()?;
            let height = inner.read_i32()?;
            let dc = inner.read_i32()?;
            let volume = inner.read_u64()?;
            let local = inner.read_i32()?;
            let secret = inner.read_u64()?;
            let file_reference = inner.read_byte_array()?;
            Some(Self {
                width,
                height,
                dc,
                volume,
                local,
                secret,
                file_reference,
            })
        } else {
            // Legacy format
            let width = tag_or_width;
            let height = reader.read_i32()?;
            let dc = reader.read_i32()?;
            let volume = reader.read_u64()?;
            let local = reader.read_i32()?;
            let secret = reader.read_u64()?;
            let file_reference = reader.read_byte_array()?;
            Some(Self {
                width,
                height,
                dc,
                volume,
                local,
                secret,
                file_reference,
            })
        }
    }
}
