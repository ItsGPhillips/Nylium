use packet_macros::{Decode, Encode};

#[derive(Debug, PartialEq, Decode, Encode)]
pub struct MyComplexPacket<'a> {
    a: i32,
    b: i32,
    c: &'a str,
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::{Decode, Encode};

    use super::*;

    #[test]
    fn complex_packet_encode_decode_roundtrip() {
        let packet = MyComplexPacket {
            a: 777,
            b: -1,
            c: "mc.somewhere.com",
        };

        let mut buf = Vec::new();
        let written = packet.encode(&mut buf).unwrap();

        // Fields are encoded in declaration order, each via its own
        // impl: `a` as a 2-byte VarInt, `b` as a max-length 5-byte
        // VarInt (negative values always are), and `c` as a
        // VarInt-prefixed UTF-8 string.
        assert_eq!(written, 2 + 5 + 1 + 16);
        assert_eq!(
            buf,
            vec![
                0b1000_1001, 0b0000_0110, // 777
                0b1111_1111, 0b1111_1111, 0b1111_1111, 0b1111_1111, 0b0000_1111, // -1
                0b0001_0000, // "mc.somewhere.com".len()
                b'm', b'c', b'.', b's', b'o', b'm', b'e', b'w', b'h', b'e', b'r', b'e', b'.', b'c', b'o', b'm',
            ]
        );

        let mut cursor = Cursor::new(&buf[..]);
        let decoded = MyComplexPacket::decode(&mut cursor).unwrap();

        assert_eq!(
            decoded,
            MyComplexPacket {
                a: 777,
                b: -1,
                c: "mc.somewhere.com",
            }
        );
        assert_eq!(cursor.position(), buf.len() as u64);
    }
}
