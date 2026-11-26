//! Telegram 2FA Cloud Password SRP (Secure Remote Password) cryptography.
//!
//! Ported from `tdesktop/Telegram/SourceFiles/core/core_cloud_password.{h,cpp}`.

use mtproto_auth_key::dh::is_prime_and_good;
use num_bigint::BigUint;
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::{Digest, Sha256, Sha512};

pub const KDF_ITERATIONS: u32 = 100_000;
pub const SIZE_FOR_HASH: usize = 256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloudPasswordAlgoModPow {
    pub salt1: Vec<u8>,
    pub salt2: Vec<u8>,
    pub g: i32,
    pub p: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloudPasswordCheckRequest {
    pub id: u64,
    pub b: Vec<u8>,
    pub algo: CloudPasswordAlgoModPow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloudPasswordResult {
    pub srp_id: u64,
    pub a: Vec<u8>,
    pub m1: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SrpError {
    InvalidPrimeOrGenerator,
    InvalidServerB,
    InvalidModExp,
}

/// Left-pads a big-endian integer byte slice with zeroes to `SIZE_FOR_HASH` (256) bytes.
pub fn num_bytes_for_hash(number: &[u8]) -> Vec<u8> {
    if number.len() >= SIZE_FOR_HASH {
        return number[number.len() - SIZE_FOR_HASH..].to_vec();
    }
    let fill = SIZE_FOR_HASH - number.len();
    let mut out = vec![0u8; SIZE_FOR_HASH];
    out[fill..].copy_from_slice(number);
    out
}

fn biguint_for_hash(n: &BigUint) -> Vec<u8> {
    num_bytes_for_hash(&n.to_bytes_be())
}

fn sha256_parts(parts: &[&[u8]]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part);
    }
    hasher.finalize().to_vec()
}

fn xor_bytes(a: &[u8], b: &[u8]) -> Vec<u8> {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect()
}

fn is_good_large(number: &BigUint, p: &BigUint) -> bool {
    number > &BigUint::from(0u32) && number < p
}

/// Computes `H(salt2 | PBKDF2-HMAC-SHA512(H(salt2 | H(salt1 | password | salt1) | salt2), salt1, 100000) | salt2)`.
pub fn compute_cloud_password_hash(algo: &CloudPasswordAlgoModPow, password: &[u8]) -> Vec<u8> {
    let hash1 = sha256_parts(&[&algo.salt1, password, &algo.salt1]);
    let hash2 = sha256_parts(&[&algo.salt2, &hash1, &algo.salt2]);
    let mut hash3 = [0u8; 64];
    pbkdf2_hmac::<Sha512>(&hash2, &algo.salt1, KDF_ITERATIONS, &mut hash3);
    sha256_parts(&[&algo.salt2, &hash3, &algo.salt2])
}

/// Computes `v = g^x mod p` (256-byte big-endian) for setting/verifying a 2FA password.
pub fn compute_cloud_password_digest(
    algo: &CloudPasswordAlgoModPow,
    password: &[u8],
) -> Result<Vec<u8>, SrpError> {
    if !is_prime_and_good(&algo.p, algo.g) {
        return Err(SrpError::InvalidPrimeOrGenerator);
    }
    let p = BigUint::from_bytes_be(&algo.p);
    let g = BigUint::from(algo.g as u32);
    let x = BigUint::from_bytes_be(&compute_cloud_password_hash(algo, password));
    let v = g.modpow(&x, &p);
    Ok(biguint_for_hash(&v))
}

/// Computes SRP client proof `(A, M1)` using OS CSPRNG.
pub fn compute_cloud_password_check(
    request: &CloudPasswordCheckRequest,
    password_hash: &[u8],
) -> Result<CloudPasswordResult, SrpError> {
    let mut rng = rand::thread_rng();
    loop {
        let mut random = [0u8; SIZE_FOR_HASH];
        rng.fill_bytes(&mut random);
        match compute_cloud_password_check_with_random(request, password_hash, &random) {
            Ok(res) => return Ok(res),
            Err(SrpError::InvalidModExp) => continue,
            Err(e) => return Err(e),
        }
    }
}

/// Deterministic SRP check computation given explicit random bytes `a` (for testing & vectors).
pub fn compute_cloud_password_check_with_random(
    request: &CloudPasswordCheckRequest,
    password_hash: &[u8],
    random_a: &[u8],
) -> Result<CloudPasswordResult, SrpError> {
    let algo = &request.algo;
    if !is_prime_and_good(&algo.p, algo.g) {
        return Err(SrpError::InvalidPrimeOrGenerator);
    }

    let p = BigUint::from_bytes_be(&algo.p);
    let g = BigUint::from(algo.g as u32);
    let b_num = BigUint::from_bytes_be(&request.b);
    if !is_good_large(&b_num, &p) {
        return Err(SrpError::InvalidServerB);
    }

    let x = BigUint::from_bytes_be(password_hash);
    let p_for_hash = num_bytes_for_hash(&algo.p);
    let g_for_hash = biguint_for_hash(&g);
    let b_for_hash = num_bytes_for_hash(&request.b);

    let g_x = g.modpow(&x, &p);
    let k = BigUint::from_bytes_be(&sha256_parts(&[&p_for_hash, &g_for_hash]));
    let kg_x = (&k * &g_x) % &p;

    let a_num = BigUint::from_bytes_be(random_a);
    let a_pub = g.modpow(&a_num, &p);
    if !is_good_large(&a_pub, &p) {
        return Err(SrpError::InvalidModExp);
    }
    let a_for_hash = biguint_for_hash(&a_pub);
    let u = BigUint::from_bytes_be(&sha256_parts(&[&a_for_hash, &b_for_hash]));
    if u == BigUint::from(0u32) {
        return Err(SrpError::InvalidModExp);
    }

    // g_b = (B - k*g^x) mod p
    let g_b = if b_num >= kg_x {
        (&b_num - &kg_x) % &p
    } else {
        (&p + &b_num - &kg_x) % &p
    };
    if !is_good_large(&g_b, &p) {
        return Err(SrpError::InvalidModExp);
    }

    let exp = &a_num + (&u * &x);
    let s = g_b.modpow(&exp, &p);
    let k_session = sha256_parts(&[&biguint_for_hash(&s)]);

    let p_xor_g = xor_bytes(&sha256_parts(&[&p_for_hash]), &sha256_parts(&[&g_for_hash]));
    let m1 = sha256_parts(&[
        &p_xor_g,
        &sha256_parts(&[&algo.salt1]),
        &sha256_parts(&[&algo.salt2]),
        &a_for_hash,
        &b_for_hash,
        &k_session,
    ]);

    Ok(CloudPasswordResult {
        srp_id: request.id,
        a: a_for_hash,
        m1,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use mtproto_auth_key::dh::GOOD_PRIME;

    #[test]
    fn test_srp_client_server_roundtrip() {
        let algo = CloudPasswordAlgoModPow {
            salt1: b"random_salt_1_for_test".to_vec(),
            salt2: b"random_salt_2_for_test".to_vec(),
            g: 3,
            p: GOOD_PRIME.to_vec(),
        };
        let password = b"super_secret_telegram_2fa";

        let hash = compute_cloud_password_hash(&algo, password);
        let v_bytes = compute_cloud_password_digest(&algo, password).unwrap();
        assert_eq!(v_bytes.len(), SIZE_FOR_HASH);

        // Server generates b and B = (k*v + g^b) mod p
        let p = BigUint::from_bytes_be(&algo.p);
        let g = BigUint::from(algo.g as u32);
        let v = BigUint::from_bytes_be(&v_bytes);
        let p_hash = num_bytes_for_hash(&algo.p);
        let g_hash = biguint_for_hash(&g);
        let k = BigUint::from_bytes_be(&sha256_parts(&[&p_hash, &g_hash]));

        let b_priv = BigUint::from_bytes_be(&[0x42u8; 32]);
        let b_pub = (&k * &v + g.modpow(&b_priv, &p)) % &p;
        let b_bytes = biguint_for_hash(&b_pub);

        let req = CloudPasswordCheckRequest {
            id: 777_888,
            b: b_bytes.clone(),
            algo: algo.clone(),
        };
        let res = compute_cloud_password_check(&req, &hash).unwrap();
        assert_eq!(res.srp_id, 777_888);
        assert_eq!(res.a.len(), SIZE_FOR_HASH);
        assert_eq!(res.m1.len(), 32);

        // Server verifies M1 using A, v, u, b_priv
        let a_pub = BigUint::from_bytes_be(&res.a);
        let u = BigUint::from_bytes_be(&sha256_parts(&[&res.a, &b_bytes]));
        let s_srv = ((&a_pub * v.modpow(&u, &p)) % &p).modpow(&b_priv, &p);
        let k_srv = sha256_parts(&[&biguint_for_hash(&s_srv)]);
        let p_xor_g = xor_bytes(&sha256_parts(&[&p_hash]), &sha256_parts(&[&g_hash]));
        let m1_srv = sha256_parts(&[
            &p_xor_g,
            &sha256_parts(&[&algo.salt1]),
            &sha256_parts(&[&algo.salt2]),
            &res.a,
            &b_bytes,
            &k_srv,
        ]);

        assert_eq!(res.m1, m1_srv);
    }

    #[test]
    fn test_invalid_prime_rejected() {
        let bad_algo = CloudPasswordAlgoModPow {
            salt1: vec![1, 2, 3],
            salt2: vec![4, 5, 6],
            g: 3,
            p: vec![0x01; 32],
        };
        assert_eq!(
            compute_cloud_password_digest(&bad_algo, b"pw"),
            Err(SrpError::InvalidPrimeOrGenerator)
        );
    }
}
