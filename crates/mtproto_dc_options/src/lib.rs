//! Connect options of every Telegram data center.
//!
//! Source of truth: `tdesktop/Telegram/SourceFiles/mtproto/mtproto_dc_options.{h,cpp}`.
//!
//! The store answers three questions: which endpoint to connect to for a given
//! dc and purpose, whether an endpoint is a CDN or a media-only one, and how the
//! whole set is written to disk. It is what `tdata_settings` hands the legacy
//! `dbiDcOptionsOld` blob to, and what the transport asks for a route.
//!
//! Layers:
//! - [`ids`] — dc ids, shifts, temporary and media-cluster detection.
//! - [`flags`] — the `dcOption` bits.
//! - [`endpoint`] — one endpoint.
//! - [`lookup`] — `lookup`, split by address family and protocol.
//! - [`options`] — the store: built-ins, merging, serialization.
//! - [`file`] — the `.tdesktop-endpoints` override file.
//!
//! Not ported here: the RSA public key store (`getDcRSAKey`, `setCDNConfig`)
//! and the `rpl` change streams, which belong to the RSA module and to the
//! reactive layer. CDN keys read from a blob are preserved verbatim instead.

pub mod blob;
pub mod endpoint;
pub mod environment;
pub mod file;
pub mod flags;
pub mod ids;
pub mod lookup;
pub mod options;

pub use endpoint::Endpoint;
pub use environment::Environment;
pub use flags::{Flags, CDN, IPV6, MEDIA_ONLY, SECRET, STATIC, TCPO_ONLY, THIS_PORT_ONLY};
pub use ids::{
    bare_dc_id, config_dc_id, download_dc_id, get_dc_id_shift, group_call_stream_dc_id,
    is_download_dc_id, is_media_cluster_dc_id, is_temporary_dc_id, logout_dc_id, shift_dc_id,
    updater_dc_id, DcId, DcType, ShiftedDcId, DC_SHIFT,
};
pub use lookup::{Variants, HTTP, IPV4, IPV6_INDEX, TCP};
pub use options::{CdnPublicKey, DcOptions, OptionEntry};
