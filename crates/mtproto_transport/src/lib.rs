//! MTProto TCP obfuscation and framing protocol implementation.
//!
//! Source of truth:
//! `tdesktop/Telegram/SourceFiles/mtproto/connection_tcp.cpp`
//! `tdesktop/Telegram/SourceFiles/mtproto/details/mtproto_tcp_socket.cpp`
//! `tdesktop/Telegram/SourceFiles/mtproto/mtproto_auth_key.cpp`

pub mod ctr128;
pub mod framing;
pub mod obfuscation;

pub use ctr128::Ctr128;
pub use framing::{AbridgedFraming, IntermediateFraming, PaddedFraming, Framing};
pub use obfuscation::{ObfuscatedStream, ProtocolVersion};
