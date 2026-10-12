mod types;
mod defs;

use packet_macros::{Decode, Encode};
use std::io::{self, Cursor, Write};
use thiserror::Error;

pub trait Encode {
    fn encode(self, write: impl Write) -> io::Result<usize>;
}

// Decode MC data types into Rust datatypes.
pub trait Decode<'input>: Sized {
    fn decode(cursor: &mut Cursor<&'input [u8]>) -> Result<Self, DecodeError>;
}

#[derive(Error, Debug)]
#[error("Decode Error")]
pub enum DecodeError {
    Utf8(#[from] std::str::Utf8Error),
}
