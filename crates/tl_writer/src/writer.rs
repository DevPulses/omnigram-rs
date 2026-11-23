//! Низкоуровневый писатель TL-потока.
//!
//! Порт `Writer<mtpBuffer>` из `Telegram/SourceFiles/mtproto/core_types.h`
//! и `Writer<LengthCounter>` из `lib_tl/tl/tl_basic_types.h`.
//! Как в оригинале, два режима:
//! - буфер слов (`put`/`put_bytes`) — сериализация;
//! - подсчёт длины без сборки буфера (`count_bytes` — `count_length`).

/// Накопитель сериализации: буфер 32-битных праймов.
#[derive(Default)]
pub struct Writer {
    words: Vec<u32>,
}

impl Writer {
    pub fn new() -> Self {
        Self::default()
    }

    /// `Writer::Put` — слово целиком.
    pub fn put(&mut self, value: u32) {
        self.words.push(value);
    }

    /// `Writer::PutBytes` — `count` байт, дополненные до слова нулями.
    ///
    /// В C++ `memcpy` копирует `count` байт в зарезервированные
    /// `primes` слов, хвост последнего слова остаётся нулевым.
    pub fn put_bytes(&mut self, bytes: &[u8]) {
        for chunk in bytes.chunks(4) {
            let mut word = [0u8; 4];
            word[..chunk.len()].copy_from_slice(chunk);
            self.words.push(u32::from_le_bytes(word));
        }
    }

    pub fn into_words(self) -> Vec<u32> {
        self.words
    }

    /// Байтовое представление (LE), всегда кратно 4.
    pub fn into_bytes(self) -> Vec<u8> {
        self.words.iter().flat_map(|w| w.to_le_bytes()).collect()
    }
}

/// `count_length` — длина сериализации `count` байт с округлением
/// до слов (`LengthCounter::PutBytes`).
pub fn count_bytes(count: usize) -> usize {
    const PRIME: usize = 4;
    (count / PRIME + (count % PRIME != 0) as usize) * PRIME
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_bytes_pads_to_words() {
        let mut w = Writer::new();
        w.put_bytes(b"ab");
        assert_eq!(w.into_bytes(), vec![b'a', b'b', 0, 0]);
    }

    #[test]
    fn multi_word_bytes_keep_zero_tail() {
        let mut w = Writer::new();
        w.put_bytes(&[1, 2, 3, 4, 5]);
        assert_eq!(w.into_words(), vec![0x0403_0201, 5]);
    }

    #[test]
    fn count_matches_original_rounding() {
        assert_eq!(count_bytes(0), 0);
        assert_eq!(count_bytes(1), 4);
        assert_eq!(count_bytes(4), 4);
        assert_eq!(count_bytes(5), 8);
        assert_eq!(count_bytes(254), 256);
    }
}
