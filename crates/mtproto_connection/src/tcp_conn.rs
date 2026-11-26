//! MTProto TCP connection and obfuscated framing codec.
//!
//! Source: `tdesktop/Telegram/SourceFiles/mtproto/connection_tcp.{h,cpp}`.

use std::collections::VecDeque;
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};

use mtproto_transport::{
    AbridgedFraming, Framing, IntermediateFraming, ObfuscatedStream, PaddedFraming, ProtocolVersion,
};

use crate::abstract_conn::{
    protocol_dc_debug_id, AbstractConnection, ConnectionError, ConnectionEvent, ConnectionState,
    ERROR_CODE_OTHER, FULL_CONNECTION_TIMEOUT_MS,
};

/// Pure byte-stream codec combining `ObfuscatedStream` (AES-CTR) and `Framing`.
/// Used by `TcpConnection` for both real sockets and deterministic unit tests.
pub struct TcpTransportCodec {
    stream: ObfuscatedStream,
    recv_buf: Vec<u8>,
}

impl TcpTransportCodec {
    /// Creates a new codec and returns the 64-byte obfuscated handshake prefix
    /// to be sent immediately upon TCP connection.
    pub fn new(protocol: ProtocolVersion, dc_id: i16, secret: &[u8]) -> (Self, [u8; 64]) {
        let (stream, handshake) = ObfuscatedStream::client_handshake(protocol, dc_id, secret);
        (
            Self {
                stream,
                recv_buf: Vec::new(),
            },
            handshake,
        )
    }

    /// Encodes and obfuscates an MTProto packet for transmission over TCP.
    pub fn encode_packet(&mut self, payload: &[u8]) -> Vec<u8> {
        let mut framed = match self.stream.protocol {
            ProtocolVersion::Version0 => AbridgedFraming.encode_packet(payload),
            ProtocolVersion::Version1 => IntermediateFraming.encode_packet(payload),
            ProtocolVersion::VersionD => PaddedFraming.encode_packet(payload),
        };
        self.stream.encrypt(&mut framed);
        framed
    }

    /// Feeds incoming encrypted TCP bytes and extracts any complete MTProto packets.
    pub fn feed_incoming(&mut self, encrypted_bytes: &[u8]) -> Result<Vec<Vec<u8>>, ConnectionError> {
        let mut chunk = encrypted_bytes.to_vec();
        self.stream.decrypt(&mut chunk);
        self.recv_buf.extend_from_slice(&chunk);

        let mut packets = Vec::new();
        loop {
            let decoded = match self.stream.protocol {
                ProtocolVersion::Version0 => AbridgedFraming.decode_packet(&self.recv_buf),
                ProtocolVersion::Version1 => IntermediateFraming.decode_packet(&self.recv_buf),
                ProtocolVersion::VersionD => PaddedFraming.decode_packet(&self.recv_buf),
            };
            match decoded {
                Ok(Some((payload, consumed))) => {
                    packets.push(payload.to_vec());
                    self.recv_buf.drain(..consumed);
                }
                Ok(None) => break,
                Err(err) => {
                    return Err(ConnectionError::Protocol(format!("Framing error: {err:?}")));
                }
            }
        }
        Ok(packets)
    }

    pub fn protocol(&self) -> ProtocolVersion {
        self.stream.protocol
    }

    pub fn debug_postfix(&self) -> &'static str {
        match self.stream.protocol {
            ProtocolVersion::Version0 => "",
            ProtocolVersion::Version1 => "_o",
            ProtocolVersion::VersionD => "_d",
        }
    }
}

/// TCP connection implementation matching `MTP::details::TcpConnection`.
pub struct TcpConnection {
    socket: Option<TcpStream>,
    codec: Option<TcpTransportCodec>,
    state: ConnectionState,
    events: VecDeque<ConnectionEvent>,
    address: String,
    port: u16,
    protocol_dc_id: i16,
    ping_time_ms: u64,
    connect_started_at: Option<Instant>,
}

impl Default for TcpConnection {
    fn default() -> Self {
        Self::new()
    }
}

impl TcpConnection {
    pub fn new() -> Self {
        Self {
            socket: None,
            codec: None,
            state: ConnectionState::Waiting,
            events: VecDeque::new(),
            address: String::new(),
            port: 0,
            protocol_dc_id: 0,
            ping_time_ms: 0,
            connect_started_at: None,
        }
    }

    fn select_protocol(protocol_secret: &[u8]) -> ProtocolVersion {
        if protocol_secret.len() >= 17 && protocol_secret[0] == 0xDD {
            ProtocolVersion::VersionD
        } else if !protocol_secret.is_empty() {
            ProtocolVersion::Version1
        } else {
            ProtocolVersion::Version0
        }
    }

    pub fn debug_state(&self) -> i32 {
        match self.state {
            ConnectionState::Waiting => 0,
            ConnectionState::Ready => 1,
            ConnectionState::Finished => 2,
        }
    }

    pub fn address(&self) -> &str {
        &self.address
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn debug_id(&self) -> String {
        protocol_dc_debug_id(self.protocol_dc_id)
    }
}

impl AbstractConnection for TcpConnection {
    fn connect_to_server(
        &mut self,
        ip: &str,
        port: u16,
        protocol_secret: &[u8],
        protocol_dc_id: i16,
        _protocol_for_files: bool,
    ) -> Result<(), ConnectionError> {
        self.disconnect_from_server();

        self.address = ip.to_string();
        self.port = port;
        self.protocol_dc_id = protocol_dc_id;
        let started = Instant::now();
        self.connect_started_at = Some(started);

        let addr: SocketAddr = format!("{ip}:{port}")
            .parse()
            .map_err(|e| ConnectionError::Io(format!("Invalid address {ip}:{port}: {e}")))?;

        let mut stream = TcpStream::connect_timeout(
            &addr,
            Duration::from_millis(FULL_CONNECTION_TIMEOUT_MS),
        )
        .map_err(|e| {
            self.state = ConnectionState::Finished;
            self.events.push_back(ConnectionEvent::Error(ERROR_CODE_OTHER));
            ConnectionError::Io(e.to_string())
        })?;

        stream
            .set_nonblocking(true)
            .map_err(|e| ConnectionError::Io(e.to_string()))?;

        let protocol = Self::select_protocol(protocol_secret);
        let (codec, handshake) = TcpTransportCodec::new(protocol, protocol_dc_id, protocol_secret);

        stream.write_all(&handshake).map_err(|e| {
            self.state = ConnectionState::Finished;
            ConnectionError::Io(e.to_string())
        })?;

        self.ping_time_ms = started.elapsed().as_millis() as u64;
        self.codec = Some(codec);
        self.socket = Some(stream);
        self.state = ConnectionState::Ready;
        self.events.push_back(ConnectionEvent::Connected);
        Ok(())
    }

    fn disconnect_from_server(&mut self) {
        if self.socket.take().is_some() {
            self.events.push_back(ConnectionEvent::Disconnected);
        }
        self.codec = None;
        self.state = ConnectionState::Finished;
    }

    fn send_data(&mut self, data: &[u8]) -> Result<(), ConnectionError> {
        if self.state != ConnectionState::Ready {
            return Err(ConnectionError::NotConnected);
        }
        let codec = self.codec.as_mut().ok_or(ConnectionError::NotConnected)?;
        let socket = self.socket.as_mut().ok_or(ConnectionError::NotConnected)?;

        let wire = codec.encode_packet(data);
        socket.write_all(&wire).map_err(|e| {
            self.state = ConnectionState::Finished;
            self.events.push_back(ConnectionEvent::Error(ERROR_CODE_OTHER));
            ConnectionError::Io(e.to_string())
        })?;
        Ok(())
    }

    fn poll_events(&mut self) -> Vec<ConnectionEvent> {
        if self.state == ConnectionState::Ready {
            if let (Some(socket), Some(codec)) = (self.socket.as_mut(), self.codec.as_mut()) {
                let mut buf = [0u8; 16 * 1024];
                loop {
                    match socket.read(&mut buf) {
                        Ok(0) => {
                            self.state = ConnectionState::Finished;
                            self.socket = None;
                            self.events.push_back(ConnectionEvent::Disconnected);
                            break;
                        }
                        Ok(n) => match codec.feed_incoming(&buf[..n]) {
                            Ok(packets) => {
                                for pkt in packets {
                                    self.events.push_back(ConnectionEvent::ReceivedPacket(pkt));
                                }
                            }
                            Err(_) => {
                                self.state = ConnectionState::Finished;
                                self.socket = None;
                                self.events.push_back(ConnectionEvent::Error(ERROR_CODE_OTHER));
                                break;
                            }
                        },
                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => break,
                        Err(_) => {
                            self.state = ConnectionState::Finished;
                            self.socket = None;
                            self.events.push_back(ConnectionEvent::Error(ERROR_CODE_OTHER));
                            break;
                        }
                    }
                }
            }
        }

        self.events.drain(..).collect()
    }

    fn is_connected(&self) -> bool {
        self.state == ConnectionState::Ready
    }

    fn state(&self) -> ConnectionState {
        self.state
    }

    fn ping_time_ms(&self) -> u64 {
        self.ping_time_ms
    }

    fn transport_name(&self) -> &'static str {
        "TCP"
    }

    fn tag(&self) -> &'static str {
        match self.codec.as_ref().map(|c| c.protocol()) {
            Some(ProtocolVersion::Version1) => "tcp_o",
            Some(ProtocolVersion::VersionD) => "tcp_d",
            _ => "tcp",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abstract_conn::{
        parse_not_secure_response, prepare_not_secure_packet, prepare_secure_packet,
        TEST_MODE_DC_ID_SHIFT,
    };
    use mtproto_transport::Ctr128;

    /// Builds a matching server-side `ObfuscatedStream` from a client's 64-byte handshake
    /// to verify end-to-end TCP obfuscation + framing without needing a live Telegram server.
    fn server_stream_from_handshake(handshake: &[u8; 64]) -> (Ctr128, Ctr128, ProtocolVersion) {
        let mut dec_key = [0u8; 32];
        dec_key.copy_from_slice(&handshake[8..40]);
        let mut dec_iv = [0u8; 16];
        dec_iv.copy_from_slice(&handshake[40..56]);

        let mut reversed = [0u8; 48];
        reversed.copy_from_slice(&handshake[8..56]);
        reversed.reverse();

        let mut enc_key = [0u8; 32];
        enc_key.copy_from_slice(&reversed[0..32]);
        let mut enc_iv = [0u8; 16];
        enc_iv.copy_from_slice(&reversed[32..48]);

        let mut server_recv_ctr = Ctr128::new(&dec_key, &dec_iv);
        let server_send_ctr = Ctr128::new(&enc_key, &enc_iv);

        let mut decrypted_header = *handshake;
        server_recv_ctr.apply_keystream(&mut decrypted_header);

        let proto_id = u32::from_le_bytes(decrypted_header[56..60].try_into().unwrap());
        let proto = match proto_id {
            0xEFEFEFEF => ProtocolVersion::Version0,
            0xEEEEEEEE => ProtocolVersion::Version1,
            0xDDDDDDDD => ProtocolVersion::VersionD,
            other => panic!("Unexpected protocol id: {other:#x}"),
        };
        (server_recv_ctr, server_send_ctr, proto)
    }

    #[test]
    fn test_codec_roundtrip_all_protocols() {
        for proto in [
            ProtocolVersion::Version0,
            ProtocolVersion::Version1,
            ProtocolVersion::VersionD,
        ] {
            let (mut client, handshake) = TcpTransportCodec::new(proto, 2, &[]);
            let (mut srv_recv, mut srv_send, detected_proto) =
                server_stream_from_handshake(&handshake);
            assert_eq!(detected_proto, proto);

            let msg = prepare_not_secure_packet(4, b"req_pq_multi_payload");
            let mut wire = client.encode_packet(&msg);

            // Server decrypts and re-encrypts response
            srv_recv.apply_keystream(&mut wire);
            let decoded = match proto {
                ProtocolVersion::Version0 => AbridgedFraming.decode_packet(&wire).unwrap().unwrap().0,
                ProtocolVersion::Version1 => IntermediateFraming.decode_packet(&wire).unwrap().unwrap().0,
                ProtocolVersion::VersionD => PaddedFraming.decode_packet(&wire).unwrap().unwrap().0,
            };
            assert_eq!(decoded, msg.as_slice());

            let reply = prepare_not_secure_packet(5, b"resPQ_123456");
            let mut reply_wire = match proto {
                ProtocolVersion::Version0 => AbridgedFraming.encode_packet(&reply),
                ProtocolVersion::Version1 => IntermediateFraming.encode_packet(&reply),
                ProtocolVersion::VersionD => PaddedFraming.encode_packet(&reply),
            };
            srv_send.apply_keystream(&mut reply_wire);

            let received = client.feed_incoming(&reply_wire).unwrap();
            assert_eq!(received.len(), 1);
            let (parsed_id, body) = parse_not_secure_response(&received[0]).unwrap();
            assert_eq!(parsed_id, 5);
            assert_eq!(body, b"resPQ_123456");
        }
    }

    #[test]
    fn test_prepare_secure_and_debug_ids() {
        let pkt = prepare_secure_packet(0x0102030405060708, [0xAA; 16], &[1, 2, 3, 4]);
        assert_eq!(pkt.len(), 8 + 16 + 4);
        assert_eq!(&pkt[0..8], &0x0102030405060708u64.to_le_bytes());

        assert_eq!(protocol_dc_debug_id(2), "prod_2");
        assert_eq!(protocol_dc_debug_id(TEST_MODE_DC_ID_SHIFT + 2), "test_2");
        assert_eq!(protocol_dc_debug_id(0), "");
    }

    #[test]
    fn test_tcp_connection_loopback() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        let handle = std::thread::spawn(move || {
            let (mut sock, _) = listener.accept().unwrap();
            let mut hs = [0u8; 64];
            sock.read_exact(&mut hs).unwrap();
            let (mut srv_recv, mut srv_send, _) = server_stream_from_handshake(&hs);

            let mut buf = [0u8; 512];
            let n = sock.read(&mut buf).unwrap();
            srv_recv.apply_keystream(&mut buf[..n]);
            let (payload, _) = AbridgedFraming.decode_packet(&buf[..n]).unwrap().unwrap();

            let mut reply = AbridgedFraming.encode_packet(payload);
            srv_send.apply_keystream(&mut reply);
            sock.write_all(&reply).unwrap();
        });

        let mut conn = TcpConnection::new();
        conn.connect_to_server(&addr.ip().to_string(), addr.port(), &[], 2, false)
            .unwrap();
        assert!(conn.is_connected());
        assert_eq!(conn.debug_id(), "prod_2");

        let pkt = prepare_not_secure_packet(8, b"ping_123");
        conn.send_data(&pkt).unwrap();

        // Wait briefly for loopback echo
        let mut got_packet = None;
        for _ in 0..50 {
            for ev in conn.poll_events() {
                if let ConnectionEvent::ReceivedPacket(p) = ev {
                    got_packet = Some(p);
                }
            }
            if got_packet.is_some() {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }

        assert_eq!(got_packet.unwrap(), pkt);
        conn.disconnect_from_server();
        assert!(!conn.is_connected());
        handle.join().unwrap();
    }
}
