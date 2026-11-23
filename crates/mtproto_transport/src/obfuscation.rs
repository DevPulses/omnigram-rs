//! MTProto TCP obfuscation layer (64-byte handshake + bidirectional AES-CTR encryption).
//!
//! Matches `TcpConnection::prepareConnectionStartPrefix` from Telegram Desktop.

use crate::ctr128::Ctr128;
use sha2::{Digest, Sha256};

pub const HANDSHAKE_SIZE: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtocolVersion {
    Version0, // Abridged: 0xEFEFEFEF
    Version1, // Intermediate / Obf: 0xEEEEEEEE
    VersionD, // Padded: 0xDDDDDDDD
}

impl ProtocolVersion {
    pub fn id(&self) -> u32 {
        match self {
            Self::Version0 => 0xEFEFEFEF,
            Self::Version1 => 0xEEEEEEEE,
            Self::VersionD => 0xDDDDDDDD,
        }
    }
}

pub struct ObfuscatedStream {
    pub send_ctr: Ctr128,
    pub recv_ctr: Ctr128,
    pub protocol: ProtocolVersion,
    pub dc_id: i16,
}

/// Checks if a 64-byte start nonce is valid.
/// Matches `TcpSocket::isGoodStartNonce` in tdesktop:
/// ```cpp
/// 	const auto zero = gsl::to_integer<uchar>(nonce.front());
/// 	const auto first = qFromUnaligned<uint32>(bytes);
/// 	const auto second = qFromUnaligned<uint32>(bytes + sizeof(uint32));
/// 	const auto reserved01 = 0x000000EFU;
/// 	const auto reserved11 = 0x44414548U;
/// 	const auto reserved12 = 0x54534F50U;
/// 	const auto reserved13 = 0x20544547U;
/// 	const auto reserved14 = 0xEEEEEEEEU;
/// 	const auto reserved15 = 0xDDDDDDDDU;
/// 	const auto reserved16 = 0x02010316U;
/// 	const auto reserved21 = 0x00000000U;
/// 	return (zero != reserved01)
/// 		&& (first != reserved11)
/// 		&& (first != reserved12)
/// 		&& (first != reserved13)
/// 		&& (first != reserved14)
/// 		&& (first != reserved15)
/// 		&& (first != reserved16)
/// 		&& (second != reserved21);
/// ```
pub fn is_good_start_nonce(nonce: &[u8; 64]) -> bool {
    let zero = nonce[0];
    let first = u32::from_le_bytes(nonce[0..4].try_into().unwrap());
    let second = u32::from_le_bytes(nonce[4..8].try_into().unwrap());

    let reserved01 = 0xEFu8;
    let reserved11 = 0x44414548u32; // "HEAD"
    let reserved12 = 0x54534F50u32; // "POST"
    let reserved13 = 0x20544547u32; // "GET "
    let reserved14 = 0xEEEEEEEEu32;
    let reserved15 = 0xDDDDDDDDu32;
    let reserved16 = 0x02010316u32;
    let reserved21 = 0x00000000u32;

    zero != reserved01
        && first != reserved11
        && first != reserved12
        && first != reserved13
        && first != reserved14
        && first != reserved15
        && first != reserved16
        && second != reserved21
}

impl ObfuscatedStream {
    /// Creates the client handshake 64-byte prefix and initializes bidirectional CTR ciphers.
    /// `secret`: optional secret for proxy / obfuscation (empty for direct, 16 bytes for MTProto proxy).
    pub fn client_handshake(
        protocol: ProtocolVersion,
        dc_id: i16,
        secret: &[u8],
    ) -> (Self, [u8; HANDSHAKE_SIZE]) {
        let mut nonce = [0u8; HANDSHAKE_SIZE];
        loop {
            rand::Rng::fill(&mut rand::thread_rng(), &mut nonce[..]);
            if is_good_start_nonce(&nonce) {
                break;
            }
        }

        let (stream, handshake) = Self::from_nonce(nonce, protocol, dc_id, secret);
        (stream, handshake)
    }

    /// Derives ciphers and handshake bytes from a specific 64-byte nonce (useful for tests / deterministic vectors).
    pub fn from_nonce(
        mut nonce: [u8; HANDSHAKE_SIZE],
        protocol: ProtocolVersion,
        dc_id: i16,
        secret: &[u8],
    ) -> (Self, [u8; HANDSHAKE_SIZE]) {
        // nonce layout:
        // [0..8]: random
        // [8..40]: send key source (32 bytes)
        // [40..56]: send ivec (16 bytes)
        // [56..60]: protocol id (4 bytes)
        // [60..64]: dc id (2 bytes) + padding (2 bytes)

        let mut send_key = [0u8; 32];
        let mut send_ivec = [0u8; 16];
        prepare_key(&mut send_key, &nonce[8..40], secret);
        send_ivec.copy_from_slice(&nonce[40..56]);

        // prepare decryption key/iv from reversed 48 bytes
        let mut reversed = [0u8; 48];
        reversed.copy_from_slice(&nonce[8..56]);
        reversed.reverse();

        let mut recv_key = [0u8; 32];
        let mut recv_ivec = [0u8; 16];
        prepare_key(&mut recv_key, &reversed[0..32], secret);
        recv_ivec.copy_from_slice(&reversed[32..48]);

        // write protocol id and dc id into nonce[56..64]
        nonce[56..60].copy_from_slice(&protocol.id().to_le_bytes());
        nonce[60..62].copy_from_slice(&dc_id.to_le_bytes());

        let mut send_ctr = Ctr128::new(&send_key, &send_ivec);
        let recv_ctr = Ctr128::new(&recv_key, &recv_ivec);

        let mut handshake = [0u8; HANDSHAKE_SIZE];
        handshake[0..56].copy_from_slice(&nonce[0..56]);

        // In tdesktop:
        // aesCtrEncrypt(nonce, _sendKey, &_sendState);
        // bytes::copy(buffer.subspan(56), nonce.subspan(56));
        send_ctr.apply_keystream(&mut nonce);
        handshake[56..64].copy_from_slice(&nonce[56..64]);

        (
            Self {
                send_ctr,
                recv_ctr,
                protocol,
                dc_id,
            },
            handshake,
        )
    }

    /// Encrypts outgoing data stream.
    pub fn encrypt(&mut self, data: &mut [u8]) {
        self.send_ctr.apply_keystream(data);
    }

    /// Decrypts incoming data stream.
    pub fn decrypt(&mut self, data: &mut [u8]) {
        self.recv_ctr.apply_keystream(data);
    }
}

fn prepare_key(key: &mut [u8; 32], source: &[u8], secret: &[u8]) {
    if secret.is_empty() {
        key.copy_from_slice(source);
    } else {
        let mut hasher = Sha256::new();
        hasher.update(source);
        hasher.update(secret);
        let res = hasher.finalize();
        key.copy_from_slice(&res);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_good_start_nonce() {
        let mut nonce = [1u8; 64];
        nonce[0] = 0xEF;
        assert!(!is_good_start_nonce(&nonce));

        nonce[0] = 0xAA;
        assert!(is_good_start_nonce(&nonce));

        // Test second == 0
        nonce[4..8].copy_from_slice(&0u32.to_le_bytes());
        assert!(!is_good_start_nonce(&nonce));
    }

    #[test]
    fn test_handshake_generation() {
        let (mut stream, handshake) = ObfuscatedStream::client_handshake(
            ProtocolVersion::Version0,
            2,
            &[],
        );
        assert_eq!(handshake.len(), 64);
        assert_ne!(handshake, [0u8; 64]);

        // Encrypt some packet
        let mut msg = b"ping_message".to_vec();
        stream.encrypt(&mut msg);
        assert_ne!(msg, b"ping_message");
    }
}

#[cfg(test)]
mod server_side_tests {
#[test]
fn dbg_stream() {
    let secret: &[u8] = b"0123456789abcdef";
    let nonce = [7u8; 64];
    let (mut client, handshake) = ObfuscatedStream::from_nonce(nonce, ProtocolVersion::Version1, 2, secret);
    let framing = AbridgedFraming;
    let payload = vec![0x11u8; 16];
    let mut packet = framing.encode_packet(&payload);
    client.encrypt(&mut packet);
    eprintln!("packet: {}", hex::encode(&packet));
    let mut recv_key = [0u8; 32];
    prepare_key(&mut recv_key, &handshake[8..40], secret);
    eprintln!("handshake: {}", hex::encode(handshake));
    let mut recv_ivec = [0u8; 16];
    recv_ivec.copy_from_slice(&handshake[40..56]);
    let mut server = Ctr128::new(&recv_key, &recv_ivec);
    let mut skip = [0u8; 64];
    server.apply_keystream(&mut skip);
    server.apply_keystream(&mut packet);
    eprintln!("dec: {}", hex::encode(&packet));
}

    use super::*;
    use crate::framing::{Framing, AbridgedFraming};

    /// Сквозной оракул: клиентская обфускация → «сервер» восстанавливает
    /// приёмный CTR из хендшейка и расшифровывает пакеты.
    #[test]
    fn server_recovers_stream_from_handshake() {
        let secret: &[u8] = b"0123456789abcdef"; // 16 байт → Version1 (sha256-ключи)
        let (mut client, handshake) = ObfuscatedStream::client_handshake(
            ProtocolVersion::Version1,
            2,
            secret,
        );

        // Сервер: приёмный ключ = клиентский send — из сырых байт
        // handshake[8..56] (не шифруются). Реверс — ключ сервера НА
        // отправку (клиентский receive).
        let mut recv_key = [0u8; 32];
        prepare_key(&mut recv_key, &handshake[8..40], secret);
        let mut recv_ivec = [0u8; 16];
        recv_ivec.copy_from_slice(&handshake[40..56]);
        let mut server = Ctr128::new(&recv_key, &recv_ivec);

        // Сервер прогоняет CTR через хендшейк (56 байт идут нешифрованными,
        // но состояние ключевого потока синхронно с клиентом).
        let mut skip = [0u8; 64];
        server.apply_keystream(&mut skip);

        // Клиент шифрует abridged-пакет.
        let framing = AbridgedFraming;
        let payload = vec![0x11u8; 16];
        let mut packet = framing.encode_packet(&payload);
        client.encrypt(&mut packet);

        // Сервер расшифровывает и разбирает.
        server.apply_keystream(&mut packet);
        let (got, _consumed) = framing.decode_packet(&packet).unwrap().unwrap();
        assert_eq!(got, &payload[..]);
    }
}
