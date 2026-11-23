//! Чтение базовых TL-типов.
//!
//! Порт `tl_basic_types.h` из `Telegram/lib_tl/tl` (только `read`-половина;
//! запись — задача `tl_writer`). Семантика совпадает 1-в-1 с C++:
//! те же конструкторы-константы (`id_int = 0xa8509bda`, …), те же проверки
//! границ и те же форматы строк/векторов.

use crate::reader::{Eof, Reader};

pub const ID_INT: u32 = 0xa850_9bda;
pub const ID_LONG: u32 = 0x2207_6cba;
pub const ID_INT128: u32 = 0x4bb5_362b;
pub const ID_INT256: u32 = 0x0929_c32f;
pub const ID_DOUBLE: u32 = 0x2210_c154;
pub const ID_STRING: u32 = 0xb528_6e24;
pub const ID_VECTOR: u32 = 0x1cb5_c415;

pub const ID_BYTES: u32 = ID_STRING;
pub const ID_FLAGS: u32 = ID_INT;
pub const ID_FLAGS64: u32 = ID_LONG;

/// 128-битное целое: low + high, как `int128_type` (поля `l`, `h`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Int128 {
    pub l: u64,
    pub h: u64,
}

/// 256-битное целое: два `Int128` (поля `l`, `h`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Int256 {
    pub l: Int128,
    pub h: Int128,
}

/// Считать `int32` (`int_type::read`).
pub fn read_int(r: &mut Reader) -> Result<i32, Eof> {
    Ok(r.get()? as i32)
}

/// Считать `long`/`int64` (`long_type::read`): два слова, low затем high.
pub fn read_long(r: &mut Reader) -> Result<u64, Eof> {
    let lo = r.get()? as u64;
    let hi = r.get()? as u64;
    Ok(lo | (hi << 32))
}

/// Считать `int128` (`int128_type::read`).
pub fn read_int128(r: &mut Reader) -> Result<Int128, Eof> {
    let l = read_long(r)?;
    let h = read_long(r)?;
    Ok(Int128 { l, h })
}

/// Считать `int256` (`int256_type::read`).
pub fn read_int256(r: &mut Reader) -> Result<Int256, Eof> {
    let l = read_int128(r)?;
    let h = read_int128(r)?;
    Ok(Int256 { l, h })
}

/// Считать `double` (`double_type::read`): 64 бита, перечитанные как f64.
pub fn read_double(r: &mut Reader) -> Result<f64, Eof> {
    Ok(f64::from_bits(read_long(r)?))
}

/// Считать строку/байты (`string_type::read`).
///
/// Формат:
/// - длина 0..=254 лежит в младшем байте первого слова (0 — пустая);
///   при длине 1..=3 хвост байтов упакован в старшие байты первого слова;
/// - длина 254 в младшем байте означает: реальная длина — старшие 3 байта
///   первого слова, дальше идут сами байты.
pub fn read_string(r: &mut Reader) -> Result<Vec<u8>, Eof> {
    let first = r.get()?;
    let last = (first & 0xFF) as usize;
    if last > 254 {
        return Err(Eof);
    } else if last == 0 {
        Ok(Vec::new())
    } else if last < 254 {
        // Случаи last == 1, 2, 3..254 единообразно: часть байтов —
        // в старших байтах `first`, остаток — в следующих словах.
        let consumed = last.min(3);
        let mut v = Vec::with_capacity(last);
        for shift in [8u32, 16, 24] {
            if v.len() == last {
                break;
            }
            v.push(((first >> shift) & 0xFF) as u8);
        }
        let remaining = last - consumed;
        if remaining > 0 {
            if !r.has_bytes(remaining) {
                return Err(Eof);
            }
            v.extend_from_slice(r.get_bytes(remaining)?);
        }
        Ok(v)
    } else {
        let length = (first >> 8) as usize;
        if !r.has_bytes(length) {
            return Err(Eof);
        }
        Ok(r.get_bytes(length)?.to_vec())
    }
}

/// Считать вектор (`vector_type::read`): слово-конструктор уже проверено
/// вызывающим кодом, здесь читается счётчик и элементы.
///
/// Как в C++ (`Has(count)` перед чтением), предварительно проверяется,
/// что на каждый элемент есть хотя бы одно слово — отсечение мусорных
/// счётчиков до аллокации.
pub fn read_vector<T>(
    r: &mut Reader,
    mut item: impl FnMut(&mut Reader) -> Result<T, Eof>,
) -> Result<Vec<T>, Eof> {
    let count = r.get()? as usize;
    if !r.has(count) {
        return Err(Eof);
    }
    let mut v = Vec::with_capacity(count.min(r.remaining_primes()));
    for _ in 0..count {
        v.push(item(r)?);
    }
    Ok(v)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reader(bytes: &[u8]) -> Reader<'_> {
        Reader::new(bytes)
    }

    #[test]
    fn int_and_long() {
        // req_pq#60469778 nonce:int128 — конструктор как int для сверки.
        let data = [0x01, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF];
        let mut r = reader(&data);
        assert_eq!(read_int(&mut r), Ok(1));
        assert_eq!(read_int(&mut r), Ok(-1));
        assert_eq!(read_int(&mut r), Err(Eof));

        let data = [0xEF, 0xBE, 0xAD, 0xDE, 0x78, 0x56, 0x34, 0x12];
        let mut r = reader(&data);
        assert_eq!(read_long(&mut r), Ok(0x1234_5678_dead_beef));
    }

    #[test]
    fn int128_int256_double() {
        let mut r = reader(&[
            1, 0, 0, 0, 0, 0, 0, 0, // l.lo, l.hi
            2, 0, 0, 0, 0, 0, 0, 0, // h.lo, h.hi
        ]);
        let v = read_int128(&mut r).unwrap();
        assert_eq!(v, Int128 { l: 1, h: 2 });

        let mut r = reader(&[0u8; 32]);
        assert_eq!(read_int256(&mut r), Ok(Int256::default()));
        assert_eq!(read_int256(&mut r), Err(Eof));

        // 1.0 -> биты 0x3FF0000000000000.
        let mut r = reader(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF0, 0x3F]);
        assert_eq!(read_double(&mut r), Ok(1.0));
    }

    #[test]
    fn strings_all_lengths() {
        // Пустая строка — одно слово 0.
        let mut r = reader(&[0, 0, 0, 0]);
        assert_eq!(read_string(&mut r), Ok(Vec::new()));

        // "q" — упакована в одно слово: 01 71 00 00.
        let mut r = reader(&[0x01, 0x71, 0x00, 0x00]);
        assert_eq!(read_string(&mut r), Ok(b"q".to_vec()));

        // "qw" — 02 71 77 00.
        let mut r = reader(&[0x02, 0x71, 0x77, 0x00]);
        assert_eq!(read_string(&mut r), Ok(b"qw".to_vec()));

        // "qwe" — 03 71 77 65 (без доп. слов: remaining = 0).
        let mut r = reader(&[0x03, 0x71, 0x77, 0x65]);
        assert_eq!(read_string(&mut r), Ok(b"qwe".to_vec()));

        // "qwert" (5) — 05 71 77 65 | 72 74 00 00 (padding).
        let mut r = reader(&[0x05, 0x71, 0x77, 0x65, 0x72, 0x74, 0x00, 0x00]);
        assert_eq!(read_string(&mut r), Ok(b"qwert".to_vec()));
        assert_eq!(r.get(), Err(Eof)); // слово с паддингом израсходовано

        // Длинная строка (300 байт): FE 2C 01 00 + 300 байт + паддинг до 304.
        let payload: Vec<u8> = (0..300).map(|i| (i % 251) as u8).collect();
        let mut data = vec![0xFE, 0x2C, 0x01, 0x00];
        data.extend_from_slice(&payload);
        data.resize(4 + 304, 0xAB);
        let mut r = reader(&data);
        assert_eq!(read_string(&mut r), Ok(payload.clone()));
        // Осталось ровно слово паддинга до границы слова.
        assert_eq!(r.get(), Ok(0xABAB_ABAB));
        assert_eq!(r.get(), Err(Eof));

        // last > 254 (255) — ошибка.
        let mut r = reader(&[0xFF, 0, 0, 0]);
        assert_eq!(read_string(&mut r), Err(Eof));
    }

    #[test]
    fn vector_of_ints() {
        // vector#1cb5c415 { n:int t } = Vector t;
        let mut data = vec![0x15, 0xC4, 0xB5, 0x1C, 0x02, 0, 0, 0, 7, 0, 0, 0, 9, 0, 0, 0];
        let mut r = reader(&data);
        assert_eq!(r.get(), Ok(ID_VECTOR)); // конструктор проверен вызывающим
        let v = read_vector(&mut r, read_int).unwrap();
        assert_eq!(v, vec![7, 9]);
        assert_eq!(r.get(), Err(Eof));

        // Мусорный счётчик отсекается Has(count).
        data[4] = 0xFF;
        let mut r = reader(&data);
        assert_eq!(read_vector(&mut r, read_int), Err(Eof));
    }

    #[test]
    fn real_req_pq_serialization() {
        // Известная сериализация req_pq#60469778 nonce:int128 = ResPQ
        // (nonce = 16 нулевых байт): конструктор + 4 слова нонса.
        let data = [
            0x78, 0x97, 0x46, 0x60, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        let mut r = reader(&data);
        assert_eq!(read_int(&mut r), Ok(0x6046_9778u32 as i32)); // конструктор
        // nonce:int128 — 16 нулевых байт = 4 слова, все нули.
        assert_eq!(read_int128(&mut r), Ok(Int128::default()));
        assert_eq!(r.get(), Err(Eof));
    }
}

#[cfg(test)]
mod dbg_tmp {
    use super::*;
    #[test]
    fn dbg_qw() {
        let mut r = Reader::new(&[0x02, 0x71, 0x77, 0x00]);
        let first = r.get().unwrap();
        eprintln!("first = {first:#010x}");
        eprintln!("b8 = {:#x}", (first >> 8) & 0xFF);
        
    }
}
