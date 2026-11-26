//! MTProto 2.0 RPC multiplexer, container/gzip unpacking, and session client.
//!
//! Ported from `tdesktop/Telegram/SourceFiles/mtproto/sender.{h,cpp}`
//! and `tdesktop/Telegram/SourceFiles/mtproto/session.{h,cpp}`.

use std::collections::{HashMap, VecDeque};
use std::io::Read;

use flate2::read::GzDecoder;
use mtproto_connection::{AbstractConnection, ConnectionError, ConnectionEvent};
use mtproto_session::{
    MtprotoSession, MTPC_GZIP_PACKED, MTPC_MSG_CONTAINER, MTPC_RPC_RESULT,
};

pub const MTPC_RPC_ERROR: u32 = 0x2144ca19;
pub const MTPC_NEW_SESSION_CREATED: u32 = 0x9ec20908;
pub const MTPC_MSGS_ACK: u32 = 0x62d6b459;
pub const MTPC_BAD_MSG_NOTIFICATION: u32 = 0xa7eff811;
pub const MTPC_BAD_SERVER_SALT: u32 = 0xedab447b;
pub const MTPC_PING: u32 = 0x7abe77ec;
pub const MTPC_PONG: u32 = 0x347773c5;
pub const MTPC_VECTOR: u32 = 0x1cb5c415;

pub type RequestId = u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MtpError {
    pub code: i32,
    pub error_type: String,
}

impl MtpError {
    pub fn new(code: i32, error_type: impl Into<String>) -> Self {
        Self {
            code,
            error_type: error_type.into(),
        }
    }

    pub fn is_default_handled(&self) -> bool {
        self.error_type.starts_with("FLOOD_WAIT_")
            || self.error_type == "AUTH_KEY_UNREGISTERED"
            || self.error_type == "USER_DEACTIVATED"
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientError {
    Connection(ConnectionError),
    Decrypt(&'static str),
    Decode(String),
}

impl From<ConnectionError> for ClientError {
    fn from(e: ConnectionError) -> Self {
        Self::Connection(e)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientEvent {
    Connected,
    Disconnected,
    ConnectionError(i32),
    RpcResponse {
        request_id: RequestId,
        payload: Vec<u8>,
    },
    RpcError {
        request_id: RequestId,
        error: MtpError,
    },
    Pong {
        ping_id: u64,
        msg_id: u64,
    },
    NewSessionCreated {
        first_msg_id: u64,
        server_salt: u64,
    },
    BadServerSalt {
        bad_msg_id: u64,
        new_server_salt: u64,
    },
    Update(Vec<u8>),
}

fn read_u32_le(buf: &[u8], offset: &mut usize) -> Result<u32, ClientError> {
    if *offset + 4 > buf.len() {
        return Err(ClientError::Decode("Unexpected EOF reading u32".into()));
    }
    let val = u32::from_le_bytes(buf[*offset..*offset + 4].try_into().unwrap());
    *offset += 4;
    Ok(val)
}

fn read_u64_le(buf: &[u8], offset: &mut usize) -> Result<u64, ClientError> {
    if *offset + 8 > buf.len() {
        return Err(ClientError::Decode("Unexpected EOF reading u64".into()));
    }
    let val = u64::from_le_bytes(buf[*offset..*offset + 8].try_into().unwrap());
    *offset += 8;
    Ok(val)
}

fn read_tl_bytes(buf: &[u8], offset: &mut usize) -> Result<Vec<u8>, ClientError> {
    if *offset >= buf.len() {
        return Err(ClientError::Decode("Unexpected EOF reading TL bytes".into()));
    }
    let first = buf[*offset];
    let (len, header_len) = if first < 254 {
        (*offset += 1, (first as usize, 1))
    } else {
        if *offset + 4 > buf.len() {
            return Err(ClientError::Decode("Unexpected EOF reading long TL bytes".into()));
        }
        let len = (buf[*offset + 1] as usize)
            | ((buf[*offset + 2] as usize) << 8)
            | ((buf[*offset + 3] as usize) << 16);
        *offset += 4;
        ((), (len, 4))
    }
    .1;
    if *offset + len > buf.len() {
        return Err(ClientError::Decode("TL bytes truncated".into()));
    }
    let data = buf[*offset..*offset + len].to_vec();
    let pad = (4 - ((header_len + len) % 4)) % 4;
    *offset = (*offset + len + pad).min(buf.len());
    Ok(data)
}

fn gunzip(data: &[u8]) -> Result<Vec<u8>, ClientError> {
    let mut decoder = GzDecoder::new(data);
    let mut out = Vec::new();
    decoder
        .read_to_end(&mut out)
        .map_err(|e| ClientError::Decode(format!("Gzip decode failed: {e}")))?;
    Ok(out)
}

/// Multiplexes MTProto RPC requests and unpacks server responses over any `AbstractConnection`.
pub struct MtprotoClient<C: AbstractConnection> {
    pub connection: C,
    pub session: MtprotoSession,
    next_request_id: RequestId,
    pending_by_msg_id: HashMap<u64, RequestId>,
    pending_acks: Vec<u64>,
    outgoing_queue: VecDeque<Vec<u8>>,
}

impl<C: AbstractConnection> MtprotoClient<C> {
    pub fn new(connection: C, session: MtprotoSession) -> Self {
        Self {
            connection,
            session,
            next_request_id: 1,
            pending_by_msg_id: HashMap::new(),
            pending_acks: Vec::new(),
            outgoing_queue: VecDeque::new(),
        }
    }

    pub fn send_request(&mut self, tl_payload: &[u8], now_secs: f64) -> Result<RequestId, ClientError> {
        let req_id = self.next_request_id;
        self.next_request_id += 1;

        let packet = self.session.encrypt_message(tl_payload, true, now_secs);
        // Re-derive msg_id from decrypted/known monotonic state? Even better: peek from session or track:
        // Since `encrypt_message` called `next_msg_id`, let's decrypt-free or expose last_msg_id?
        // Actually `MtprotoSession::decrypt_message` is for incoming; let's check if we can compute or add a helper,
        // or decrypt our own packet? No, `part_for_msg_key(true)` vs `false`.
        // Wait: `next_msg_id` is deterministic if we pass `now_secs`, OR we can track `msg_id` by reading `tl_payload`?
        // Let's track `last_sent_msg_id`: wait, `MtprotoSession` doesn't expose `last_msg_id` directly, let's check!
        // We can expose `last_msg_id()` on `MtprotoSession` or return `(msg_id, packet)` from `MtprotoSession`.
        // For now, let's add a tiny `last_msg_id(&self) -> u64` getter on `MtprotoSession`!
        let msg_id = self.session.last_msg_id();
        self.pending_by_msg_id.insert(msg_id, req_id);
        self.connection.send_data(&packet)?;
        Ok(req_id)
    }

    pub fn send_ping(&mut self, ping_id: u64, now_secs: f64) -> Result<(), ClientError> {
        let mut payload = Vec::with_capacity(12);
        payload.extend_from_slice(&MTPC_PING.to_le_bytes());
        payload.extend_from_slice(&ping_id.to_le_bytes());
        let packet = self.session.encrypt_message(&payload, false, now_secs);
        self.connection.send_data(&packet)?;
        Ok(())
    }

    pub fn flush_acks(&mut self, now_secs: f64) -> Result<usize, ClientError> {
        if self.pending_acks.is_empty() {
            return Ok(0);
        }
        let acks: Vec<u64> = self.pending_acks.drain(..).collect();
        let count = acks.len();
        let mut payload = Vec::with_capacity(12 + count * 8);
        payload.extend_from_slice(&MTPC_MSGS_ACK.to_le_bytes());
        payload.extend_from_slice(&MTPC_VECTOR.to_le_bytes());
        payload.extend_from_slice(&(count as u32).to_le_bytes());
        for id in acks {
            payload.extend_from_slice(&id.to_le_bytes());
        }
        let packet = self.session.encrypt_message(&payload, false, now_secs);
        self.connection.send_data(&packet)?;
        Ok(count)
    }

    pub fn pending_count(&self) -> usize {
        self.pending_by_msg_id.len()
    }

    pub fn poll_events(&mut self) -> Result<Vec<ClientEvent>, ClientError> {
        let conn_events = self.connection.poll_events();
        let mut out = Vec::new();

        for ev in conn_events {
            match ev {
                ConnectionEvent::Connected => out.push(ClientEvent::Connected),
                ConnectionEvent::Disconnected => out.push(ClientEvent::Disconnected),
                ConnectionEvent::Error(code) => out.push(ClientEvent::ConnectionError(code)),
                ConnectionEvent::ReceivedPacket(bytes) => {
                    let dec = self
                        .session
                        .decrypt_message(&bytes)
                        .map_err(ClientError::Decrypt)?;
                    if dec.seq_no % 2 == 1 {
                        self.pending_acks.push(dec.msg_id);
                    }
                    self.handle_payload(&dec.payload, dec.msg_id, &mut out)?;
                }
            }
        }
        Ok(out)
    }

    fn handle_payload(
        &mut self,
        payload: &[u8],
        msg_id: u64,
        out: &mut Vec<ClientEvent>,
    ) -> Result<(), ClientError> {
        let mut off = 0;
        let ctor = read_u32_le(payload, &mut off)?;
        match ctor {
            MTPC_GZIP_PACKED => {
                let packed = read_tl_bytes(payload, &mut off)?;
                let unpacked = gunzip(&packed)?;
                self.handle_payload(&unpacked, msg_id, out)?;
            }
            MTPC_MSG_CONTAINER => {
                let count = read_u32_le(payload, &mut off)?;
                for _ in 0..count {
                    let inner_msg_id = read_u64_le(payload, &mut off)?;
                    let inner_seq_no = read_u32_le(payload, &mut off)?;
                    let bytes_len = read_u32_le(payload, &mut off)? as usize;
                    if off + bytes_len > payload.len() {
                        return Err(ClientError::Decode("Container item truncated".into()));
                    }
                    if inner_seq_no % 2 == 1 {
                        self.pending_acks.push(inner_msg_id);
                    }
                    let inner = &payload[off..off + bytes_len];
                    off += bytes_len;
                    self.handle_payload(inner, inner_msg_id, out)?;
                }
            }
            MTPC_RPC_RESULT => {
                let req_msg_id = read_u64_le(payload, &mut off)?;
                let rest = &payload[off..];
                let req_id = self.pending_by_msg_id.remove(&req_msg_id).unwrap_or(0);
                if rest.len() >= 4 {
                    let mut r_off = 0;
                    let inner_ctor = read_u32_le(rest, &mut r_off)?;
                    if inner_ctor == MTPC_GZIP_PACKED {
                        let packed = read_tl_bytes(rest, &mut r_off)?;
                        let unpacked = gunzip(&packed)?;
                        self.emit_rpc_body(req_id, &unpacked, out)?;
                    } else {
                        self.emit_rpc_body(req_id, rest, out)?;
                    }
                } else {
                    out.push(ClientEvent::RpcResponse {
                        request_id: req_id,
                        payload: rest.to_vec(),
                    });
                }
            }
            MTPC_PONG => {
                let _pong_msg_id = read_u64_le(payload, &mut off)?;
                let ping_id = read_u64_le(payload, &mut off)?;
                out.push(ClientEvent::Pong { ping_id, msg_id });
            }
            MTPC_NEW_SESSION_CREATED => {
                let first_msg_id = read_u64_le(payload, &mut off)?;
                let _unique_id = read_u64_le(payload, &mut off)?;
                let server_salt = read_u64_le(payload, &mut off)?;
                self.session.set_server_salt(server_salt);
                out.push(ClientEvent::NewSessionCreated {
                    first_msg_id,
                    server_salt,
                });
            }
            MTPC_BAD_SERVER_SALT => {
                let bad_msg_id = read_u64_le(payload, &mut off)?;
                let _bad_seq = read_u32_le(payload, &mut off)?;
                let _err_code = read_u32_le(payload, &mut off)?;
                let new_server_salt = read_u64_le(payload, &mut off)?;
                self.session.set_server_salt(new_server_salt);
                out.push(ClientEvent::BadServerSalt {
                    bad_msg_id,
                    new_server_salt,
                });
            }
            MTPC_MSGS_ACK => {
                // Server acknowledged our messages; ignore or log
            }
            _ => {
                out.push(ClientEvent::Update(payload.to_vec()));
            }
        }
        Ok(())
    }

    fn emit_rpc_body(
        &self,
        request_id: RequestId,
        body: &[u8],
        out: &mut Vec<ClientEvent>,
    ) -> Result<(), ClientError> {
        let mut off = 0;
        let ctor = read_u32_le(body, &mut off)?;
        if ctor == MTPC_RPC_ERROR {
            let code = read_u32_le(body, &mut off)? as i32;
            let msg_bytes = read_tl_bytes(body, &mut off)?;
            let err_str = String::from_utf8_lossy(&msg_bytes).to_string();
            out.push(ClientEvent::RpcError {
                request_id,
                error: MtpError::new(code, err_str),
            });
        } else {
            out.push(ClientEvent::RpcResponse {
                request_id,
                payload: body.to_vec(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crypto_aes_ige::aes_ige_encrypt;
    use mtproto_auth_key::AuthKey;
    use mtproto_connection::ConnectionState;
    use rand::RngCore;
    use sha2::{Digest, Sha256};

    struct MockConnection {
        sent: Vec<Vec<u8>>,
        incoming: VecDeque<ConnectionEvent>,
        connected: bool,
    }

    impl MockConnection {
        fn new() -> Self {
            Self {
                sent: Vec::new(),
                incoming: VecDeque::new(),
                connected: true,
            }
        }
    }

    impl AbstractConnection for MockConnection {
        fn connect_to_server(
            &mut self,
            _ip: &str,
            _port: u16,
            _secret: &[u8],
            _dc: i16,
            _files: bool,
        ) -> Result<(), ConnectionError> {
            self.connected = true;
            Ok(())
        }
        fn disconnect_from_server(&mut self) {
            self.connected = false;
        }
        fn send_data(&mut self, data: &[u8]) -> Result<(), ConnectionError> {
            self.sent.push(data.to_vec());
            Ok(())
        }
        fn poll_events(&mut self) -> Vec<ConnectionEvent> {
            self.incoming.drain(..).collect()
        }
        fn is_connected(&self) -> bool {
            self.connected
        }
        fn state(&self) -> ConnectionState {
            ConnectionState::Ready
        }
        fn ping_time_ms(&self) -> u64 {
            10
        }
        fn transport_name(&self) -> &'static str {
            "MOCK"
        }
        fn tag(&self) -> &'static str {
            "mock"
        }
    }

    fn server_encrypt(auth_key: &AuthKey, salt: u64, session_id: u64, msg_id: u64, seq_no: u32, payload: &[u8]) -> Vec<u8> {
        let raw_len = 32 + payload.len();
        let mut pad_len = 16 - (raw_len % 16);
        if pad_len < 12 {
            pad_len += 16;
        }
        let mut data = Vec::with_capacity(raw_len + pad_len);
        data.extend_from_slice(&salt.to_le_bytes());
        data.extend_from_slice(&session_id.to_le_bytes());
        data.extend_from_slice(&msg_id.to_le_bytes());
        data.extend_from_slice(&seq_no.to_le_bytes());
        data.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        data.extend_from_slice(payload);
        let mut pad = vec![0u8; pad_len];
        rand::thread_rng().fill_bytes(&mut pad);
        data.extend_from_slice(&pad);

        let mut hasher = Sha256::new();
        hasher.update(auth_key.part_for_msg_key(false));
        hasher.update(&data);
        let hash = hasher.finalize();
        let mut msg_key = [0u8; 16];
        msg_key.copy_from_slice(&hash[8..24]);

        let (aes_key, aes_iv) = auth_key.prepare_aes(&msg_key, false);
        aes_ige_encrypt(&mut data, &aes_key, &aes_iv).unwrap();

        let mut packet = Vec::new();
        packet.extend_from_slice(&auth_key.key_id().to_le_bytes());
        packet.extend_from_slice(&msg_key);
        packet.extend_from_slice(&data);
        packet
    }

    #[test]
    fn test_rpc_request_and_response_correlation() {
        let auth_key = AuthKey::from_key([0x33u8; 256]);
        let session = MtprotoSession::new(auth_key.clone(), 0x12345678);
        let session_id = session.session_id();
        let mut client = MtprotoClient::new(MockConnection::new(), session);

        let req_id = client.send_request(&[0xAA, 0xBB, 0xCC, 0xDD], 1000.0).unwrap();
        let req_msg_id = client.session.last_msg_id();
        assert_eq!(client.pending_count(), 1);

        // Build rpc_result containing [0x11, 0x22, 0x33, 0x44]
        let mut rpc_res = Vec::new();
        rpc_res.extend_from_slice(&MTPC_RPC_RESULT.to_le_bytes());
        rpc_res.extend_from_slice(&req_msg_id.to_le_bytes());
        rpc_res.extend_from_slice(&[0x11, 0x22, 0x33, 0x44]);

        let srv_pkt = server_encrypt(&auth_key, 0x12345678, session_id, 999999, 1, &rpc_res);
        client.connection.incoming.push_back(ConnectionEvent::ReceivedPacket(srv_pkt));

        let events = client.poll_events().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(
            events[0],
            ClientEvent::RpcResponse {
                request_id: req_id,
                payload: vec![0x11, 0x22, 0x33, 0x44]
            }
        );
        assert_eq!(client.pending_count(), 0);
        assert_eq!(client.flush_acks(1001.0).unwrap(), 1);
    }
}
