use aes::cipher::{BlockEncrypt, KeyInit};
use aes::Aes256;

pub const KEY_SIZE: usize = 32;
pub const IV_SIZE: usize = 16;
pub const BLOCK_SIZE: usize = 16;

/// AES-256 CTR cipher state matching OpenSSL `CRYPTO_ctr128_encrypt`
/// exactly as used in Telegram Desktop (`CTRState`).
#[derive(Clone)]
pub struct Ctr128 {
    cipher: Aes256,
    ivec: [u8; IV_SIZE],
    ecount: [u8; BLOCK_SIZE],
    num: usize,
}

impl Ctr128 {
    pub fn new(key: &[u8; KEY_SIZE], ivec: &[u8; IV_SIZE]) -> Self {
        let cipher = Aes256::new_from_slice(key).unwrap();
        Self {
            cipher,
            ivec: *ivec,
            ecount: [0u8; BLOCK_SIZE],
            num: 0,
        }
    }

    /// In-place encrypts/decrypts data stream.
    pub fn apply_keystream(&mut self, data: &mut [u8]) {
        for b in data.iter_mut() {
            if self.num == 0 {
                let mut block = aes::Block::from(self.ivec);
                self.cipher.encrypt_block(&mut block);
                self.ecount.copy_from_slice(&block);
                // Increment 128-bit big-endian counter
                for byte in self.ivec.iter_mut().rev() {
                    *byte = byte.wrapping_add(1);
                    if *byte != 0 {
                        break;
                    }
                }
            }
            *b ^= self.ecount[self.num];
            self.num = (self.num + 1) % BLOCK_SIZE;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openssl_ctr128_oracle() {
        let mut key = [0u8; KEY_SIZE];
        for i in 0..KEY_SIZE {
            key[i] = i as u8;
        }
        let mut ivec = [0u8; IV_SIZE];
        for i in 0..IV_SIZE {
            ivec[i] = i as u8;
        }

        let mut ctr = Ctr128::new(&key, &ivec);
        let mut data = b"Hello MTProto CTR mode test data".to_vec();
        ctr.apply_keystream(&mut data);

        let expected_hex = "120b683b67db3cc2a05c3a496de3e5c632d37dbaa7e0782930658f71168454ae";
        assert_eq!(hex::encode(&data), expected_hex);

        // Decryption check with fresh CTR
        let mut decrypt_ctr = Ctr128::new(&key, &ivec);
        decrypt_ctr.apply_keystream(&mut data);
        assert_eq!(data, b"Hello MTProto CTR mode test data");
    }
}
