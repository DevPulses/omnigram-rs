//! Низкоуровневый ридер TL-потока.
//!
//! Порт `Reader<mtpPrime>` из `Telegram/SourceFiles/mtproto/core_types.h`.
//! Поток TL — последовательность 32-битных «праймов» (little-endian слова).
//! Ридер работает поверх байтового среза, но продвигается словами:
//! `get_bytes` копирует `count` байт и сдвигает позицию на
//! `ceil(count / 4)` слов, как оригинальный `memcpy` + `from += primes`.

/// Ошибка чтения: в потоке не хватает данных.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Eof;

/// Ридер TL-потока поверх байтового среза.
///
/// Позиция всегда кратна 4 на границах слов; чтение байтовых
/// последовательностей добирает до границы слова, как в C++.
pub struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

/// ceil(count / 4) — число праймов, занимаемых `count` байтами.
const fn primes_for_bytes(count: usize) -> usize {
    count / 4 + (count % 4 != 0) as usize
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    /// Осталось праймов (слов) до конца потока.
    pub fn remaining_primes(&self) -> usize {
        (self.data.len() - self.pos) / 4
    }

    /// `Reader::Has` — хватает ли `primes` слов до конца.
    pub fn has(&self, primes: usize) -> bool {
        self.remaining_primes() >= primes
    }

    /// `Reader::HasBytes` — хватает ли `count` байт
    /// (сравнение по занимаемым праймам).
    pub fn has_bytes(&self, count: usize) -> bool {
        self.has(primes_for_bytes(count))
    }

    /// `Reader::Get` — очередное 32-битное слово (LE).
    pub fn get(&mut self) -> Result<u32, Eof> {
        if !self.has(1) {
            return Err(Eof);
        }
        let v = u32::from_le_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ]);
        self.pos += 4;
        Ok(v)
    }

    /// `Reader::GetBytes` — скопировать `count` байт и продвинуться
    /// на `ceil(count / 4)` слов (хвостовые байты слова теряются).
    pub fn get_bytes(&mut self, count: usize) -> Result<&'a [u8], Eof> {
        if !self.has_bytes(count) {
            return Err(Eof);
        }
        let bytes = &self.data[self.pos..self.pos + count];
        self.pos += primes_for_bytes(count) * 4;
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_reads_little_endian_words() {
        let mut r = Reader::new(&[0x78, 0x97, 0x46, 0x60, 0x01, 0x00, 0x00, 0x00]);
        assert_eq!(r.get(), Ok(0x6046_9778));
        assert_eq!(r.get(), Ok(1));
        assert_eq!(r.get(), Err(Eof));
    }

    #[test]
    fn has_bytes_rounds_up_to_primes() {
        // 5 байт занимают 2 слова, значит из 8 байт (2 слова) хватает.
        let r = Reader::new(&[0u8; 8]);
        assert!(r.has_bytes(5));
        assert!(r.has_bytes(8));
        assert!(!r.has_bytes(9));
    }

    #[test]
    fn get_bytes_advances_to_word_boundary() {
        let mut r = Reader::new(&[1, 2, 3, 4, 5, 0xAA, 0xBB, 0xCC, 9, 9, 9, 9]);
        assert_eq!(r.get_bytes(3), Ok(&[1u8, 2, 3][..]));
        // хвост первого слова (0xAA,0xBB,0xCC) пропущен, читается слово 2.
        assert_eq!(r.get(), Ok(0xCCBB_AA05));
    }
}
