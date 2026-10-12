use packet_macros::{Decode, Encode};

#[derive(Debug, Decode, Encode)]
pub struct MyComplexPacket<'a> {
    a: i32,
    b: i32,
    c: &'a str
}
