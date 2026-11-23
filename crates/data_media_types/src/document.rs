use serde::{Deserialize, Serialize};
use crate::photo::Size;

pub type DocumentId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StickerType {
    Webp,
    Tgs,
    Webm,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StickerData {
    pub alt: String,
    pub set_id: u64,
    pub set_access_hash: u64,
    pub sticker_type: StickerType,
}

impl StickerData {
    pub fn is_static(&self) -> bool {
        self.sticker_type == StickerType::Webp
    }

    pub fn is_lottie(&self) -> bool {
        self.sticker_type == StickerType::Tgs
    }

    pub fn is_webm(&self) -> bool {
        self.sticker_type == StickerType::Webm
    }

    pub fn is_animated(&self) -> bool {
        !self.is_static()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SongData {
    pub title: String,
    pub performer: String,
    pub duration: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoiceData {
    pub duration: i32,
    pub waveform: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VideoData {
    pub duration: i32,
    pub size: Size,
    pub supports_streaming: bool,
    pub is_round: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentAdditionalData {
    Sticker(StickerData),
    Song(SongData),
    Voice(VoiceData),
    Video(VideoData),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentData {
    pub id: DocumentId,
    pub access_hash: u64,
    pub file_reference: Vec<u8>,
    pub date: i32,
    pub mime_type: String,
    pub size: i64,
    pub dc: i32,
    pub filename: String,
    pub thumbnail_bytes: Option<Vec<u8>>,
    pub dimensions: Size,
    pub additional: Option<DocumentAdditionalData>,
}

impl DocumentData {
    pub fn new(id: DocumentId, access_hash: u64, mime_type: String, size: i64, dc: i32) -> Self {
        Self {
            id,
            access_hash,
            file_reference: Vec::new(),
            date: 0,
            mime_type,
            size,
            dc,
            filename: String::new(),
            thumbnail_bytes: None,
            dimensions: Size::new(0, 0),
            additional: None,
        }
    }

    pub fn is_sticker(&self) -> bool {
        matches!(self.additional, Some(DocumentAdditionalData::Sticker(_)))
    }

    pub fn is_voice(&self) -> bool {
        matches!(self.additional, Some(DocumentAdditionalData::Voice(_)))
    }

    pub fn is_video(&self) -> bool {
        matches!(self.additional, Some(DocumentAdditionalData::Video(_)))
    }

    pub fn is_song(&self) -> bool {
        matches!(self.additional, Some(DocumentAdditionalData::Song(_)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::photo::*;

    #[test]
    fn test_photo_data_creation() {
        let mut photo = PhotoData::new(12345, 1700000000);
        assert_eq!(photo.id, 12345);
        assert!(!photo.is_null());
        assert!(!photo.has_video());

        photo.large = Some(PhotoSizeData {
            location: ImageLocation {
                dc: 2,
                volume_id: 100,
                local_id: 200,
                secret: 300,
                file_reference: vec![1, 2, 3],
            },
            size: Size::new(800, 600),
            byte_size: 45000,
        });

        assert_eq!(photo.best_dimension(), Size::new(800, 600));
        assert!(photo.size_data(PhotoSize::Large).is_some());
    }

    #[test]
    fn test_document_sticker_attributes() {
        let mut doc = DocumentData::new(999, 888, "image/webp".to_string(), 12000, 2);
        doc.additional = Some(DocumentAdditionalData::Sticker(StickerData {
            alt: "👍".to_string(),
            set_id: 1,
            set_access_hash: 2,
            sticker_type: StickerType::Webp,
        }));

        assert!(doc.is_sticker());
        if let Some(DocumentAdditionalData::Sticker(s)) = &doc.additional {
            assert!(s.is_static());
            assert!(!s.is_animated());
            assert_eq!(s.alt, "👍");
        } else {
            panic!("Expected sticker");
        }
    }
}
