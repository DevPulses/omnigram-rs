//! `serialize_models` — Storage serialization of Peer and Document models.
//!
//! Source:
//! - `tdesktop/Telegram/SourceFiles/storage/serialize_peer.{h,cpp}`
//! - `tdesktop/Telegram/SourceFiles/storage/serialize_document.{h,cpp}`

pub mod image_location;
pub mod peer;
pub mod document;

pub use image_location::ImageLocation;
pub use peer::{read_peer, write_peer};
pub use document::{DocumentData, read_document, write_document};
