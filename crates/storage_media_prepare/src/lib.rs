//! Media upload preparation, file classification, and size limit checks.
//!
//! Ported from `tdesktop/Telegram/SourceFiles/storage/storage_media_prepare.{h,cpp}`.

use std::path::Path;

use data_media_types::Size;

pub const MAX_FILE_SIZE_FREE: u64 = 2 * 1024 * 1024 * 1024; // 2 GiB
pub const MAX_FILE_SIZE_PREMIUM: u64 = 4 * 1024 * 1024 * 1024; // 4 GiB
pub const PHOTO_SIDE_LIMIT: i32 = 1280;
pub const PHOTO_HD_SIDE_LIMIT: i32 = 2560;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MimeDataState {
    None,
    Files,
    FilesArchive,
    FilesArchiveOnly,
    PhotoFiles,
    PhotoFilesArchive,
    MediaFiles,
    MediaFilesArchive,
    Image,
    Media,
    Folder,
    FolderArchiveOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparedFileType {
    Photo,
    Video,
    Audio,
    File,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlbumType {
    None,
    PhotoVideo,
    Music,
    Files,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedFile {
    pub path: String,
    pub display_name: String,
    pub file_size: u64,
    pub file_type: PreparedFileType,
    pub dimensions: Size,
    pub send_large_photos: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedList {
    pub files: Vec<PreparedFile>,
    pub album_type: AlbumType,
    pub error_message: Option<String>,
}

impl PreparedList {
    pub fn empty() -> Self {
        Self {
            files: Vec::new(),
            album_type: AlbumType::None,
            error_message: None,
        }
    }
}

pub fn max_upload_size(is_premium: bool) -> u64 {
    if is_premium {
        MAX_FILE_SIZE_PREMIUM
    } else {
        MAX_FILE_SIZE_FREE
    }
}

pub fn classify_extension(path: &str) -> PreparedFileType {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "jpg" | "jpeg" | "png" | "webp" | "bmp" | "tiff" => PreparedFileType::Photo,
        "mp4" | "mov" | "mkv" | "webm" | "avi" => PreparedFileType::Video,
        "mp3" | "ogg" | "opus" | "flac" | "wav" | "m4a" => PreparedFileType::Audio,
        _ => PreparedFileType::File,
    }
}

pub fn prepare_media_list(entries: &[(&str, u64)], is_premium: bool) -> PreparedList {
    if entries.is_empty() {
        return PreparedList::empty();
    }

    let limit = max_upload_size(is_premium);
    let mut files = Vec::with_capacity(entries.len());
    let mut has_photo_video = false;
    let mut has_audio = false;
    let mut has_other = false;

    for &(path, size) in entries {
        if size == 0 {
            return PreparedList {
                files: Vec::new(),
                album_type: AlbumType::None,
                error_message: Some(format!("File is empty: {path}")),
            };
        }
        if size > limit {
            return PreparedList {
                files: Vec::new(),
                album_type: AlbumType::None,
                error_message: Some(format!("File exceeds upload limit: {path}")),
            };
        }

        let file_type = classify_extension(path);
        match file_type {
            PreparedFileType::Photo | PreparedFileType::Video => has_photo_video = true,
            PreparedFileType::Audio => has_audio = true,
            PreparedFileType::File => has_other = true,
        }

        let display_name = Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(path)
            .to_string();

        files.push(PreparedFile {
            path: path.to_string(),
            display_name,
            file_size: size,
            file_type,
            dimensions: Size::new(0, 0),
            send_large_photos: false,
        });
    }

    let album_type = if files.len() > 1 {
        if has_photo_video && !has_audio && !has_other {
            AlbumType::PhotoVideo
        } else if has_audio && !has_photo_video && !has_other {
            AlbumType::Music
        } else {
            AlbumType::Files
        }
    } else {
        AlbumType::None
    };

    PreparedList {
        files,
        album_type,
        error_message: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_and_album_grouping() {
        let list = prepare_media_list(
            &[("/tmp/a.jpg", 1024), ("/tmp/b.mp4", 2048)],
            false,
        );
        assert!(list.error_message.is_none());
        assert_eq!(list.album_type, AlbumType::PhotoVideo);
        assert_eq!(list.files[0].display_name, "a.jpg");
    }

    #[test]
    fn test_size_limits_free_vs_premium() {
        let three_gib = 3 * 1024 * 1024 * 1024u64;
        let free_res = prepare_media_list(&[("/tmp/big.zip", three_gib)], false);
        assert!(free_res.error_message.is_some());

        let prem_res = prepare_media_list(&[("/tmp/big.zip", three_gib)], true);
        assert!(prem_res.error_message.is_none());
        assert_eq!(prem_res.files[0].file_type, PreparedFileType::File);
    }
}
