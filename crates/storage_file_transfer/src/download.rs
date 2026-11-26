//! Chunked download task state machine matching `FileLoader`.
//!
//! Source: `tdesktop/Telegram/SourceFiles/storage/file_download.{h,cpp}`.

pub const MAX_FILE_IN_MEMORY: usize = 10 * 1024 * 1024;
pub const MAX_STICKER_BYTES_SIZE: usize = 2 * 1024 * 1024;
pub const MAX_WALLPAPER_DIMENSION: i32 = 4096;
pub const DOWNLOAD_CHUNK_SIZE: usize = 128 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFailureReason {
    NoFailure,
    FileWriteFailure,
    OtherFailure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadPartRequest {
    pub dc_id: i32,
    pub offset: u64,
    pub limit: usize,
}

#[derive(Debug, Clone)]
pub struct FileDownloadTask {
    pub dc_id: i32,
    pub full_size: u64,
    pub chunk_size: usize,
    pub data: Vec<u8>,
    pub offset: u64,
    pub finished: bool,
    pub cancelled: bool,
    pub failure: FileFailureReason,
}

impl FileDownloadTask {
    pub fn new(dc_id: i32, full_size: u64) -> Self {
        Self {
            dc_id,
            full_size,
            chunk_size: DOWNLOAD_CHUNK_SIZE,
            data: Vec::new(),
            offset: 0,
            finished: false,
            cancelled: false,
            failure: FileFailureReason::NoFailure,
        }
    }

    pub fn next_request(&self) -> Option<DownloadPartRequest> {
        if self.finished || self.cancelled || self.failure != FileFailureReason::NoFailure {
            return None;
        }
        Some(DownloadPartRequest {
            dc_id: self.dc_id,
            offset: self.offset,
            limit: self.chunk_size,
        })
    }

    pub fn on_chunk_received(&mut self, bytes: &[u8]) {
        if self.cancelled || self.finished {
            return;
        }
        self.data.extend_from_slice(bytes);
        self.offset += bytes.len() as u64;
        if bytes.len() < self.chunk_size || (self.full_size > 0 && self.offset >= self.full_size) {
            self.finished = true;
        }
    }

    pub fn cancel(&mut self) {
        self.cancelled = true;
    }

    pub fn fail(&mut self, reason: FileFailureReason) {
        self.failure = reason;
    }

    pub fn progress(&self) -> f64 {
        if self.full_size == 0 {
            return if self.finished { 1.0 } else { 0.0 };
        }
        (self.offset as f64 / self.full_size as f64).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::upload::*;

    #[test]
    fn test_upload_chunks_and_md5() {
        let payload = vec![0x41u8; 40_000]; // > 32 KiB => 2 parts
        let mut up = FileUploadTask::new(123456, payload.len() as u64);
        assert_eq!(up.total_parts, 2);
        assert!(!up.is_big_file);

        let p0 = up.next_chunk(&payload).unwrap();
        assert_eq!(p0.file_part, 0);
        assert_eq!(p0.bytes.len(), 32 * 1024);
        up.on_part_acked();

        let p1 = up.next_chunk(&payload).unwrap();
        assert_eq!(p1.file_part, 1);
        assert_eq!(p1.bytes.len(), 40_000 - 32 * 1024);
        up.on_part_acked();

        assert!(up.is_finished());
        assert!((up.progress() - 1.0).abs() < 1e-6);
        assert!(up.finalize_md5_hex().is_some());
    }

    #[test]
    fn test_download_chunks() {
        let mut dl = FileDownloadTask::new(2, 150_000);
        let req1 = dl.next_request().unwrap();
        assert_eq!(req1.offset, 0);
        dl.on_chunk_received(&vec![1u8; DOWNLOAD_CHUNK_SIZE]);
        assert!(!dl.finished);

        let req2 = dl.next_request().unwrap();
        assert_eq!(req2.offset, DOWNLOAD_CHUNK_SIZE as u64);
        dl.on_chunk_received(&vec![2u8; 150_000 - DOWNLOAD_CHUNK_SIZE]);
        assert!(dl.finished);
        assert_eq!(dl.data.len(), 150_000);
    }
}
