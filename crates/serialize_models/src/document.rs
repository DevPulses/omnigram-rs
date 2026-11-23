//! Document serialization/deserialization.
//!
//! Source: `storage/serialize_document.{h,cpp}`.

use crate::image_location::ImageLocation;
use storage_serialize::{ByteArrayReader, ByteArrayWriter};

pub const VERSION_TAG: i32 = 0x7FFF_FFFF;
pub const VERSION: i32 = 6;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DocumentData {
    pub id: u64,
    pub access: u64,
    pub date: i32,
    pub file_reference: Vec<u8>,
    pub filename: String,
    pub mime: String,
    pub dc: i32,
    pub size: i32,
    pub width: i32,
    pub height: i32,
    pub doc_type: i32,
    pub duration: i64,
}

pub fn write_document(writer: &mut ByteArrayWriter, doc: &DocumentData) {
    writer.write_u64(doc.id);
    writer.write_u64(doc.access);
    writer.write_i32(doc.date);
    writer.write_byte_array(Some(&doc.file_reference));
    writer.write_i32(VERSION_TAG);
    writer.write_i32(VERSION);
    writer.write_string(Some(&doc.filename));
    writer.write_string(Some(&doc.mime));
    writer.write_i32(doc.dc);
    writer.write_i32(doc.size);
    writer.write_i32(doc.width);
    writer.write_i32(doc.height);
    writer.write_i32(doc.doc_type);

    writer.write_i64(doc.duration);

    // Dummy thumbnail locations
    let dummy_thumb = ImageLocation::default();
    dummy_thumb.write(writer);
    writer.write_i32(0); // thumbnailByteSize
    dummy_thumb.write(writer);
    writer.write_i32(0); // videoThumbnailByteSize
    writer.write_i32(0); // inlineThumbnailIsPath
    writer.write_byte_array(Some(&[])); // inlineThumbnailBytes
}

pub fn read_document(reader: &mut ByteArrayReader, stream_version: i32) -> Option<DocumentData> {
    let id = reader.read_u64()?;
    let access = reader.read_u64()?;
    let date = reader.read_i32()?;
    let mut file_reference = Vec::new();

    let mut version = 0;
    if stream_version >= 9061 {
        if stream_version >= 1003013 {
            file_reference = reader.read_byte_array()?;
        }
        let version_tag = reader.read_i32()?;
        if version_tag == VERSION_TAG {
            version = reader.read_i32()?;
        }
    }

    let filename = reader.read_string()?;
    let mime = reader.read_string()?;
    let dc = reader.read_i32()?;
    let size = reader.read_i32()?;
    let width = reader.read_i32()?;
    let height = reader.read_i32()?;
    let doc_type = reader.read_i32()?;

    let mut duration = -1;
    if version > 0 {
        duration = reader.read_i64()?;
    }

    // Skip thumbnails
    let _thumb = ImageLocation::read(reader, stream_version)?;
    let _thumb_size = reader.read_i32()?;
    let _video_thumb = ImageLocation::read(reader, stream_version)?;
    let _video_thumb_size = reader.read_i32()?;
    let _inline_is_path = reader.read_i32()?;
    let _inline_bytes = reader.read_byte_array()?;

    Some(DocumentData {
        id,
        access,
        date,
        file_reference,
        filename,
        mime,
        dc,
        size,
        width,
        height,
        doc_type,
        duration,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_roundtrip() {
        let doc = DocumentData {
            id: 998877,
            access: 112233,
            date: 1600000000,
            file_reference: vec![1, 2, 3, 4],
            filename: "photo.jpg".to_string(),
            mime: "image/jpeg".to_string(),
            dc: 2,
            size: 1024,
            width: 800,
            height: 600,
            doc_type: 0,
            duration: -1,
        };

        let mut writer = ByteArrayWriter::new();
        write_document(&mut writer, &doc);

        let mut reader = ByteArrayReader::from_slice(writer.result());
        let read = read_document(&mut reader, 1003013).unwrap();

        assert_eq!(read.id, doc.id);
        assert_eq!(read.access, doc.access);
        assert_eq!(read.filename, doc.filename);
        assert_eq!(read.mime, doc.mime);
        assert_eq!(read.file_reference, doc.file_reference);
    }
}
