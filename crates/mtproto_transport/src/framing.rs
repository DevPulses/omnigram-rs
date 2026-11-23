//! MTProto TCP framing protocols:
//! - Abridged (0xef / Version0): packet size in 4-byte words (1 byte or 0x7f + 3 bytes).
//! - Intermediate (0xee / Version1)
//! - Padded Intermediate (0xdd / VersionD): length in bytes + 4, random padding 0..15 bytes.

pub trait Framing {
    fn id(&self) -> u32;
    fn encode_packet(&self, payload: &[u8]) -> Vec<u8>;
    /// Returns `Ok(Some((packet_payload, consumed_bytes)))` or `Ok(None)` if more data needed.
    fn decode_packet<'a>(&self, buf: &'a [u8]) -> Result<Option<(&'a [u8], usize)>, FramingError>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum FramingError {
    InvalidLength,
    PacketTooLarge,
}

pub const PACKET_SIZE_MAX: usize = 0x01000000 * 4; // 64 MB

/// Abridged protocol (TcpConnection::Protocol::Version0)
/// ID: 0xEFEFEFEF
#[derive(Default, Clone, Copy)]
pub struct AbridgedFraming;

impl Framing for AbridgedFraming {
    fn id(&self) -> u32 {
        0xEFEFEFEF
    }

    fn encode_packet(&self, payload: &[u8]) -> Vec<u8> {
        assert_eq!(payload.len() % 4, 0, "Payload must be 4-byte aligned");
        let ints_size = payload.len() / 4;
        let mut out = Vec::new();
        if ints_size < 0x7F {
            out.push(ints_size as u8);
        } else {
            out.push(0x7F);
            out.push((ints_size & 0xFF) as u8);
            out.push(((ints_size >> 8) & 0xFF) as u8);
            out.push(((ints_size >> 16) & 0xFF) as u8);
        }
        out.extend_from_slice(payload);
        out
    }

    fn decode_packet<'a>(&self, buf: &'a [u8]) -> Result<Option<(&'a [u8], usize)>, FramingError> {
        if buf.is_empty() {
            return Ok(None);
        }
        let first = buf[0];
        let (ints, header_len) = if first == 0x7F {
            if buf.len() < 4 {
                return Ok(None);
            }
            let ints = (buf[1] as usize)
                | ((buf[2] as usize) << 8)
                | ((buf[3] as usize) << 16);
            if ints < 0x7F {
                return Err(FramingError::InvalidLength);
            }
            (ints, 4)
        } else if first > 0 && first < 0x7F {
            (first as usize, 1)
        } else {
            return Err(FramingError::InvalidLength);
        };

        let payload_len = ints * 4;
        let total_len = header_len + payload_len;
        if total_len > PACKET_SIZE_MAX {
            return Err(FramingError::PacketTooLarge);
        }
        if buf.len() < total_len {
            return Ok(None);
        }
        Ok(Some((&buf[header_len..total_len], total_len)))
    }
}

/// Intermediate protocol (0xEEEEEEEE)
#[derive(Default, Clone, Copy)]
pub struct IntermediateFraming;

impl Framing for IntermediateFraming {
    fn id(&self) -> u32 {
        0xEEEEEEEE
    }

    fn encode_packet(&self, payload: &[u8]) -> Vec<u8> {
        let len = payload.len() as u32;
        let mut out = Vec::with_capacity(4 + payload.len());
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(payload);
        out
    }

    fn decode_packet<'a>(&self, buf: &'a [u8]) -> Result<Option<(&'a [u8], usize)>, FramingError> {
        if buf.len() < 4 {
            return Ok(None);
        }
        let payload_len = u32::from_le_bytes(buf[0..4].try_into().unwrap()) as usize;
        let total_len = 4 + payload_len;
        if total_len > PACKET_SIZE_MAX {
            return Err(FramingError::PacketTooLarge);
        }
        if buf.len() < total_len {
            return Ok(None);
        }
        Ok(Some((&buf[4..total_len], total_len)))
    }
}

/// Padded Intermediate protocol (TcpConnection::Protocol::VersionD)
/// ID: 0xDDDDDDDD
#[derive(Default, Clone, Copy)]
pub struct PaddedFraming;

impl Framing for PaddedFraming {
    fn id(&self) -> u32 {
        0xDDDDDDDD
    }

    fn encode_packet(&self, payload: &[u8]) -> Vec<u8> {
        let padding_len = (rand::random::<u32>() & 0x0F) as usize;
        let bytes_size = (payload.len() + padding_len) as u32;
        let mut out = Vec::with_capacity(4 + payload.len() + padding_len);
        out.extend_from_slice(&bytes_size.to_le_bytes());
        out.extend_from_slice(payload);
        for _ in 0..padding_len {
            out.push(rand::random::<u8>());
        }
        out
    }

    fn decode_packet<'a>(&self, buf: &'a [u8]) -> Result<Option<(&'a [u8], usize)>, FramingError> {
        if buf.len() < 4 {
            return Ok(None);
        }
        let bytes_size = u32::from_le_bytes(buf[0..4].try_into().unwrap()) as usize;
        let total_len = 4 + bytes_size;
        if total_len < 8 || total_len >= PACKET_SIZE_MAX {
            return Err(FramingError::InvalidLength);
        }
        if buf.len() < total_len {
            return Ok(None);
        }
        Ok(Some((&buf[4..total_len], total_len)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abridged_framing_small() {
        let framing = AbridgedFraming;
        let payload = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let encoded = framing.encode_packet(&payload);
        assert_eq!(encoded[0], 2); // 8 bytes / 4 = 2 ints
        assert_eq!(&encoded[1..], &payload[..]);

        let decoded = framing.decode_packet(&encoded).unwrap().unwrap();
        assert_eq!(decoded.0, &payload[..]);
        assert_eq!(decoded.1, encoded.len());
    }

    #[test]
    fn test_abridged_framing_large() {
        let framing = AbridgedFraming;
        let payload = vec![0xAA; 1000 * 4]; // 1000 ints > 0x7f
        let encoded = framing.encode_packet(&payload);
        assert_eq!(encoded[0], 0x7F);
        let ints = (encoded[1] as usize) | ((encoded[2] as usize) << 8) | ((encoded[3] as usize) << 16);
        assert_eq!(ints, 1000);

        let decoded = framing.decode_packet(&encoded).unwrap().unwrap();
        assert_eq!(decoded.0, &payload[..]);
        assert_eq!(decoded.1, encoded.len());
    }

    #[test]
    fn test_padded_framing() {
        let framing = PaddedFraming;
        let payload = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let encoded = framing.encode_packet(&payload);
        let decoded = framing.decode_packet(&encoded).unwrap().unwrap();
        assert!(decoded.0.starts_with(&payload));
        assert_eq!(decoded.1, encoded.len());
    }
}
