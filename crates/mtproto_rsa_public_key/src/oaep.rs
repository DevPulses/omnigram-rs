//! RSAES-OAEP encoding, RFC 8017 section 7.1.1, with the parameter
//! choices of the OpenSSL `RSA_PKCS1_OAEP_PADDING` defaults: SHA-1 for
//! the label hash and for MGF1, and an empty label. That is the padding
//! behind `RSAPublicKey::Private::encryptOAEPpadding`.

use sha1::{Digest, Sha1};

/// The SHA-1 output size, also the seed length of the encoding.
pub(crate) const HASH_LEN: usize = 20;

/// The largest message a modulus of `k` bytes carries: `k - 2*hLen - 2`,
/// the limit `RSA_padding_add_PKCS1_OAEP_mgf1` enforces.
pub(crate) fn max_message_len(k: usize) -> Option<usize> {
    k.checked_sub(2 * HASH_LEN + 2)
}

/// The encoded message `EM = 0x00 || maskedSeed || maskedDB` of exactly
/// `k` bytes, ready for the raw RSA operation.
pub(crate) fn encode(k: usize, message: &[u8], seed: &[u8; HASH_LEN]) -> Option<Vec<u8>> {
    let max = max_message_len(k)?;
    if message.len() > max {
        return None;
    }
    let db_len = k - HASH_LEN - 1;
    let mut db = Vec::with_capacity(db_len);
    db.extend_from_slice(&Sha1::digest(b""));
    db.resize(db_len - message.len() - 1, 0);
    db.push(0x01);
    db.extend_from_slice(message);
    let db_mask = mgf1(seed, db_len);
    for (byte, mask) in db.iter_mut().zip(&db_mask) {
        *byte ^= mask;
    }
    let seed_mask = mgf1(&db, HASH_LEN);
    let mut masked_seed = *seed;
    for (byte, mask) in masked_seed.iter_mut().zip(&seed_mask) {
        *byte ^= mask;
    }
    let mut em = Vec::with_capacity(k);
    em.push(0x00);
    em.extend_from_slice(&masked_seed);
    em.extend_from_slice(&db);
    Some(em)
}

/// MGF1 with SHA-1, RFC 8017 appendix B.2.1.
fn mgf1(seed: &[u8], length: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(length);
    let mut counter: u32 = 0;
    while out.len() < length {
        let mut hasher = Sha1::new();
        hasher.update(seed);
        hasher.update(counter.to_be_bytes());
        out.extend_from_slice(&hasher.finalize());
        counter += 1;
    }
    out.truncate(length);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_message_follows_rfc_limit() {
        assert_eq!(max_message_len(256), Some(214));
        assert_eq!(max_message_len(42), Some(0));
        assert_eq!(max_message_len(41), None);
    }

    #[test]
    fn encode_shape() {
        // EM = 0x00 || maskedSeed(20) || maskedDB(235);
        // DB = lHash || PS || 0x01 || M; maskedDB is DB ^ dbMask.
        // Decoding maskedDB recovers the original message at the end of DB.
        let seed = [7u8; HASH_LEN];
        let em = encode(256, b"hello", &seed).unwrap();
        assert_eq!(em.len(), 256);
        assert_eq!(em[0], 0);

        let masked_seed = &em[1..1 + HASH_LEN];
        let masked_db = &em[1 + HASH_LEN..];
        let seed_mask = mgf1(masked_db, HASH_LEN);
        let mut recovered_seed = [0u8; HASH_LEN];
        for (i, (&m, s)) in masked_seed.iter().zip(&seed_mask).enumerate() {
            recovered_seed[i] = m ^ s;
        }
        assert_eq!(recovered_seed, seed);

        let db_mask = mgf1(&recovered_seed, masked_db.len());
        let mut db = masked_db.to_vec();
        for (byte, mask) in db.iter_mut().zip(&db_mask) {
            *byte ^= mask;
        }
        assert_eq!(&db[db.len() - 5..], b"hello");
        assert!(encode(256, &[0u8; 215], &seed).is_none());
    }
}
