//! RSA public key of the MTProto layer: PEM parsing, the server key
//! fingerprint, and the fixed-size operations the protocol needs.
//!
//! Source of truth:
//! `tdesktop/Telegram/SourceFiles/mtproto/details/mtproto_rsa_public_key.{h,cpp}`.
//!
//! The C++ class holds an OpenSSL `RSA*`; here a key is the `(n, e)` pair
//! and every primitive stays in a tested crate: modular exponentiation is
//! `num_bigint::BigUint::modpow`, SHA-1 is the RustCrypto `sha1` crate,
//! the OAEP seed is OS randomness through `rand`. The protocol layer
//! around them is ported from the original:
//!
//! - the fingerprint is `SHA1(tl_bytes(n) || tl_bytes(e))`, bytes `12..20`
//!   read as a little-endian `u64` — `Private::computeFingerprint`, with
//!   the serialization being the `tl_writer` port of `MTP_bytes::write`
//!   and `n`/`e` in the minimal big-endian form of `BN_bn2bin`
//!   (`ToBytes` in the original);
//! - `encrypt` and `decrypt` reproduce the observable result of
//!   `RSA_public_{encrypt,decrypt}` with `RSA_NO_PADDING` over exactly
//!   256 bytes: a 256-byte output with the value right-aligned — what
//!   OpenSSL 3.x writes through `BN_bn2binpad`, and what the original's
//!   short-result realignment branch computes on older OpenSSL — and an
//!   empty vector wherever OpenSSL reports an error;
//! - `encrypt_oaep_padding` is `RSA_PKCS1_OAEP_PADDING` with the OpenSSL
//!   defaults the original relies on: SHA-1 for the label hash and MGF1,
//!   empty label.
//!
//! Deviations from the original, all in its crash-only corners:
//!
//! - an unparseable PEM yields an invalid key instead of aborting through
//!   `Unexpected("format in RSAPublicKey::Private::Create.")` — keys
//!   arrive from the network (CDN config) and must not kill the process;
//! - the original `decrypt` short-result branch shifts by
//!   `zeroBytes - res`, which underflows for any `res > 128`; OpenSSL 3.x
//!   always returns `RSA_size` bytes, so that branch is dead, and this
//!   port implements only the reachable behavior.
//!
//! Layers:
//! - [`pem`] — `GuessFormat` plus the PKCS#1 / SPKI readers;
//! - [`oaep`] — the RSAES-OAEP encoding of RFC 8017.

mod oaep;
mod pem;

use num_bigint::BigUint;
use rand::RngCore;
use sha1::{Digest, Sha1};
use tl_writer::{write_string, Writer};

#[cfg(test)]
mod test_data;

/// `kEncryptSize` and `kDecryptSize` from the original: the byte size of
/// the 2048-bit modulus, the only key size the protocol uses.
const KEY_SIZE: usize = 256;

/// `MTP::details::RSAPublicKey`.
#[derive(Clone, Default, Debug)]
pub struct RsaPublicKey {
    /// The `std::shared_ptr<Private>`: `None` only for the default
    /// constructor.
    private: Option<Private>,
}

/// `RSAPublicKey::Private`.
#[derive(Clone, Debug)]
enum Private {
    /// `_rsa == nullptr`: constructed, but the key did not parse.
    Failed,
    Key(KeyData),
}

#[derive(Clone, Debug)]
struct KeyData {
    n: BigUint,
    e: BigUint,
    fingerprint: u64,
}

impl KeyData {
    /// `BN_num_bytes(n)` — the minimal big-endian length, 0 for zero.
    fn byte_len(&self) -> usize {
        ((self.n.bits() as usize) + 7) / 8
    }
}

impl RsaPublicKey {
    /// `RSAPublicKey()` — the empty key: `is_empty()`, not `is_valid()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `RSAPublicKey(nBytes, eBytes)` — a key from big-endian modulus and
    /// exponent bytes. `RSA_set0_key` accepts any pair, so, like the
    /// original, the result is `is_valid()` even for zero or short
    /// values; operations on such a key then fail the way
    /// `RSA_public_encrypt` does, by returning an empty vector.
    pub fn from_parts(n: &[u8], e: &[u8]) -> Self {
        Self::from_key_data(KeyData {
            n: BigUint::from_bytes_be(n),
            e: BigUint::from_bytes_be(e),
            fingerprint: 0,
        })
    }

    /// `RSAPublicKey(key)` — parses a PEM in the
    /// `-----BEGIN RSA PUBLIC KEY-----` (PKCS#1) or
    /// `-----BEGIN PUBLIC KEY-----` (SPKI) form, picked by searching the
    /// raw bytes for the markers, like `GuessFormat`. A key that does not
    /// parse is not empty, but not valid either.
    pub fn from_pem(key: &[u8]) -> Self {
        match pem::parse_pem(key) {
            Some(parts) => Self::from_key_data(KeyData {
                n: parts.n,
                e: parts.e,
                fingerprint: 0,
            }),
            None => Self {
                private: Some(Private::Failed),
            },
        }
    }

    /// `empty()` — true only for the default-constructed key.
    pub fn is_empty(&self) -> bool {
        self.private.is_none()
    }

    /// `valid()`.
    pub fn is_valid(&self) -> bool {
        matches!(self.private, Some(Private::Key(_)))
    }

    /// `fingerprint()`.
    ///
    /// Panics when the key is not valid, mirroring the `Expects` of the
    /// original.
    pub fn fingerprint(&self) -> u64 {
        self.key().fingerprint
    }

    /// `getN()` — the minimal big-endian modulus, as `BN_bn2bin` writes
    /// it (empty for a zero modulus).
    pub fn get_n(&self) -> Vec<u8> {
        to_minimal_bytes(&self.key().n)
    }

    /// `getE()` — the minimal big-endian exponent.
    pub fn get_e(&self) -> Vec<u8> {
        to_minimal_bytes(&self.key().e)
    }

    /// `encrypt(data)` — raw RSA (`RSA_NO_PADDING`) over exactly 256
    /// bytes. Returns the 256-byte ciphertext, the value right-aligned,
    /// or an empty vector where OpenSSL fails: a modulus whose byte
    /// length is not 256 ("data too small for key size"), or a value not
    /// below the modulus ("data too large for modulus").
    ///
    /// Panics when the key is not valid or the input is not 256 bytes
    /// long, mirroring the `Expects` contracts of the original.
    pub fn encrypt(&self, data: &[u8]) -> Vec<u8> {
        let key = self.key();
        assert_eq!(
            data.len(),
            KEY_SIZE,
            "RSAPublicKey::encrypt: data must be exactly {KEY_SIZE} bytes"
        );
        raw_operation(key, &BigUint::from_bytes_be(data))
    }

    /// `decrypt(data)` — the raw RSA public operation
    /// (`RSA_public_decrypt` with `RSA_NO_PADDING`) over exactly 256
    /// bytes: `data^e mod n`, right-aligned into 256 bytes, with the same
    /// failure and panic behavior as
    /// [`encrypt`](Self::encrypt).
    pub fn decrypt(&self, data: &[u8]) -> Vec<u8> {
        let key = self.key();
        assert_eq!(
            data.len(),
            KEY_SIZE,
            "RSAPublicKey::decrypt: data must be exactly {KEY_SIZE} bytes"
        );
        raw_operation(key, &BigUint::from_bytes_be(data))
    }

    /// `encryptOAEPpadding(data)` — `RSA_PKCS1_OAEP_PADDING` with the
    /// OpenSSL defaults: SHA-1 for the label hash and MGF1, empty label,
    /// a fresh 20 random seed bytes. Returns exactly `RSA_size` bytes.
    /// The input must be at most `RSA_size - 42` bytes — 214 for a
    /// 2048-bit key, where the original's header comment rounds up to
    /// 215 — otherwise the result is an empty vector, the OpenSSL error
    /// path.
    ///
    /// Panics when the key is not valid, mirroring `Expects`.
    pub fn encrypt_oaep_padding(&self, data: &[u8]) -> Vec<u8> {
        let key = self.key();
        let mut seed = [0u8; oaep::HASH_LEN];
        rand::thread_rng().fill_bytes(&mut seed);
        encrypt_oaep_with_seed(key, data, &seed)
    }

    fn from_key_data(mut data: KeyData) -> Self {
        data.fingerprint = compute_fingerprint(&data.n, &data.e);
        Self {
            private: Some(Private::Key(data)),
        }
    }

    /// The `Expects(valid())` in front of every accessor.
    fn key(&self) -> &KeyData {
        match &self.private {
            Some(Private::Key(key)) => key,
            _ => panic!("RSAPublicKey: the key is not valid"),
        }
    }
}

/// One `RSA_NO_PADDING` public operation for `encrypt` and `decrypt`:
/// `value^e mod n`, right-aligned to 256 bytes.
fn raw_operation(key: &KeyData, value: &BigUint) -> Vec<u8> {
    // RSA_padding_add_none demands an input byte length equal to the
    // modulus, and rsa_ossl_{encrypt,decrypt} reject a value not below
    // the modulus; both become the C++ empty-vector error path.
    if key.byte_len() != KEY_SIZE || value >= &key.n {
        return Vec::new();
    }
    let result = value.modpow(&key.e, &key.n);
    right_aligned(&result, KEY_SIZE).unwrap_or_default()
}

/// The deterministic core of `encrypt_oaep_padding`, split out so tests
/// can pin the seed and check the encoding bit for bit.
pub(crate) fn encrypt_oaep_with_seed(
    key: &KeyData,
    data: &[u8],
    seed: &[u8; oaep::HASH_LEN],
) -> Vec<u8> {
    let size = key.byte_len();
    match oaep::max_message_len(size) {
        Some(max) if data.len() <= max => {}
        _ => return Vec::new(),
    }
    let Some(encoded) = oaep::encode(size, data, seed) else {
        return Vec::new();
    };
    // The encoding starts with a zero byte, so it stays below any
    // modulus of the same byte length.
    let cipher = BigUint::from_bytes_be(&encoded).modpow(&key.e, &key.n);
    right_aligned(&cipher, size).unwrap_or_default()
}

/// `ToBytes` in original: BN_bn2bin writes 0 bytes if the number is zero.
fn to_minimal_bytes(value: &BigUint) -> Vec<u8> {
    if value == &BigUint::ZERO {
        Vec::new()
    } else {
        value.to_bytes_be()
    }
}

/// `Private::computeFingerprint`: `MTP_bytes(ToBytes(n))` and
/// `MTP_bytes(ToBytes(e))` written into an `mtpBuffer`, SHA-1 over the
/// raw buffer, bytes `12..20` as a little-endian `u64`.
fn compute_fingerprint(n: &BigUint, e: &BigUint) -> u64 {
    let mut buffer = Writer::new();
    write_string(&mut buffer, &to_minimal_bytes(n));
    write_string(&mut buffer, &to_minimal_bytes(e));
    let mut hasher = Sha1::new();
    hasher.update(&buffer.into_bytes());
    let digest = hasher.finalize();
    u64::from_le_bytes(digest[12..20].try_into().expect("20 digest bytes"))
}

/// `BN_bn2binpad`: the minimal big-endian bytes, zero-padded on the left.
fn right_aligned(value: &BigUint, size: usize) -> Option<Vec<u8>> {
    let bytes = value.to_bytes_be();
    if bytes.len() > size {
        return None;
    }
    let mut out = vec![0u8; size];
    out[size - bytes.len()..].copy_from_slice(&bytes);
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::test_data as td;
    use super::*;

    #[test]
    fn builtin_fingerprints_match_independent_oracle() {
        let test = RsaPublicKey::from_pem(td::TEST_PKCS1_PEM.as_bytes());
        assert_eq!(test.fingerprint(), 0xb258_98df_208d_2603);
        let prod = RsaPublicKey::from_pem(td::PROD_PKCS1_PEM.as_bytes());
        assert_eq!(prod.fingerprint(), 0xd09d_1d85_de64_fd85);
        let prod_spki = RsaPublicKey::from_pem(td::PROD_SPKI_PEM.as_bytes());
        assert_eq!(prod_spki.fingerprint(), 0xd09d_1d85_de64_fd85);
    }

    #[test]
    fn pem_formats_and_parts_agree() {
        let by_pkcs1 = RsaPublicKey::from_pem(td::TEST_PKCS1_PEM.as_bytes());
        let by_spki = RsaPublicKey::from_pem(td::TEST_SPKI_PEM.as_bytes());
        assert!(by_pkcs1.is_valid() && by_spki.is_valid());
        assert_eq!(by_pkcs1.fingerprint(), by_spki.fingerprint());
        assert_eq!(by_pkcs1.get_n(), by_spki.get_n());
        assert_eq!(by_pkcs1.get_e(), by_spki.get_e());
        assert_eq!(by_pkcs1.get_e(), vec![0x01, 0x00, 0x01]);
        assert_eq!(by_pkcs1.get_n().len(), 256);

        let from_parts = RsaPublicKey::from_parts(&by_pkcs1.get_n(), &by_pkcs1.get_e());
        assert_eq!(from_parts.fingerprint(), by_pkcs1.fingerprint());
        assert_eq!(from_parts.get_n(), by_pkcs1.get_n());
    }

    #[test]
    fn generated_key_matches_openssl_modulus() {
        let key = RsaPublicKey::from_pem(td::GEN_SPKI_PEM.as_bytes());
        assert_eq!(key.get_n(), hex::decode(td::GEN_N_HEX).unwrap());
        assert_eq!(key.get_e(), hex::decode(td::GEN_E_HEX).unwrap());
        assert_eq!(key.fingerprint(), td::GEN_FINGERPRINT);
        let parts = RsaPublicKey::from_parts(&key.get_n(), &key.get_e());
        assert_eq!(parts.fingerprint(), td::GEN_FINGERPRINT);
    }

    #[test]
    fn parse_failures_are_invalid_but_not_empty() {
        for bad in [
            &b""[..],
            b"not a pem at all",
            b"-----BEGIN RSA PUBLIC KEY-----\n!!!\n-----END RSA PUBLIC KEY-----",
            b"-----BEGIN PUBLIC KEY-----\nAAAA\n-----END PUBLIC KEY-----",
            b"-----BEGIN RSA PUBLIC KEY-----\nno end marker",
        ] {
            let key = RsaPublicKey::from_pem(bad);
            assert!(!key.is_empty(), "{bad:?}");
            assert!(!key.is_valid(), "{bad:?}");
        }
        let empty = RsaPublicKey::new();
        assert!(empty.is_empty());
        assert!(!empty.is_valid());
    }

    #[test]
    fn pem_reader_skips_leading_lines() {
        let with_prefix = format!("some header line\n\n{}", td::GEN_PKCS1_PEM);
        let key = RsaPublicKey::from_pem(with_prefix.as_bytes());
        assert!(key.is_valid());
        assert_eq!(key.fingerprint(), td::GEN_FINGERPRINT);
    }

    #[test]
    #[should_panic(expected = "the key is not valid")]
    fn accessors_panic_on_invalid_key() {
        let key = RsaPublicKey::from_pem(b"garbage");
        let _ = key.fingerprint();
    }

    #[test]
    fn raw_encrypt_is_bit_exact_with_openssl() {
        // openssl pkeyutl -encrypt -pubin -pkeyopt rsa_padding_mode:none
        let key = RsaPublicKey::from_pem(td::GEN_SPKI_PEM.as_bytes());
        let m = hex::decode(td::M_RAW_HEX).unwrap();
        assert_eq!(key.encrypt(&m), hex::decode(td::C_RAW_HEX).unwrap());
    }

    #[test]
    fn raw_decrypt_matches_modpow_oracle() {
        // Expected value: s^e mod n, with the arithmetic cross-checked
        // against OpenSSL raw encryption in the fixture above.
        let key = RsaPublicKey::from_pem(td::GEN_SPKI_PEM.as_bytes());
        let s = hex::decode(td::S_RAW_HEX).unwrap();
        assert_eq!(key.decrypt(&s), hex::decode(td::M_EXP_HEX).unwrap());
    }

    #[test]
    fn raw_operation_rejects_value_not_below_modulus() {
        // openssl: "data too large for modulus".
        let key = RsaPublicKey::from_pem(td::GEN_SPKI_PEM.as_bytes());
        assert!(key.encrypt(&[0xff; KEY_SIZE]).is_empty());
        assert!(key.decrypt(&[0xff; KEY_SIZE]).is_empty());
    }

    #[test]
    fn raw_operation_of_zero_is_zero_block() {
        let key = RsaPublicKey::from_pem(td::GEN_SPKI_PEM.as_bytes());
        assert_eq!(key.encrypt(&[0; KEY_SIZE]), vec![0; KEY_SIZE]);
    }

    #[test]
    #[should_panic(expected = "exactly 256 bytes")]
    fn encrypt_rejects_wrong_size() {
        let key = RsaPublicKey::from_pem(td::GEN_SPKI_PEM.as_bytes());
        let _ = key.encrypt(&[0u8; 128]);
    }

    #[test]
    fn oaep_encoding_is_bit_exact_with_openssl() {
        // The expected ciphertext was produced with an independent Python
        // OAEP-MGF1-SHA1 encoder and confirmed decryptable by
        // `openssl pkeyutl -decrypt -pkeyopt rsa_padding_mode:oaep
        //  -pkeyopt rsa_oaep_md:sha1` back to the original message.
        let key = RsaPublicKey::from_pem(td::GEN_SPKI_PEM.as_bytes());
        let msg = hex::decode(td::OAEP_MSG_HEX).unwrap();
        let seed: [u8; oaep::HASH_LEN] =
            hex::decode(td::OAEP_SEED_HEX).unwrap().try_into().unwrap();
        let result = encrypt_oaep_with_seed(key.key(), &msg, &seed);
        assert_eq!(result, hex::decode(td::OAEP_C_HEX).unwrap());
    }

    #[test]
    fn oaep_random_seed_full_size_result() {
        let key = RsaPublicKey::from_pem(td::GEN_SPKI_PEM.as_bytes());
        let msg = hex::decode(td::OAEP_MSG_HEX).unwrap();
        let out = key.encrypt_oaep_padding(&msg);
        assert_eq!(out.len(), KEY_SIZE);
        assert!(!out.iter().all(|&byte| byte == 0));
    }

    #[test]
    fn oaep_input_limit_is_modulus_minus_42() {
        let key = RsaPublicKey::from_pem(td::GEN_SPKI_PEM.as_bytes());
        assert!(!key.encrypt_oaep_padding(&[7u8; KEY_SIZE - 42]).is_empty());
        assert!(key.encrypt_oaep_padding(&[7u8; KEY_SIZE - 41]).is_empty());
    }

    #[test]
    fn short_modulus_fails_both_operations() {
        // RSA_padding_add_none demands a 256-byte modulus for the raw
        // path, and OAEP needs at least 42 bytes plus the message.
        let gen_n = hex::decode(td::GEN_N_HEX).unwrap();
        let key = RsaPublicKey::from_parts(&gen_n[..16], &[1, 0, 1]);
        assert!(key.is_valid());
        assert!(key.encrypt(&[0; KEY_SIZE]).is_empty());
        assert!(key.decrypt(&[0; KEY_SIZE]).is_empty());
        assert!(key.encrypt_oaep_padding(b"secret").is_empty());
    }

    #[test]
    fn zero_modulus_parts_stay_valid_but_inert() {
        // RSA_set0_key accepts a zero modulus, so from_parts stays valid;
        // operations then fail the way RSA_public_encrypt would.
        let key = RsaPublicKey::from_parts(&[], &[]);
        assert!(!key.is_empty());
        assert!(key.is_valid());
        assert_eq!(key.get_n(), Vec::new());
        assert_eq!(key.get_e(), Vec::new());
        assert!(key.encrypt(&[0; KEY_SIZE]).is_empty());
        assert!(key.decrypt(&[0; KEY_SIZE]).is_empty());
        assert!(key.encrypt_oaep_padding(b"abc").is_empty());
    }
}
