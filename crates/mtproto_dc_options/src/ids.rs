//! Data center ids and the shift arithmetic around them.
//!
//! Source of truth: `mtproto/core_types.h` and `mtproto/facade.h`.
//!
//! A bare id names a data center; a shifted id names a connection to it for one
//! purpose (config fetch, upload, download slot, group call stream, ...). The
//! shift is always a multiple of [`DC_SHIFT`], so the bare id is the remainder.

/// `MTP::DcId`.
pub type DcId = i32;
/// `MTP::ShiftedDcId`.
pub type ShiftedDcId = i32;

/// `MTP::kDcShift`.
pub const DC_SHIFT: ShiftedDcId = 10_000;
/// `MTP::kConfigDcShift`.
pub const CONFIG_DC_SHIFT: i32 = 0x01;
/// `MTP::kLogoutDcShift`.
pub const LOGOUT_DC_SHIFT: i32 = 0x02;
/// `MTP::kUpdaterDcShift`.
pub const UPDATER_DC_SHIFT: i32 = 0x03;
/// `MTP::kExportDcShift`.
pub const EXPORT_DC_SHIFT: i32 = 0x04;
/// `MTP::kExportMediaDcShift`.
pub const EXPORT_MEDIA_DC_SHIFT: i32 = 0x05;
/// `MTP::kGroupCallStreamDcShift`.
pub const GROUP_CALL_STREAM_DC_SHIFT: i32 = 0x06;
/// `MTP::kMaxMediaDcCount`.
pub const MAX_MEDIA_DC_COUNT: i32 = 0x10;
/// `MTP::kBaseDownloadDcShift`.
pub const BASE_DOWNLOAD_DC_SHIFT: i32 = 0x10;
/// `MTP::kBaseUploadDcShift`.
pub const BASE_UPLOAD_DC_SHIFT: i32 = 0x20;
/// `MTP::Instance::Fields::kTemporaryMainDc` — ids from here up are temporary.
pub const TEMPORARY_MAIN_DC: DcId = 1000;

/// `MTP::BareDcId`.
#[must_use]
pub const fn bare_dc_id(shifted_dc_id: ShiftedDcId) -> DcId {
    shifted_dc_id % DC_SHIFT
}

/// `MTP::ShiftDcId`.
#[must_use]
pub const fn shift_dc_id(dc_id: DcId, value: i32) -> ShiftedDcId {
    dc_id + DC_SHIFT * value
}

/// `MTP::GetDcIdShift`.
#[must_use]
pub const fn get_dc_id_shift(shifted_dc_id: ShiftedDcId) -> i32 {
    shifted_dc_id / DC_SHIFT
}

/// `MTP::configDcId` — for enumerating data centers.
#[must_use]
pub const fn config_dc_id(dc_id: DcId) -> ShiftedDcId {
    shift_dc_id(dc_id, CONFIG_DC_SHIFT)
}

/// `MTP::logoutDcId`.
#[must_use]
pub const fn logout_dc_id(dc_id: DcId) -> ShiftedDcId {
    shift_dc_id(dc_id, LOGOUT_DC_SHIFT)
}

/// `MTP::updaterDcId`.
#[must_use]
pub const fn updater_dc_id(dc_id: DcId) -> ShiftedDcId {
    shift_dc_id(dc_id, UPDATER_DC_SHIFT)
}

/// `MTP::groupCallStreamDcId`.
#[must_use]
pub const fn group_call_stream_dc_id(dc_id: DcId) -> ShiftedDcId {
    shift_dc_id(dc_id, GROUP_CALL_STREAM_DC_SHIFT)
}

/// `MTP::downloadDcId`.
#[must_use]
pub const fn download_dc_id(dc_id: DcId, index: i32) -> ShiftedDcId {
    shift_dc_id(dc_id, BASE_DOWNLOAD_DC_SHIFT + index)
}

/// `MTP::isDownloadDcId` — a download slot of any data center.
#[must_use]
pub const fn is_download_dc_id(shifted_dc_id: ShiftedDcId) -> bool {
    shifted_dc_id >= download_dc_id(0, 0)
        && shifted_dc_id < download_dc_id(0, MAX_MEDIA_DC_COUNT - 1) + DC_SHIFT
}

/// `MTP::isMediaClusterDcId` — a connection that serves media only.
#[must_use]
pub const fn is_media_cluster_dc_id(shifted_dc_id: ShiftedDcId) -> bool {
    let shift = get_dc_id_shift(shifted_dc_id);
    is_download_dc_id(shifted_dc_id)
        || shift == GROUP_CALL_STREAM_DC_SHIFT
        || shift == EXPORT_MEDIA_DC_SHIFT
        || shift == UPDATER_DC_SHIFT
}

/// `MTP::isTemporaryDcId` — a data center opened for one guest session.
#[must_use]
pub const fn is_temporary_dc_id(shifted_dc_id: ShiftedDcId) -> bool {
    bare_dc_id(shifted_dc_id) >= TEMPORARY_MAIN_DC
}

/// `MTP::DcType` — what a connection to a data center is for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DcType {
    Regular,
    Temporary,
    MediaCluster,
    Cdn,
}
