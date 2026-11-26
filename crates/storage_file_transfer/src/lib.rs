//! Chunked MTProto file upload and download state machines.
//!
//! Ported from `tdesktop/Telegram/SourceFiles/storage/file_upload.{h,cpp}`
//! and `tdesktop/Telegram/SourceFiles/storage/file_download.{h,cpp}`.

pub mod download;
pub mod upload;

pub use download::{
    DownloadPartRequest, FileDownloadTask, FileFailureReason, MAX_FILE_IN_MEMORY,
    MAX_STICKER_BYTES_SIZE, MAX_WALLPAPER_DIMENSION,
};
pub use upload::{
    choose_upload_part_size, FileUploadTask, UploadPartRequest, DEFAULT_UPLOAD_PART_SIZE,
    USE_BIG_FILES_FROM,
};
