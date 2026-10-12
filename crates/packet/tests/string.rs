use std::io::Cursor;

use packet::{Decode, Encode};
use pretty_assertions::assert_eq;

#[test]
fn encode_decode_roundtrip() {
    let cases: [&str; _] = [
        "",
        "a",
        "mc.somewhere.com",
        "héllo wörld",
        "emoji: 🧱 block",
        "多バイト文字列",
    ];

    for value in cases {
        let mut buf = Vec::new();
        let written = value.encode(&mut buf).unwrap();

        assert_eq!(written, buf.len());

        let mut cursor = Cursor::new(&buf[..]);
        let actual: &str = Decode::decode(&mut cursor).unwrap();

        assert_eq!(actual, value);
    }
}

#[test]
fn decode_reads_varint_prefixed_utf8() {
    // "abc" encoded as length-prefixed string: 3, 'a', 'b', 'c'
    let bytes: &[u8] = &[0b0000_0011, b'a', b'b', b'c'];
    let mut cursor = Cursor::new(bytes);

    let actual: &str = Decode::decode(&mut cursor).unwrap();

    assert_eq!(actual, "abc");
}

#[test]
fn encode_prefixes_length_as_varint() {
    let mut buf = Vec::new();
    "abc".encode(&mut buf).unwrap();

    assert_eq!(buf, &[0b0000_0011, b'a', b'b', b'c']);
}

#[test]
fn decode_advances_cursor_past_string() {
    let bytes: &[u8] = &[0b0000_0011, b'a', b'b', b'c', 0b0000_0001];
    let mut cursor = Cursor::new(bytes);

    let actual: &str = Decode::decode(&mut cursor).unwrap();

    assert_eq!(actual, "abc");
    assert_eq!(cursor.position(), 4);
}
