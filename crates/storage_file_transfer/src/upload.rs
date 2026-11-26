//! Chunked upload task state machine matching `Storage::Uploader`.
//!
//! Source: `tdesktop/Telegram/SourceFiles/storage/file_upload.{h,cpp}`.

use md5::{Digest, Md5};

/// MTP big files methods used for files greater than 30 MiB (`kUseBigFilesFrom`).
pub const USE_BIG_FILES_FROM: u64 = 30 * 1024 * 1024;
pub const DEFAULT_UPLOAD_PART_SIZE: usize = 512 * 1024;

/// Chooses an MTProto-compliant upload part size (divisor of 512 KiB, max 512 KiB, <= 3000 parts).
pub fn choose_upload_part_size(total_size: u64) -> usize {
    let mut part_size = 32 * 1024usize;
    while total_size > (part_size as u64) * 3000 && part_size < DEFAULT_UPLOAD_PART_SIZE {
        part_size *= 2;
    }
    part_size
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadPartRequest {
    pub file_id: u64,
    pub file_part: u32,
    pub file_total_parts: u32,
    pub is_big_file: bool,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct FileUploadTask {
    pub file_id: u64,
    pub total_size: u64,
    pub part_size: usize,
    pub total_parts: u32,
    pub is_big_file: bool,
    next_part: u32,
    acked_parts: u32,
    paused: bool,
    cancelled: bool,
    md5: Md5,
}

impl FileUploadTask {
    pub fn new(file_id: u64, total_size: u64) -> Self {
        let part_size = choose_upload_part_size(total_size);
        let total_parts = total_size.div_ceil(part_size as u64).max(1) as u32;
        Self {
            file_id,
            total_size,
            part_size,
            total_parts,
            is_big_file: total_size >= USE_BIG_FILES_FROM,
            next_part: 0,
            acked_parts: 0,
            paused: false,
            cancelled: false,
            md5: Md5::new(),
        }
    }

    pub fn next_chunk(&mut self, file_bytes: &[u8]) -> Option<UploadPartRequest> {
        if self.paused || self.cancelled || self.next_part >= self.total_parts {
            return None;
        }
        let part_idx = self.next_part;
        let start = (part_idx as usize) * self.part_size;
        let end = (start + self.part_size).min(file_bytes.len());
        let chunk = file_bytes.get(start..end)?.to_vec();
        if !self.is_big_file {
            self.md5.update(&chunk);
        }
        self.next_part += 1;
        Some(UploadPartRequest {
            file_id: self.file_id,
            file_part: part_idx,
            file_total_parts: self.total_parts,
            is_big_file: self.is_big_file,
            bytes: chunk,
        })
    }

    pub fn on_part_acked(&mut self) {
        self.acked_parts = (self.acked_parts + 1).min(self.total_parts);
    }

    pub fn is_finished(&self) -> bool {
        self.acked_parts >= self.total_parts
    }

    pub fn progress(&self) -> f64 {
        (self.acked_parts as f64 / self.total_parts as f64).clamp(0.0, 1.0)
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;
    }

    pub fn cancel(&mut self) {
        self.cancelled = true;
    }

    pub fn finalize_md5_hex(self) -> Option<String> {
        if self.is_big_file {
            return None;
        }
        let digest = self.md5.finalize();
        Some(digest.iter().map(|b| format!("{b:02x}")).collect())
    }
}
