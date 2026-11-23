use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Size {
    pub width: i32,
    pub height: i32,
}

impl Size {
    pub fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }
}

pub type PhotoId = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PhotoSize {
    Small,
    Thumbnail,
    Large,
}

impl PhotoSize {
    pub const COUNT: usize = 3;

    pub fn index(self) -> usize {
        match self {
            PhotoSize::Small => 0,
            PhotoSize::Thumbnail => 1,
            PhotoSize::Large => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageLocation {
    pub dc: i32,
    pub volume_id: u64,
    pub local_id: i32,
    pub secret: u64,
    pub file_reference: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhotoSizeData {
    pub location: ImageLocation,
    pub size: Size,
    pub byte_size: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhotoData {
    pub id: PhotoId,
    pub date: i32,
    pub has_attached_stickers: bool,
    pub inline_thumbnail_bytes: Option<Vec<u8>>,
    pub small: Option<PhotoSizeData>,
    pub thumbnail: Option<PhotoSizeData>,
    pub large: Option<PhotoSizeData>,
    pub video_small: Option<PhotoSizeData>,
    pub video_large: Option<PhotoSizeData>,
    pub video_duration: Option<i32>,
    pub waiting_for_album: bool,
}

impl PhotoData {
    pub fn new(id: PhotoId, date: i32) -> Self {
        Self {
            id,
            date,
            has_attached_stickers: false,
            inline_thumbnail_bytes: None,
            small: None,
            thumbnail: None,
            large: None,
            video_small: None,
            video_large: None,
            video_duration: None,
            waiting_for_album: false,
        }
    }

    pub fn is_null(&self) -> bool {
        self.id == 0
    }

    pub fn has_video(&self) -> bool {
        self.video_large.is_some() || self.video_small.is_some()
    }

    pub fn size_data(&self, size: PhotoSize) -> Option<&PhotoSizeData> {
        match size {
            PhotoSize::Small => self.small.as_ref(),
            PhotoSize::Thumbnail => self.thumbnail.as_ref(),
            PhotoSize::Large => self.large.as_ref(),
        }
    }

    pub fn best_dimension(&self) -> Size {
        if let Some(l) = &self.large {
            l.size
        } else if let Some(t) = &self.thumbnail {
            t.size
        } else if let Some(s) = &self.small {
            s.size
        } else {
            Size::new(0, 0)
        }
    }
}
