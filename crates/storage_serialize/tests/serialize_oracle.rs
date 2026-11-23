//! Oracle for the storage buffer primitives: the Python writer produces the
//! bytes, the port must write the same ones and read them back to the same
//! values.

mod vectors;

use storage_serialize::date_time::SerializedDateTime;
use storage_serialize::sizes::{
    bytearray_size, bytes_size, color_size, date_time_size, string_size, LENGTH_SIZE,
};
use storage_serialize::{ByteArrayReader, ByteArrayWriter, Color, NULL_LENGTH};

fn oracle_payload() -> Vec<u8> {
    let mut writer = ByteArrayWriter::new();
    writer.write_u32(0x0102_0304);
    writer.write_i32(-123_456);
    writer.write_u64(0x0102_0304_0506_0708);
    writer.write_i64(-1_234_567_890_123);
    writer.write_bool(true);
    writer.write_string(Some(vectors::TEXT));
    writer.write_string(None);
    writer.write_byte_array(Some(&hex::decode(vectors::BYTE_ARRAY_HEX).unwrap()));
    writer.write_byte_array(Some(&[]));
    writer.write_byte_array(None);
    writer.write_bytes(&hex::decode(vectors::BYTES_VECTOR_HEX).unwrap());
    writer.write_bytes(&[]);
    writer.write_color(Color::new(0x12, 0x34, 0x56, 0x78));
    writer.write_date_time(SerializedDateTime {
        julian_day: vectors::JULIAN_DAY,
        msecs: vectors::MSECS,
        spec: vectors::DATE_TIME_SPEC,
    });
    writer.into_result()
}

#[test]
fn writer_reproduces_the_oracle_buffer() {
    assert_eq!(hex::encode(oracle_payload()), vectors::PAYLOAD_HEX);
}

#[test]
fn reader_parses_the_oracle_buffer() {
    let mut reader = ByteArrayReader::from_slice(&hex::decode(vectors::PAYLOAD_HEX).unwrap());

    assert_eq!(reader.read_u32(), Some(0x0102_0304));
    assert_eq!(reader.read_i32(), Some(-123_456));
    assert_eq!(reader.read_u64(), Some(0x0102_0304_0506_0708));
    assert_eq!(reader.read_i64(), Some(-1_234_567_890_123));
    assert_eq!(reader.read_bool(), Some(true));
    assert_eq!(reader.read_string().as_deref(), Some(vectors::TEXT));
    assert_eq!(reader.read_string().as_deref(), Some(""));
    assert_eq!(
        reader.read_byte_array(),
        Some(hex::decode(vectors::BYTE_ARRAY_HEX).unwrap())
    );
    assert_eq!(reader.read_byte_array(), Some(Vec::new()));
    assert_eq!(reader.read_byte_array(), Some(Vec::new()));
    assert_eq!(
        reader.read_bytes(),
        Some(hex::decode(vectors::BYTES_VECTOR_HEX).unwrap())
    );
    assert_eq!(reader.read_bytes(), Some(Vec::new()));
    assert_eq!(
        reader.read_color(),
        Some(Color::new(0x12, 0x34, 0x56, 0x78))
    );
    assert_eq!(
        reader.read_date_time(),
        Some(SerializedDateTime {
            julian_day: vectors::JULIAN_DAY,
            msecs: vectors::MSECS,
            spec: vectors::DATE_TIME_SPEC,
        })
    );
    assert!(reader.at_end());
    assert!(reader.is_ok());
}

#[test]
fn sizes_match_the_written_lengths() {
    assert_eq!(string_size(vectors::TEXT), vectors::STRING_SIZE);
    assert_eq!(
        bytearray_size(&hex::decode(vectors::BYTE_ARRAY_HEX).unwrap()),
        vectors::BYTEARRAY_SIZE
    );
    assert_eq!(
        bytes_size(&hex::decode(vectors::BYTES_VECTOR_HEX).unwrap()),
        vectors::BYTES_SIZE
    );
    assert_eq!(color_size(), 4);
    assert_eq!(date_time_size(), 13);
    assert_eq!(LENGTH_SIZE, 4);

    // The sizes are what the writer actually puts on the wire.
    let mut writer = ByteArrayWriter::new();
    writer.write_string(Some(vectors::TEXT));
    assert_eq!(writer.result().len(), vectors::STRING_SIZE);
    let mut writer = ByteArrayWriter::with_capacity(0);
    writer.write_date_time(SerializedDateTime::default());
    assert_eq!(writer.result().len(), date_time_size());
}

#[test]
fn empty_and_null_are_different_on_the_wire() {
    let mut writer = ByteArrayWriter::new();
    writer.write_byte_array(Some(&[]));
    writer.write_byte_array(None);
    writer.write_bytes(&[]);
    let bytes = writer.into_result();
    // An empty QByteArray is a zero length, a null one and an empty bytes
    // vector are both 0xFFFFFFFF.
    assert_eq!(bytes.len(), 12);
    assert_eq!(&bytes[0..4], &0u32.to_be_bytes());
    assert_eq!(&bytes[4..8], &NULL_LENGTH.to_be_bytes());
    assert_eq!(&bytes[8..12], &NULL_LENGTH.to_be_bytes());

    let mut reader = ByteArrayReader::new(bytes);
    assert_eq!(reader.read_byte_array(), Some(Vec::new()));
    assert_eq!(reader.read_byte_array(), Some(Vec::new()));
    assert_eq!(reader.read_bytes(), Some(Vec::new()));
    assert!(reader.is_ok());
}

#[test]
fn a_short_buffer_fails_quietly_and_stays_failed() {
    let mut reader = ByteArrayReader::from_slice(&hex::decode(vectors::TRUNCATED_HEX).unwrap());
    assert_eq!(reader.read_u32(), Some(0x0102_0304));
    assert_eq!(reader.read_i32(), Some(-123_456));
    assert_eq!(reader.read_u64(), Some(0x0102_0304_0506_0708));
    assert_eq!(reader.read_i64(), Some(-1_234_567_890_123));
    assert_eq!(reader.read_bool(), Some(true));
    // The string claims more bytes than are left: the value is the default
    // (an empty string) and the stream is bad from here on.
    assert_eq!(reader.read_string(), None);
    assert!(!reader.is_ok());
    assert_eq!(reader.read_u32(), None);
    assert_eq!(reader.read_string(), None);
    assert!(!reader.is_ok());
}

#[test]
fn a_missing_string_length_is_a_null_string() {
    let mut writer = ByteArrayWriter::new();
    writer.write_string(None);
    let mut reader = ByteArrayReader::new(writer.into_result());
    assert_eq!(reader.read_string().as_deref(), Some(""));
    assert!(reader.is_ok());
}

#[test]
fn odd_string_lengths_fail_the_read() {
    let mut writer = ByteArrayWriter::new();
    writer.write_u32(3);
    writer.write_raw(&[0x00, 0x61, 0x00]);
    let mut reader = ByteArrayReader::new(writer.into_result());
    assert_eq!(reader.read_string(), None);
    assert!(!reader.is_ok());
}

#[test]
fn color_packs_red_first() {
    let color = Color::new(0x12, 0x34, 0x56, 0x78);
    assert_eq!(color.packed(), vectors::COLOR_PACKED);
    assert_eq!(Color::from_packed(vectors::COLOR_PACKED), color);
    assert_eq!(Color::from_packed(0), Color::default());

    let mut writer = ByteArrayWriter::new();
    writer.write_color(color);
    assert_eq!(writer.result(), &vectors::COLOR_PACKED.to_be_bytes());
}

#[test]
fn date_time_is_thirteen_bytes_and_round_trips() {
    let value = SerializedDateTime::from_unix_millis(1_700_000_000_123);
    let mut writer = ByteArrayWriter::new();
    writer.write_date_time(value);
    assert_eq!(writer.result().len(), date_time_size());

    let mut reader = ByteArrayReader::new(writer.into_result());
    assert_eq!(reader.read_date_time(), Some(value));
    assert_eq!(value.unix_millis(), 1_700_000_000_123);
    // 2023-11-14T22:13:20.123Z falls on Julian day 2460263.
    assert_eq!(value.julian_day, 2_460_263);
    assert_eq!(value.msecs, 80_000_123);
    assert_eq!(value.spec, storage_serialize::LOCAL_TIME);
}

#[test]
fn raw_fixed_size_reads() {
    let mut writer = ByteArrayWriter::new();
    writer.write_raw(&[0xAB; 256]);
    let mut reader = ByteArrayReader::new(writer.into_result());
    assert_eq!(reader.read_raw::<256>(), Some([0xAB; 256]));
    assert!(reader.at_end());

    let mut short = ByteArrayReader::from_slice(&[1, 2, 3]);
    assert_eq!(short.read_raw::<4>(), None);
    assert!(!short.is_ok());
}