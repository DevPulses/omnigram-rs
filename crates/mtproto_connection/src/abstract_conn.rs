//! Abstract MTProto connection interface and packet helpers.
//!
//! Source: `tdesktop/Telegram/SourceFiles/mtproto/connection_abstract.{h,cpp}`.

use std::fmt;

/// Used to emit an error with no real MTProto error code from the server.
pub const ERROR_CODE_OTHER: i32 = -499;

/// Default full connection timeout in milliseconds (`kFullConnectionTimeout`).
pub const FULL_CONNECTION_TIMEOUT_MS: u64 = 8_000;

/// Shift applied to `protocol_dc_id` in test mode (`kTestModeDcIdShift`).
pub const TEST_MODE_DC_ID_SHIFT: i16 = 10_000;

/// Connection lifecycle state matching `TcpConnection::Status`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Waiting,
    Ready,
    Finished,
}

/// Events produced by a connection during polling or I/O.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionEvent {
    Connected,
    Disconnected,
    ReceivedPacket(Vec<u8>),
    Error(i32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionError {
    Io(String),
    Protocol(String),
    InvalidNotSecurePacket,
    NotConnected,
}

impl fmt::Display for ConnectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(msg) => write!(f, "I/O error: {msg}"),
            Self::Protocol(msg) => write!(f, "Protocol error: {msg}"),
            Self::InvalidNotSecurePacket => write!(f, "Invalid unencrypted MTProto packet"),
            Self::NotConnected => write!(f, "Connection is not ready"),
        }
    }
}

impl std::error::Error for ConnectionError {}

/// Prepares an unencrypted MTProto packet (`auth_key_id = 0`, `msg_id`, `msg_len`, `payload`).
/// Matches `AbstractConnection::prepareNotSecurePacket`.
pub fn prepare_not_secure_packet(msg_id: u64, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + 8 + 4 + payload.len());
    out.extend_from_slice(&0u64.to_le_bytes());
    out.extend_from_slice(&msg_id.to_le_bytes());
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(payload);
    out
}

/// Prepares an encrypted MTProto packet (`key_id`, `msg_key`, `encrypted_payload`).
/// Matches `AbstractConnection::prepareSecurePacket` payload framing.
pub fn prepare_secure_packet(
    key_id: u64,
    msg_key: [u8; 16],
    encrypted_payload: &[u8],
) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + 16 + encrypted_payload.len());
    out.extend_from_slice(&key_id.to_le_bytes());
    out.extend_from_slice(&msg_key);
    out.extend_from_slice(encrypted_payload);
    out
}

/// Parses an unencrypted MTProto response packet, returning `(msg_id, payload)`.
/// Matches `AbstractConnection::parseNotSecureResponse`.
pub fn parse_not_secure_response(packet: &[u8]) -> Result<(u64, &[u8]), ConnectionError> {
    // Minimum size: 8 (auth_key_id) + 8 (msg_id) + 4 (len) + 4 (at least 1 mtpPrime) = 24 bytes
    if packet.len() < 24 {
        return Err(ConnectionError::InvalidNotSecurePacket);
    }
    let auth_key_id = u64::from_le_bytes(packet[0..8].try_into().unwrap());
    if auth_key_id != 0 {
        return Err(ConnectionError::InvalidNotSecurePacket);
    }
    let msg_id = u64::from_le_bytes(packet[8..16].try_into().unwrap());
    let len = u32::from_le_bytes(packet[16..20].try_into().unwrap()) as usize;
    if 20 + len > packet.len() || len % 4 != 0 {
        return Err(ConnectionError::InvalidNotSecurePacket);
    }
    Ok((msg_id, &packet[20..20 + len]))
}

/// Formats a DC debug identifier matching `AbstractConnection::ProtocolDcDebugId`.
pub fn protocol_dc_debug_id(protocol_dc_id: i16) -> String {
    if protocol_dc_id == 0 {
        return String::new();
    }
    let abs_id = protocol_dc_id.abs();
    if abs_id > TEST_MODE_DC_ID_SHIFT {
        format!("test_{}", abs_id - TEST_MODE_DC_ID_SHIFT)
    } else {
        format!("prod_{abs_id}")
    }
}

/// Abstract connection interface matching `MTP::details::AbstractConnection`.
pub trait AbstractConnection {
    fn connect_to_server(
        &mut self,
        ip: &str,
        port: u16,
        protocol_secret: &[u8],
        protocol_dc_id: i16,
        protocol_for_files: bool,
    ) -> Result<(), ConnectionError>;

    fn disconnect_from_server(&mut self);
    fn send_data(&mut self, data: &[u8]) -> Result<(), ConnectionError>;
    fn poll_events(&mut self) -> Vec<ConnectionEvent>;
    fn is_connected(&self) -> bool;
    fn state(&self) -> ConnectionState;
    fn ping_time_ms(&self) -> u64;
    fn full_connect_timeout_ms(&self) -> u64 {
        FULL_CONNECTION_TIMEOUT_MS
    }
    fn transport_name(&self) -> &'static str;
    fn tag(&self) -> &'static str;
}
