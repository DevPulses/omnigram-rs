//! Оракул 2: бит-в-бит проверки и раундтрипы сгенерированного кода
//! `mtproto.tl` (задача tl_generator).

use tl_generator::generated_mtproto::functions::*;
use tl_generator::generated_mtproto::*;
use tl_reader::{Int128, Int256, Reader};
use tl_writer::Writer;

fn nonce128() -> Int128 {
    Int128 {
        l: 0x0123_4567_89ab_cdef,
        h: 0xfedc_ba98_7654_3210,
    }
}

#[test]
fn req_pq_bit_exact() {
    // req_pq#60469778 nonce:int128 — байты сверяются с ручным расчётом
    // по правилам TL (LE-слова, конструктор, int128 = 4 слова).
    let mut w = Writer::new();
    ReqPq {
        nonce: nonce128(),
    }
    .write_boxed(&mut w);
    let bytes = w.into_bytes();
    let mut expected = vec![0x78, 0x97, 0x46, 0x60]; // конструктор LE
    expected.extend_from_slice(&[
        0xef, 0xcd, 0xab, 0x89, 0x67, 0x45, 0x23, 0x01, // l (2 слова LE)
        0x10, 0x32, 0x54, 0x76, 0x98, 0xba, 0xdc, 0xfe, // h
    ]);
    assert_eq!(bytes, expected);
    assert_eq!(bytes.len(), 20);
}

#[test]
fn req_pq_parse_back() {
    let mut w = Writer::new();
    ReqPq {
        nonce: nonce128(),
    }
    .write_boxed(&mut w);
    let bytes = w.into_bytes();
    let mut r = Reader::new(&bytes);
    // Прочитать как ResPQ нельзя (это функция), читаем вручную:
    // конструктор + int128.
    assert_eq!(r.get(), Ok(0x6046_9778));
    let n = tl_reader::read_int128(&mut r).unwrap();
    assert_eq!(n, nonce128());
}

#[test]
fn respq_roundtrip_and_string_packing() {
    // resPQ#05162463 nonce server_nonce pq:string fingerprints:Vector<long>
    let value = ResPQ::ResPQ {
        nonce: nonce128(),
        server_nonce: Int128::default(),
        pq: b"qq".to_vec(), // короткая строка, упакованная в слово
        server_public_key_fingerprints: vec![0x1122_3344_5566_7788, 1],
    };
    let mut w = Writer::new();
    value.write_boxed(&mut w);
    let bytes = w.into_bytes();

    // Конструктор(1) + 2 нонса(8 слов) + строка "qq"(1) + вектор
    // (счётчик + 2 long = 5 слов) = 15 слов = 60 байт.
    assert_eq!(bytes.len(), 60);
    assert_eq!(&bytes[..4], &[0x63, 0x24, 0x16, 0x05]); // 0x05162463 LE
    // Строка: 02 71 71 00 (длина 2 + 'q','q').
    assert_eq!(&bytes[36..40], &[0x02, b'q', b'q', 0x00]);

    // Раундтрип.
    let mut r = Reader::new(&bytes);
    let parsed = ResPQ::read_boxed(&mut r).unwrap();
    assert_eq!(parsed, value);
    assert_eq!(r.get(), Err(tl_reader::Eof));
}

#[test]
fn p_q_inner_data_roundtrip() {
    let value = PQInnerData::PQInnerData {
        pq: b"qq".to_vec(),
        p: b"p".to_vec(),
        q: b"q".to_vec(),
        nonce: nonce128(),
        server_nonce: Int128::default(),
        new_nonce: Int256::default(),
    };
    let mut w = Writer::new();
    value.write_boxed(&mut w);
    let bytes = w.into_bytes();
    assert_eq!(&bytes[..4], &[0xec, 0x5a, 0xc9, 0x83]); // 0x83c95aec LE

    let mut r = Reader::new(&bytes);
    let parsed = PQInnerData::read_boxed(&mut r).unwrap();
    assert_eq!(parsed, value);
}

#[test]
fn long_string_roundtrip() {
    // server_DH_params_ok#d0e8075c ... encrypted_answer:string
    let long_answer: Vec<u8> = (0..300).map(|i| (i % 251) as u8).collect();
    let value = ServerDHParams::ServerDHParamsOk {
        nonce: nonce128(),
        server_nonce: Int128::default(),
        encrypted_answer: long_answer.clone(),
    };
    let mut w = Writer::new();
    value.write_boxed(&mut w);
    let bytes = w.into_bytes();

    // Заголовок long-строки: 254 в младшем байте, длина 300 в старших
    // (конструктор 1 + 2 нонса по 4 слова = смещение 36).
    let header = u32::from_le_bytes([bytes[36], bytes[37], bytes[38], bytes[39]]);
    assert_eq!(header, (300 << 8) | 254);

    let mut r = Reader::new(&bytes);
    let parsed = ServerDHParams::read_boxed(&mut r).unwrap();
    assert_eq!(parsed, value);
}

#[test]
fn flags_and_template_codegen() {
    // initConnection#c1cd5ea9 {X:Type} flags:# ... proxy:flags.0?InputClientProxy
    // ... query:!X = X; — из api.tl: проверяем текст генерата.
    let dir = std::env::temp_dir().join(format!("tl_gen_test_{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let scheme_path = dir.join("mini.tl");
    std::fs::write(
        &scheme_path,
        "inputClientProxy#54edf930 address:string port:int = InputClientProxy;\n\
         ---functions---\n\
         initConnection#c1cd5ea9 {X:Type} flags:# api_id:int device_model:string proxy:flags.0?InputClientProxy query:!X = X;\n",
    )
    .unwrap();
    let scheme = tl_generator::parse_files(&[scheme_path.to_str().unwrap()]).unwrap();
    let code = tl_generator::generate(&scheme, "mini");
    let _ = std::fs::remove_dir_all(&dir);

    // Конструктор по cleanline CRC (проверка алгоритма на этой строке).
    assert!(code.contains("INPUT_CLIENT_PROXY: u32 = 0x54edf930"), "{code}");
    assert!(code.contains("INIT_CONNECTION: u32 = 0xc1cd5ea9"), "{code}");
    // Флаги: u32-поле, условное поле Option, чтение/запись по биту.
    assert!(code.contains("proxy: Option<InputClientProxy>"));
    // Запись: сначала бит флага, затем значение (чтение функций
    // не генерируется — запросы только отправляются).
    assert!(code.contains("if self.flags & (1u32 << 0) != 0 {\n            if let Some(proxy) = &self.proxy {"));
    // Шаблонная функция: дженерик с трейтом, !X сериализуется boxed.
    assert!(code.contains("pub struct InitConnection<T>"));
    assert!(code.contains("impl<T: TlBoxedWrite> InitConnection<T>"));
    assert!(code.contains("query: T"));
    assert!(code.contains("query.tl_write_boxed(w);"));
}

#[test]
fn msgs_state_req_bare_vector() {
    // msgs_state_req#da69fb52 msg_ids:Vector<long> — bare-вектор:
    // счётчик + long'и, без конструктора вектора.
    let value = MsgsStateReq::MsgsStateReq {
        msg_ids: vec![1, 2, 3],
    };
    let mut w = Writer::new();
    value.write_boxed(&mut w);
    let bytes = w.into_bytes();
    // Конструктор(1) + счётчик(1) + 3 long(6 слов) = 8 слов = 32 байта.
    assert_eq!(bytes.len(), 32);
    assert_eq!(&bytes[..4], &[0x52, 0xfb, 0x69, 0xda]);
    // Счётчик = 3.
    assert_eq!(u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]), 3);

    let mut r = Reader::new(&bytes);
    let cons = r.get().unwrap();
    let parsed = MsgsStateReq::read_unboxed(&mut r, cons).unwrap();
    assert_eq!(parsed, value);
}

#[test]
fn future_salts_roundtrip() {
    // future_salts#ae500895 req_msg_id:long now:int salts:vector<future_salt>
    // — bare-элементы с собственным конструктором.
    let salt = FutureSalt::FutureSalt {
        valid_since: 100,
        valid_until: 200,
        salt: 0xdead_beef_dead_beef,
    };
    let value = FutureSalts::FutureSalts {
        req_msg_id: 7,
        now: 42,
        salts: vec![salt],
    };
    let mut w = Writer::new();
    value.write_boxed(&mut w);
    let bytes = w.into_bytes();

    let mut r = Reader::new(&bytes);
    let parsed = FutureSalts::read_boxed(&mut r).unwrap();
    assert_eq!(parsed, value);

    // Конструктор salts (1) + req_msg_id (2) + now (1) + счётчик (1)
    // + элемент (конструктор 1 + 2 + 2 = 5 слов) = 10 слов = 40 байт.
    assert_eq!(bytes.len(), 40);
}

#[test]
fn constructor_count_and_ids() {
    // Все конструкторы mtproto.tl распознаны, id соответствуют схеме.
    assert_eq!(ResPQ::RES_PQ, 0x0516_2463);
    assert_eq!(PQInnerData::P_Q_INNER_DATA, 0x83c9_5aec);
    assert_eq!(ReqPq::ID, 0x6046_9778);
}
