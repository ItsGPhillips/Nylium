use std::io::{BufRead, Cursor};

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
