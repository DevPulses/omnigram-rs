//! Запись базовых TL-типов.
//!
//! Порт `write`-половины `tl_basic_types.h` из `Telegram/lib_tl/tl`.
//! Форматы зеркальны ридеру из `tl_reader::basic_types`.

use tl_reader::{Int128, Int256, ID_VECTOR};

use crate::writer::Writer;

/// Записать `int` (`int_type::write`).
pub fn write_int(w: &mut Writer, v: i32) {
    w.put(v as u32);
}

/// Записать `long`/`int64` (`long_type::write`): low, затем high.
pub fn write_long(w: &mut Writer, v: u64) {
    w.put((v & 0xFFFF_FFFF) as u32);
    w.put((v >> 32) as u32);
}

/// Записать `int128` (`int128_type::write`).
pub fn write_int128(w: &mut Writer, v: Int128) {
    write_long(w, v.l);
    write_long(w, v.h);
}

/// Записать `int256` (`int256_type::write`).
pub fn write_int256(w: &mut Writer, v: Int256) {
    write_int128(w, v.l);
    write_int128(w, v.h);
}

/// Записать `double` (`double_type::write`): биты значения как 64-битное.
pub fn write_double(w: &mut Writer, v: f64) {
    write_long(w, v.to_bits());
}

/// Записать строку/байты (`string_type::write`).
///
/// Паникует при длине >= 0x1000000, как `Expects` в оригинале.
pub fn write_string(w: &mut Writer, v: &[u8]) {
    assert!(v.len() < 0x100_0000, "string_type::write: length overflow");

    let size = v.len();
    match size {
        0 => w.put(0),
        1 => w.put(1 | ((v[0] as u32) << 8)),
        2 => w.put(2 | ((v[0] as u32) << 8) | ((v[1] as u32) << 16)),
        _ if size < 254 => {
            w.put(
                size as u32 | ((v[0] as u32) << 8)
                    | ((v[1] as u32) << 16)
                    | ((v[2] as u32) << 24),
            );
            w.put_bytes(&v[3..]);
        }
        _ => {
            w.put(((size as u32) << 8) | 254);
            w.put_bytes(v);
        }
    }
}

/// Записать вектор (`vector_type::write`): длина, затем элементы.
pub fn write_vector<T>(
    w: &mut Writer,
    v: &[T],
    mut item: impl FnMut(&mut Writer, &T),
) {
    w.put(v.len() as u32);
    for x in v {
        item(w, x);
    }
}

/// Записать вектор `int` (частый частный случай).
pub fn write_vector_int(w: &mut Writer, v: &[i32]) {
    w.put(ID_VECTOR);
    write_vector(w, v, |w, x| write_int(w, *x));
}

/// Записать boxed-значение: конструктор, затем тело (`tl_boxed.h`).
pub fn write_boxed(w: &mut Writer, constructor: u32, body: impl FnOnce(&mut Writer)) {
    w.put(constructor);
    body(w);
}

#[cfg(test)]
mod tests {
    use super::*;
    use tl_reader::{read_int, read_string, read_vector, Reader};

    fn roundtrip(write: impl FnOnce(&mut Writer), check: impl FnOnce(&mut Reader)) {
        let mut w = Writer::new();
        write(&mut w);
        let bytes = w.into_bytes();
        let mut r = Reader::new(&bytes);
        check(&mut r);
    }

    #[test]
    fn ints() {
        roundtrip(
            |w| {
                write_int(w, -1);
                write_long(w, 0x1234_5678_dead_beef);
            },
            |r| {
                assert_eq!(read_int(r), Ok(-1));
                assert_eq!(tl_reader::read_long(r), Ok(0x1234_5678_dead_beef));
            },
        );
    }

    #[test]
    fn doubles() {
        roundtrip(
            |w| write_double(w, 1.0),
            |r| assert_eq!(tl_reader::read_double(r), Ok(1.0)),
        );
    }

    #[test]
    fn strings_exact_bytes() {
        // Бит-в-бит сверка форматов из C++ write-методов.
        let mut w = Writer::new();
        write_string(&mut w, b""); // 00 00 00 00
        write_string(&mut w, b"q"); // 01 71 00 00
        write_string(&mut w, b"qw"); // 02 71 77 00
        write_string(&mut w, b"qwe"); // 03 71 77 65
        write_string(&mut w, b"qwert"); // 05 71 77 65 | 72 74 00 00
        assert_eq!(
            w.into_bytes(),
            vec![
                0, 0, 0, 0, //
                0x01, 0x71, 0x00, 0x00, //
                0x02, 0x71, 0x77, 0x00, //
                0x03, 0x71, 0x77, 0x65, //
                0x05, 0x71, 0x77, 0x65, 0x72, 0x74, 0x00, 0x00, //
            ]
        );
    }

    #[test]
    fn long_string_header() {
        let payload: Vec<u8> = (0..300).map(|i| (i % 251) as u8).collect();
        let mut w = Writer::new();
        write_string(&mut w, &payload);
        let words: Vec<u32> = w.into_words();
        // Заголовок: 254 в младшем байте, длина 300 в старших трёх.
        assert_eq!(words[0], (300 << 8) | 254);
        // 300 байт занимают 75 слов; всего 76.
        assert_eq!(words.len(), 76);
        // Раундтрип.
        let bytes: Vec<u8> = words.iter().flat_map(|x| x.to_le_bytes()).collect();
        let mut r = Reader::new(&bytes);
        assert_eq!(read_string(&mut r), Ok(payload));
    }

    #[test]
    fn vector_roundtrip() {
        let mut w = Writer::new();
        w.put(ID_VECTOR);
        write_vector(&mut w, &[7i32, 9, -3], |w, x| write_int(w, *x));
        let bytes = w.into_bytes();
        let mut r = Reader::new(&bytes);
        assert_eq!(r.get(), Ok(ID_VECTOR));
        assert_eq!(read_vector(&mut r, read_int), Ok(vec![7, 9, -3]));
    }

    #[test]
    fn boxed_req_pq_shape() {
        // req_pq#60469778 nonce:int128 — классическая сериализация.
        let mut w = Writer::new();
        write_boxed(&mut w, 0x6046_9778, |w| {
            write_int128(w, Int128::default());
        });
        assert_eq!(
            w.into_bytes(),
            vec![0x78, 0x97, 0x46, 0x60, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    #[should_panic(expected = "length overflow")]
    fn string_too_long_panics() {
        let mut w = Writer::new();
        write_string(&mut w, &vec![0u8; 0x100_0000]);
    }
}
