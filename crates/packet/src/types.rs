use std::io::{self, Cursor, Write};

use crate::{Decode, DecodeError, Encode};

const MAX_VARINT_LENGTH: usize = 5;

impl Encode for i32 {
    /// Writes `self` as a VarInt: the 7 least significant bits are
    /// encoded per byte and the most significant bit indicates whether
    /// another byte follows. The least significant group is written
    /// first. VarInts are never longer than 5 bytes.
    ///
    /// # References
    /// <https://minecraft.wiki/w/Java_Edition_protocol/Packets#VarInt_and_VarLong>
    /// <https://protobuf.dev/programming-guides/encoding/#varints>
    fn encode(self, mut write: impl Write) -> io::Result<usize> {
        // u32 makes right shifts fill with zeroes
        let mut val = self.cast_unsigned();
        let mut bytes_written: usize = 0;

        for _ in 0..MAX_VARINT_LENGTH {
            let mut data = (val & 0b0111_1111) as u8;
            val >>= 7;

            // If there is data, toggle first bit
            // this helps parser indicate next byte in LE exists
            if val != 0 {
                data |= 0b1000_0000;
            }

            write.write_all(&[data])?;
            bytes_written += 1;

            if val == 0 {
                break;
            }
        }
        Ok(bytes_written)
    }
}

impl Decode<'_> for i32 {
    fn decode(cursor: &mut Cursor<&'_ [u8]>) -> Result<Self, DecodeError> {
        use std::io::BufRead;

        let mut val: i32 = 0;
        for i in 0..MAX_VARINT_LENGTH {
            let offset = cursor.position() as usize;
            let remaining = &cursor.get_ref()[offset..];

            if let Some(byte) = remaining.first() {
                // save cursor in BE form
                val |= (i32::from(*byte) & 0b0111_1111) << (7 * i);

                // Mask first bit to check for next byte
                // if 0 we reached the end of this payload
                if byte & 0b1000_0000 == 0 {
                    cursor.consume(1);
                    break;
                }
            }

            cursor.consume(1);
        }
        Ok(val)
    }
}

impl Encode for &str {
    fn encode(self, mut write: impl Write) -> io::Result<usize> {
        let len = i32::try_from(self.len())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "string too long"))?;
        let mut written = len.encode(&mut write)?;
        write.write_all(self.as_bytes())?;
        written += self.len();
        Ok(written)
    }
}

impl<'a> Decode<'a> for &'a str {
    fn decode(cursor: &mut Cursor<&'a [u8]>) -> Result<Self, DecodeError> {
        use std::io::BufRead;

        let str_size = i32::decode(cursor)? as usize;

        let start = cursor.position() as usize;
        let end = start + str_size;

        let inner_ref: &'a [u8] = cursor.get_ref();
        let str = std::str::from_utf8(&inner_ref[start..end])?;

        cursor.consume(str_size);
        Ok(str)
    }
}
