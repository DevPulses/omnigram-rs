//! Diffie-Hellman utilities for MTProto key exchange matching Telegram Desktop
//! (`tdesktop/Telegram/SourceFiles/mtproto/mtproto_dh_utils.{h,cpp}`).

use num_bigint::BigUint;
use rand::RngCore;

pub const GOOD_PRIME: [u8; 256] = [
    0xC7, 0x1C, 0xAE, 0xB9, 0xC6, 0xB1, 0xC9, 0x04, 0x8E, 0x6C, 0x52, 0x2F, 0x70, 0xF1, 0x3F, 0x73,
    0x98, 0x0D, 0x40, 0x23, 0x8E, 0x3E, 0x21, 0xC1, 0x49, 0x34, 0xD0, 0x37, 0x56, 0x3D, 0x93, 0x0F,
    0x48, 0x19, 0x8A, 0x0A, 0xA7, 0xC1, 0x40, 0x58, 0x22, 0x94, 0x93, 0xD2, 0x25, 0x30, 0xF4, 0xDB,
    0xFA, 0x33, 0x6F, 0x6E, 0x0A, 0xC9, 0x25, 0x13, 0x95, 0x43, 0xAE, 0xD4, 0x4C, 0xCE, 0x7C, 0x37,
    0x20, 0xFD, 0x51, 0xF6, 0x94, 0x58, 0x70, 0x5A, 0xC6, 0x8C, 0xD4, 0xFE, 0x6B, 0x6B, 0x13, 0xAB,
    0xDC, 0x97, 0x46, 0x51, 0x29, 0x69, 0x32, 0x84, 0x54, 0xF1, 0x8F, 0xAF, 0x8C, 0x59, 0x5F, 0x64,
    0x24, 0x77, 0xFE, 0x96, 0xBB, 0x2A, 0x94, 0x1D, 0x5B, 0xCD, 0x1D, 0x4A, 0xC8, 0xCC, 0x49, 0x88,
    0x07, 0x08, 0xFA, 0x9B, 0x37, 0x8E, 0x3C, 0x4F, 0x3A, 0x90, 0x60, 0xBE, 0xE6, 0x7C, 0xF9, 0xA4,
    0xA4, 0xA6, 0x95, 0x81, 0x10, 0x51, 0x90, 0x7E, 0x16, 0x27, 0x53, 0xB5, 0x6B, 0x0F, 0x6B, 0x41,
    0x0D, 0xBA, 0x74, 0xD8, 0xA8, 0x4B, 0x2A, 0x14, 0xB3, 0x14, 0x4E, 0x0E, 0xF1, 0x28, 0x47, 0x54,
    0xFD, 0x17, 0xED, 0x95, 0x0D, 0x59, 0x65, 0xB4, 0xB9, 0xDD, 0x46, 0x58, 0x2D, 0xB1, 0x17, 0x8D,
    0x16, 0x9C, 0x6B, 0xC4, 0x65, 0xB0, 0xD6, 0xFF, 0x9C, 0xA3, 0x92, 0x8F, 0xEF, 0x5B, 0x9A, 0xE4,
    0xE4, 0x18, 0xFC, 0x15, 0xE8, 0x3E, 0xBE, 0xA0, 0xF8, 0x7F, 0xA9, 0xFF, 0x5E, 0xED, 0x70, 0x05,
    0x0D, 0xED, 0x28, 0x49, 0xF4, 0x7B, 0xF9, 0x59, 0xD9, 0x56, 0x85, 0x0C, 0xE9, 0x29, 0x85, 0x1F,
    0x0D, 0x81, 0x15, 0xF6, 0x35, 0xB1, 0x05, 0xEE, 0x2E, 0x4E, 0x15, 0xD0, 0x4B, 0x24, 0x54, 0xBF,
    0x6F, 0x4F, 0xAD, 0xF0, 0x34, 0xB1, 0x04, 0x03, 0x11, 0x9C, 0xD8, 0xE3, 0xB9, 0x2F, 0xCC, 0x5B,
];

pub const RANDOM_POWER_SIZE: usize = 256;
pub const MAX_MOD_EXP_SIZE: usize = 256;

#[derive(Debug, Clone)]
pub struct ModExpFirst {
    pub modexp: Vec<u8>,
    pub random_power: Vec<u8>,
}

pub fn is_prime_and_good(prime_bytes: &[u8], g: i32) -> bool {
    if prime_bytes == &GOOD_PRIME[..] {
        if g == 3 || g == 4 || g == 5 || g == 7 {
            return true;
        }
    }
    // Check 2048-bit size
    let prime = BigUint::from_bytes_be(prime_bytes);
    if prime.bits() != 2048 {
        return false;
    }
    match g {
        2 => (&prime % 8u32) == 7u32.into(),
        3 => (&prime % 3u32) == 2u32.into(),
        4 => true,
        5 => {
            let mod5 = &prime % 5u32;
            mod5 == 1u32.into() || mod5 == 4u32.into()
        }
        6 => {
            let mod24 = &prime % 24u32;
            mod24 == 19u32.into() || mod24 == 23u32.into()
        }
        7 => {
            let mod7 = &prime % 7u32;
            mod7 == 3u32.into() || mod7 == 5u32.into() || mod7 == 6u32.into()
        }
        _ => false,
    }
}

pub fn is_good_mod_exp_first(modexp: &BigUint, prime: &BigUint) -> bool {
    if modexp >= prime {
        return false;
    }
    let diff = prime - modexp;
    const MIN_DIFF_BITS: u64 = 2048 - 64;
    if diff.bits() < MIN_DIFF_BITS
        || modexp.bits() < MIN_DIFF_BITS
        || modexp.to_bytes_be().len() > MAX_MOD_EXP_SIZE
    {
        return false;
    }
    true
}

pub fn create_mod_exp(g: i32, prime_bytes: &[u8], random_seed: &[u8]) -> ModExpFirst {
    assert_eq!(random_seed.len(), RANDOM_POWER_SIZE);
    let prime = BigUint::from_bytes_be(prime_bytes);
    let g_big = BigUint::from(g as u32);
    let mut rng = rand::thread_rng();

    loop {
        let mut random_power = vec![0u8; RANDOM_POWER_SIZE];
        rng.fill_bytes(&mut random_power);
        for i in 0..RAND_POWER_SIZE_DEF {
            random_power[i] ^= random_seed[i];
        }
        let b = BigUint::from_bytes_be(&random_power);
        let modexp = g_big.modpow(&b, &prime);
        if is_good_mod_exp_first(&modexp, &prime) {
            return ModExpFirst {
                modexp: modexp.to_bytes_be(),
                random_power,
            };
        }
    }
}

const RAND_POWER_SIZE_DEF: usize = 256;

pub fn create_auth_key(
    first_bytes: &[u8],
    random_bytes: &[u8],
    prime_bytes: &[u8],
) -> Option<Vec<u8>> {
    let first = BigUint::from_bytes_be(first_bytes);
    let prime = BigUint::from_bytes_be(prime_bytes);
    if !is_good_mod_exp_first(&first, &prime) {
        return None;
    }
    let b = BigUint::from_bytes_be(random_bytes);
    Some(first.modpow(&b, &prime).to_bytes_be())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_good_prime_check() {
        assert!(is_prime_and_good(&GOOD_PRIME, 3));
        assert!(is_prime_and_good(&GOOD_PRIME, 4));
        assert!(is_prime_and_good(&GOOD_PRIME, 5));
        assert!(is_prime_and_good(&GOOD_PRIME, 7));
        assert!(!is_prime_and_good(&GOOD_PRIME, 2));
    }

    #[test]
    fn test_dh_exchange() {
        let seed_a = [7u8; 256];
        let seed_b = [9u8; 256];

        // Alice produces g_a
        let alice = create_mod_exp(3, &GOOD_PRIME, &seed_a);
        // Bob produces g_b
        let bob = create_mod_exp(3, &GOOD_PRIME, &seed_b);

        // Alice computes key from g_b
        let key_alice = create_auth_key(&bob.modexp, &alice.random_power, &GOOD_PRIME)
            .expect("alice key");
        // Bob computes key from g_a
        let key_bob = create_auth_key(&alice.modexp, &bob.random_power, &GOOD_PRIME)
            .expect("bob key");

        assert_eq!(key_alice, key_bob);
    }
}
