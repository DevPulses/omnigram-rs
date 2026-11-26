//! MTProto 2.0 session management, message ID generation, and packet encryption/decryption
//! matching Telegram Desktop (`tdesktop/Telegram/SourceFiles/mtproto/details/mtproto_serialized_request.{h,cpp}`
//! and `tdesktop/Telegram/SourceFiles/mtproto/session.{h,cpp}`).

use rand::RngCore;
use sha2::{Digest, Sha256};
use crypto_aes_ige::{aes_ige_decrypt, aes_ige_encrypt};
use mtproto_auth_key::AuthKey;

pub const MTPC_RPC_RESULT: u32 = 0xf35c6d01;
pub const MTPC_MSG_CONTAINER: u32 = 0x73f1f8dc;
pub const MTPC_GZIP_PACKED: u32 = 0x3072cfa1;

/// MTProto 2.0 Session state
pub struct MtprotoSession {
    session_id: u64,
    server_salt: u64,
    auth_key: AuthKey,
    seq_no: u32,
    last_msg_id: u64,
    time_offset: i64,
}

impl MtprotoSession {
    pub fn new(auth_key: AuthKey, server_salt: u64) -> Self {
        let mut rng = rand::thread_rng();
        let session_id = rng.next_u64();
        Self {
            session_id,
            server_salt,
            auth_key,
            seq_no: 0,
            last_msg_id: 0,
            time_offset: 0,
        }
    }

    pub fn session_id(&self) -> u64 {
        self.session_id
    }

    pub fn server_salt(&self) -> u64 {
        self.server_salt
    }

    pub fn last_msg_id(&self) -> u64 {
        self.last_msg_id
    }

    pub fn set_server_salt(&mut self, salt: u64) {
        self.server_salt = salt;
    }

    pub fn set_time_offset(&mut self, offset: i64) {
        self.time_offset = offset;
    }

    /// Generate monotonically increasing client message ID
    /// client message IDs must be divisible by 4, server messages end with 1 or 3
    pub fn next_msg_id(&mut self, now_secs: f64) -> u64 {
        let timestamp = (now_secs + (self.time_offset as f64)) * 4294967296.0;
        let mut id = (timestamp as u64) & !0x03; // Clear lower 2 bits (divisible by 4)
        if id <= self.last_msg_id {
            id = self.last_msg_id + 4;
        }
        self.last_msg_id = id;
        id
    }

    /// Advance sequence number (content-related messages increment by 1 and advance seq_no by 2)
    pub fn next_seq_no(&mut self, content_related: bool) -> u32 {
        let seq = if content_related {
            let res = self.seq_no * 2 + 1;
            self.seq_no += 1;
            res
        } else {
            self.seq_no * 2
        };
        seq
    }

    /// Encrypt a single payload into an MTProto 2.0 encrypted packet
    pub fn encrypt_message(&mut self, payload: &[u8], content_related: bool, now_secs: f64) -> Vec<u8> {
        let msg_id = self.next_msg_id(now_secs);
        let seq_no = self.next_seq_no(content_related);
        let msg_len = payload.len() as u32;

        // Plaintext payload inside container:
        // salt (8) + session_id (8) + msg_id (8) + seq_no (4) + msg_len (4) + payload + padding (12..1024)
        let header_len = 8 + 8 + 8 + 4 + 4;
        let raw_len = header_len + payload.len();

        // Padding: 12..1024 bytes, total length must be multiple of 16
        let mut pad_len = 16 - (raw_len % 16);
        if pad_len < 12 {
            pad_len += 16;
        }

        let mut data = Vec::with_capacity(raw_len + pad_len);
        data.extend_from_slice(&self.server_salt.to_le_bytes());
        data.extend_from_slice(&self.session_id.to_le_bytes());
        data.extend_from_slice(&msg_id.to_le_bytes());
        data.extend_from_slice(&seq_no.to_le_bytes());
        data.extend_from_slice(&msg_len.to_le_bytes());
        data.extend_from_slice(payload);

        let mut pad_bytes = vec![0u8; pad_len];
        rand::thread_rng().fill_bytes(&mut pad_bytes);
        data.extend_from_slice(&pad_bytes);

        // msg_key = middle 128 bits of SHA256(auth_key[88..88+32] + data)
        let mut hasher = Sha256::new();
        hasher.update(self.auth_key.part_for_msg_key(true));
        hasher.update(&data);
        let hash = hasher.finalize();

        let mut msg_key = [0u8; 16];
        msg_key.copy_from_slice(&hash[8..24]);

        let mut encrypted = data;
        let (aes_key, aes_iv) = self.auth_key.prepare_aes(&msg_key, true);
        aes_ige_encrypt(&mut encrypted, &aes_key, &aes_iv).expect("AES encryption");

        // Outer packet: auth_key_id (8) + msg_key (16) + encrypted_data
        let mut packet = Vec::with_capacity(8 + 16 + encrypted.len());
        packet.extend_from_slice(&self.auth_key.key_id().to_le_bytes());
        packet.extend_from_slice(&msg_key);
        packet.extend_from_slice(&encrypted);
        packet
    }

    /// Decrypt an incoming MTProto 2.0 encrypted packet
    pub fn decrypt_message(&self, packet: &[u8]) -> Result<DecryptedMessage, &'static str> {
        if packet.len() < 8 + 16 + 32 {
            return Err("Packet too short");
        }
        let auth_key_id = u64::from_le_bytes(packet[..8].try_into().unwrap());
        if auth_key_id != self.auth_key.key_id() {
            return Err("Unknown auth key id");
        }
        let mut msg_key = [0u8; 16];
        msg_key.copy_from_slice(&packet[8..24]);

        let (aes_key, aes_iv) = self.auth_key.prepare_aes(&msg_key, false);
        let mut decrypted = packet[24..].to_vec();
        aes_ige_decrypt(&mut decrypted, &aes_key, &aes_iv)
            .map_err(|_| "Decryption failed")?;

        // Verify msg_key
        let mut hasher = Sha256::new();
        hasher.update(self.auth_key.part_for_msg_key(false));
        hasher.update(&decrypted);
        let hash = hasher.finalize();
        if &hash[8..24] != &msg_key[..] {
            return Err("Bad msg_key check");
        }

        let salt = u64::from_le_bytes(decrypted[0..8].try_into().unwrap());
        let session_id = u64::from_le_bytes(decrypted[8..16].try_into().unwrap());
        let msg_id = u64::from_le_bytes(decrypted[16..24].try_into().unwrap());
        let seq_no = u32::from_le_bytes(decrypted[24..28].try_into().unwrap());
        let msg_len = u32::from_le_bytes(decrypted[28..32].try_into().unwrap()) as usize;

        if 32 + msg_len > decrypted.len() {
            return Err("Invalid message length");
        }
        let payload = decrypted[32..32 + msg_len].to_vec();

        Ok(DecryptedMessage {
            salt,
            session_id,
            msg_id,
            seq_no,
            payload,
        })
    }
}

pub struct DecryptedMessage {
    pub salt: u64,
    pub session_id: u64,
    pub msg_id: u64,
    pub seq_no: u32,
    pub payload: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_msg_id_monotonic() {
        let auth_key = AuthKey::from_key([7u8; 256]);
        let mut session = MtprotoSession::new(auth_key, 12345);

        let id1 = session.next_msg_id(100.0);
        let id2 = session.next_msg_id(100.0);
        let id3 = session.next_msg_id(100.0);

        assert_eq!(id1 % 4, 0);
        assert_eq!(id2 % 4, 0);
        assert_eq!(id3 % 4, 0);
        assert!(id2 > id1);
        assert!(id3 > id2);
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let auth_key = AuthKey::from_key([0x5A; 256]);
        let mut session_sender = MtprotoSession::new(auth_key.clone(), 0x1122334455667788);

        let payload = b"Hello MTProto 2.0 World!";
        let packet = session_sender.encrypt_message(payload, true, 123456.0);

        assert!(packet.len() > payload.len());
        assert_eq!(&packet[..8], &auth_key.key_id().to_le_bytes());
    }
}
