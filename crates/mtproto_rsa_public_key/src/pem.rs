//! PEM and DER decoding of RSA public keys.
//!
//! Port of the parsing half of
//! `tdesktop/Telegram/SourceFiles/mtproto/details/mtproto_rsa_public_key.cpp`:
//! `GuessFormat` picks the container by searching the raw key bytes for
//! the markers, then `PEM_read_bio_RSAPublicKey` or
//! `PEM_read_bio_RSA_PUBKEY` decodes it. The DER support covers exactly
//! what those two readers need: definite lengths, `RSAPublicKey` (PKCS#1)
//! and `SubjectPublicKeyInfo` (SPKI) with the `rsaEncryption` algorithm
//! identifier.

use num_bigint::BigUint;

/// The `rsaEncryption` algorithm identifier — OID 1.2.840.113549.1.1.1
/// with NULL parameters, or with the parameters omitted: `d2i_RSA_PUBKEY`
/// accepts both spellings.
/// Note: These are the inner contents of the AlgorithmIdentifier SEQUENCE (tag 0x30).
const ALG_ID_WITH_NULL: &[u8] = &[
    0x06, 0x09, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01,
    0x01, 0x05, 0x00,
];
const ALG_ID_BARE: &[u8] = &[
    0x06, 0x09, 0x2a, 0x86, 0x48, 0x86, 0xf7, 0x0d, 0x01, 0x01,
    0x01,
];

/// The `(n, e)` pair every supported container decodes to.
pub(crate) struct KeyParts {
    pub(crate) n: BigUint,
    pub(crate) e: BigUint,
}

/// `GuessFormat` plus the matching PEM reader. Returns `None` where the
/// original ends with `nullptr` from `PEM_read_bio_*`; an unknown format,
/// which the original reports through `Unexpected`, becomes the same
/// `None` here (see the crate docs).
pub(crate) fn parse_pem(key: &[u8]) -> Option<KeyParts> {
    if contains(key, b"BEGIN RSA PUBLIC KEY") {
        pem_body(
            key,
            b"-----BEGIN RSA PUBLIC KEY-----",
            b"-----END RSA PUBLIC KEY-----",
        )
        .and_then(|der| parse_pkcs1(&der))
    } else if contains(key, b"BEGIN PUBLIC KEY") {
        pem_body(key, b"-----BEGIN PUBLIC KEY-----", b"-----END PUBLIC KEY-----")
            .and_then(|der| parse_spki(&der))
    } else {
        None
    }
}

/// `QByteArray::indexOf` over raw bytes.
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|window| window == needle)
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// The base64 payload between the markers, with the whitespace the PEM
/// reader skips taken out.
fn pem_body(key: &[u8], begin: &[u8], end: &[u8]) -> Option<Vec<u8>> {
    let start = find(key, begin)? + begin.len();
    let rest = &key[start..];
    let stop = find(rest, end)?;
    let body: Vec<u8> = rest[..stop]
        .iter()
        .copied()
        .filter(|byte| !matches!(byte, b' ' | b'\t' | b'\r' | b'\n'))
        .collect();
    base64_decode(&body)
}

/// Strict base64 decoding of a canonical PEM body: padding only at the
/// end, and only the zero leftover bits a canonical encoder produces.
fn base64_decode(input: &[u8]) -> Option<Vec<u8>> {
    fn value(byte: u8) -> Option<u8> {
        match byte {
            b'A'..=b'Z' => Some(byte - b'A'),
            b'a'..=b'z' => Some(byte - b'a' + 26),
            b'0'..=b'9' => Some(byte - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }

    let mut out = Vec::with_capacity(input.len() / 4 * 3);
    let mut accumulator = 0u32;
    let mut bits = 0u32;
    let mut padding = 0usize;
    for &byte in input {
        if byte == b'=' {
            padding += 1;
            continue;
        }
        if padding > 0 {
            return None;
        }
        accumulator = (accumulator << 6) | value(byte)? as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((accumulator >> bits) as u8);
        }
    }
    let expected_bits = match padding {
        0 => 0,
        1 => 2,
        2 => 4,
        _ => return None,
    };
    if bits != expected_bits {
        return None;
    }
    if bits > 0 && (accumulator & ((1 << bits) - 1)) != 0 {
        return None;
    }
    Some(out)
}

/// Minimal DER cursor for the two key containers.
struct Der<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Der<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    /// One TLV element: the expected tag plus its content.
    fn element(&mut self, tag: u8) -> Option<&'a [u8]> {
        if self.data.get(self.pos) != Some(&tag) {
            return None;
        }
        self.pos += 1;
        let length = self.length()?;
        let end = self.pos.checked_add(length)?;
        if end > self.data.len() {
            return None;
        }
        let content = &self.data[self.pos..end];
        self.pos = end;
        Some(content)
    }

    /// A definite DER length: the short form, or the long form with up to
    /// four length bytes. Indefinite (0x80) lengths are rejected.
    fn length(&mut self) -> Option<usize> {
        let first = *self.data.get(self.pos)?;
        self.pos += 1;
        if first < 0x80 {
            return Some(first as usize);
        }
        let count = (first & 0x7f) as usize;
        if !(1..=4).contains(&count) {
            return None;
        }
        let mut length = 0usize;
        for _ in 0..count {
            let byte = *self.data.get(self.pos)?;
            self.pos += 1;
            length = length.checked_mul(0x100)?.checked_add(byte as usize)?;
        }
        Some(length)
    }

    fn at_end(&self) -> bool {
        self.pos == self.data.len()
    }
}

/// `d2i_RSAPublicKey`: `SEQUENCE { INTEGER n, INTEGER e }`.
fn parse_pkcs1(der: &[u8]) -> Option<KeyParts> {
    let mut outer = Der::new(der);
    let sequence = outer.element(0x30)?;
    let mut inner = Der::new(sequence);
    let n = integer(&mut inner)?;
    let e = integer(&mut inner)?;
    if !inner.at_end() || !outer.at_end() {
        return None;
    }
    Some(KeyParts { n, e })
}

/// `d2i_RSA_PUBKEY`: `SubjectPublicKeyInfo` — the algorithm identifier,
/// then the key as a BIT STRING around the PKCS#1 body.
fn parse_spki(der: &[u8]) -> Option<KeyParts> {
    let mut outer = Der::new(der);
    let sequence = outer.element(0x30)?;
    let mut inner = Der::new(sequence);
    let alg_id = inner.element(0x30)?;
    if alg_id != ALG_ID_WITH_NULL && alg_id != ALG_ID_BARE {
        return None;
    }
    let bit_string = inner.element(0x03)?;
    if !inner.at_end() || !outer.at_end() {
        return None;
    }
    let (unused_bits, key_der) = bit_string.split_first()?;
    if *unused_bits != 0 {
        return None;
    }
    parse_pkcs1(key_der)
}

fn integer(der: &mut Der<'_>) -> Option<BigUint> {
    let content = der.element(0x02)?;
    if content.is_empty() {
        return None; // a DER INTEGER carries at least one byte
    }
    // A positive INTEGER keeps a 0x00 sign byte in front; the big-endian
    // value ignores leading zeros either way.
    Some(BigUint::from_bytes_be(content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_parse() {
        for (name, pem) in [
            ("test_pkcs1", crate::test_data::TEST_PKCS1_PEM),
            ("prod_pkcs1", crate::test_data::PROD_PKCS1_PEM),
            ("gen_pkcs1", crate::test_data::GEN_PKCS1_PEM),
            ("test_spki", crate::test_data::TEST_SPKI_PEM),
            ("gen_spki", crate::test_data::GEN_SPKI_PEM),
        ] {
            let body = pem_body(
                pem.as_bytes(),
                b"-----BEGIN RSA PUBLIC KEY-----",
                b"-----END RSA PUBLIC KEY-----",
            )
            .or_else(|| {
                pem_body(
                    pem.as_bytes(),
                    b"-----BEGIN PUBLIC KEY-----",
                    b"-----END PUBLIC KEY-----",
                )
            });
            match body {
                Some(b) => eprintln!(
                    "{name}: body {} bytes, pkcs1_ok={} spki_ok={}",
                    b.len(),
                    parse_pkcs1(&b).is_some(),
                    parse_spki(&b).is_some()
                ),
                None => eprintln!("{name}: pem_body FAILED"),
            }
        }
    }

    #[test]
    fn base64_known_vectors() {
        assert_eq!(base64_decode(b"").unwrap(), b"");
        assert_eq!(base64_decode(b"QQ==").unwrap(), b"A");
        assert_eq!(base64_decode(b"QUI=").unwrap(), b"AB");
        assert_eq!(base64_decode(b"QUJD").unwrap(), b"ABC");
        assert_eq!(base64_decode(b"QUJDRA==").unwrap(), b"ABCD");
        assert!(base64_decode(b"Q").is_none());
        assert!(base64_decode(b"QQ=").is_none());
        assert!(base64_decode(b"QQ==Q").is_none());
        assert!(base64_decode(b"Q!==").is_none());
        assert!(base64_decode(b"====").is_none());
    }

    #[test]
    fn decodes_both_builtin_formats() {
        let pkcs1 = parse_pem(crate::test_data::TEST_PKCS1_PEM.as_bytes()).unwrap();
        let spki = parse_pem(crate::test_data::TEST_SPKI_PEM.as_bytes()).unwrap();
        assert_eq!(pkcs1.n, spki.n);
        assert_eq!(pkcs1.e, spki.e);
    }

    #[test]
    fn rejects_unknown_and_broken_input() {
        assert!(parse_pem(b"-----BEGIN EC PUBLIC KEY-----").is_none());
        assert!(parse_pem(b"-----BEGIN RSA PUBLIC KEY-----\nMIIBCgKCAQEA").is_none());
    }

    #[test]
    fn der_rejects_malformed_input() {
        assert!(parse_pkcs1(&[]).is_none());
        assert!(parse_pkcs1(&[0x30, 0x00]).is_none());
        // A single INTEGER is not a key.
        assert!(parse_pkcs1(&[0x30, 0x03, 0x02, 0x01, 0x05]).is_none());
        // Trailing bytes after the SEQUENCE.
        assert!(
            parse_pkcs1(&[
                0x30, 0x06, 0x02, 0x01, 0x05, 0x02, 0x01, 0x03, 0x00
            ])
            .is_none()
        );
        // A valid tiny key parses: n = 5, e = 3.
        let key = parse_pkcs1(&[0x30, 0x06, 0x02, 0x01, 0x05, 0x02, 0x01, 0x03]).unwrap();
        assert_eq!(key.n, BigUint::from(5u32));
        assert_eq!(key.e, BigUint::from(3u32));
    }

    #[test]
    fn spki_requires_rsa_encryption_algorithm() {
        // The PKCS#1 body wrapped as a bare BIT STRING, without the
        // algorithm identifier, must not pass as SPKI.
        let pkcs1 = parse_pem(crate::test_data::GEN_PKCS1_PEM.as_bytes()).unwrap();
        assert!(parse_spki(&pkcs1.n.to_bytes_be()).is_none());
        assert!(parse_spki(&[0x30, 0x00]).is_none());
    }
}
