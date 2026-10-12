use std::io::Cursor;

use packet::{Decode, Encode};
use pretty_assertions::assert_eq;

#[test]
fn decode_i32_varint() {
    let cases: [(&[u8], i32); _] = [
        (&[0b0000_0000], 0),
        (&[0b0000_0001], 1),
        (&[0b0000_0010], 2),
        (&[0b0111_1111], 127),
        (&[0b1000_0000, 0b0000_0001], 128),
        (&[0b1001_0110, 0b0000_0001], 150),
        (&[0b1111_1111, 0b0000_0001], 255),
        (&[0b1101_1101, 0b1100_0111, 0b0000_0001], 25565),
        (&[0b1111_1111, 0b1111_1111, 0b0111_1111], 2_097_151),
        (
            &[
                0b1111_1111,
                0b1111_1111,
                0b1111_1111,
                0b1111_1111,
                0b0000_0111,
            ],
            2_147_483_647,
        ),
        (
            &[
                0b1111_1111,
                0b1111_1111,
                0b1111_1111,
                0b1111_1111,
                0b0000_1111,
            ],
            -1,
        ),
        (
            &[
                0b1000_0000,
                0b1000_0000,
                0b1000_0000,
                0b1000_0000,
                0b0000_1000,
            ],
            -2_147_483_648,
        ),
    ];

    for (bytes, expected) in cases {
        let mut cursor = Cursor::new(bytes);
        let actual = i32::decode(&mut cursor).unwrap();

        assert_eq!(actual, expected);
    }
}

#[test]
fn encode_i32_varint() {
    let cases: [(i32, &[u8]); _] = [
        (0, &[0b0000_0000]),
        (1, &[0b0000_0001]),
        (2, &[0b0000_0010]),
        (127, &[0b0111_1111]),
        (128, &[0b1000_0000, 0b0000_0001]),
        (150, &[0b1001_0110, 0b0000_0001]),
        (255, &[0b1111_1111, 0b0000_0001]),
        (25565, &[0b1101_1101, 0b1100_0111, 0b0000_0001]),
        (2_097_151, &[0b1111_1111, 0b1111_1111, 0b0111_1111]),
        (
            2_147_483_647,
            &[
                0b1111_1111,
                0b1111_1111,
                0b1111_1111,
                0b1111_1111,
                0b0000_0111,
            ],
        ),
        (
            -1,
            &[
                0b1111_1111,
                0b1111_1111,
                0b1111_1111,
                0b1111_1111,
                0b0000_1111,
            ],
        ),
        (
            -2_147_483_648,
            &[
                0b1000_0000,
                0b1000_0000,
                0b1000_0000,
                0b1000_0000,
                0b0000_1000,
            ],
        ),
    ];

    for (value, expected) in cases {
        let mut out = Vec::new();
        let written = value.encode(&mut out).unwrap();

        assert_eq!(out, expected);
        assert_eq!(written, expected.len());
    }
}

#[test]
fn encode_decode_roundtrip() {
    let cases: [i32; _] = [
        0,
        1,
        2,
        127,
        128,
        150,
        255,
        25565,
        2_097_151,
        2_147_483_647,
        -1,
        -2_147_483_648,
    ];

    for value in cases {
        let mut buf = Vec::new();
        let written = value.encode(&mut buf).unwrap();

        assert_eq!(written, buf.len());

        let mut cursor = Cursor::new(&buf[..]);
        assert_eq!(i32::decode(&mut cursor).unwrap(), value);
    }
}
