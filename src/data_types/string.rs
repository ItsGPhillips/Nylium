use std::io::{BufRead, Cursor};

/// UTF-8 string prefixed with its size in bytes as a VarInt.
/// Maximum length of `n` characters, which varies by context.
///
/// The encoding used on the wire is regular UTF-8, not
/// Java's "slight modification".
///
/// The length of the string for purposes of the length limit is
/// its number of UTF-16 code units: scalar values > U+FFFF are
/// counted as two.
///
/// Up to `n * 3` bytes can be used to encode a UTF-8 string
/// comprising `n` code units when converted to UTF-16, and both
/// of those limits are checked.
///
/// Maximum `n` value is 32767. The +3 is due to the max size
/// of a valid length VarInt.
///
/// # Why +3 is max size of VarInt?
/// `+3` is the largest number of bytes needed to encode the
/// length prefix, not extra string data. The maximum allowed
/// string length is `n * 4`, with `n <= 32767`, so the largest
/// byte length is `32767 * 4 = 131068`, which fits within
/// 3 VarInt bytes because they carry 21 bits (up to 2,097,151)
pub fn read_string<'a>(
    bytes: &mut Cursor<&'a [u8]>,
    length_in_bytes: usize,
) -> Result<&'a str, std::str::Utf8Error> {
    let start = bytes.position() as usize;
    let end = start + length_in_bytes;

    let inner_ref: &'a [u8] = bytes.get_ref();
    let str = std::str::from_utf8(&inner_ref[start..end])?;

    bytes.consume(length_in_bytes);
    Ok(str)
}
