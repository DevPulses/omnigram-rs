//! MTProto abstract and TCP connection layer.
//!
//! Ported from `tdesktop/Telegram/SourceFiles/mtproto/connection_abstract.{h,cpp}`
//! and `tdesktop/Telegram/SourceFiles/mtproto/connection_tcp.{h,cpp}`.

pub mod abstract_conn;
pub mod tcp_conn;

pub use abstract_conn::{
    parse_not_secure_response, prepare_not_secure_packet, prepare_secure_packet,
    protocol_dc_debug_id, AbstractConnection, ConnectionError, ConnectionEvent, ConnectionState,
    ERROR_CODE_OTHER, FULL_CONNECTION_TIMEOUT_MS, TEST_MODE_DC_ID_SHIFT,
};
pub use tcp_conn::{TcpConnection, TcpTransportCodec};
