use std::{
    fmt::Debug,
    io::{self, Cursor},
};

use thiserror::Error;

use crate::data_types::{
    ClientIntent, UShort, read_u16_short_be, string::read_string, varint::read_var_int,
};

pub const PROTOCOL_VERSION: i32 = 777;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ProtocolState {
    Handshaking,
    Status,
    Login,
    Configuration,
    Play,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketDirection {
    Serverbound, // client -> server
    Clientbound, // server -> client
}

/// Figure out the shape of the packets coming in
/// i.e. length of bytes, frame boundaries
fn process_packets_frames(packets: &[u8]) {
    // Iterate byte-by-byte

    // Figure out packet frame boundaries (start and end)

    // Decode packet ID from frame, rest is payload
}

/// Packet Format
///
/// Packets cannot be larger than 2^21 - 1 or 2,097,151 bytes (the
/// maximum that can be sent in a 3-byte VarInt). The length field
/// must not be longer than 3 bytes, even if the encoded value is
/// within the limit.
///
/// Unnecessarily long encodings at 3 bytes or below are still
/// allowed. For compressed packets, this applies to the Packet
/// Length field, i.e. the compressed length.
///
/// ## Without compression
/// Length    = VarInt     | Length of Packet ID + Data
/// Packet ID = VarInt     | `protocol_id` from server's packet report
/// Data      = Byte Array | Depends on connection state and packet ID
///
/// ## With compression
/// TODO for later
///
/// # References
/// https://minecraft.wiki/w/Java_Edition_protocol/Packets#Packet_Format
trait Packet {
    const OFFICIAL_NAME: &'static str;
}

struct DecodeError;

#[derive(Error, Debug)]
#[error("error processing incoming packets")]
pub enum PacketProcessingError {
    #[error("failed to process strings")]
    InvalidUTF8(#[from] std::str::Utf8Error),
    Io(#[from] io::Error),
    #[error("failed to process intent from request")]
    InvalidIntent,
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

enum PacketServerLogin {}
enum PacketServerConfig {}
enum PacketServerPlay {}

#[derive(Debug, PartialEq)]
struct PacketServerHandshake {
    protocol_version: i32,
    server_address: String,
    server_port: UShort,
    intent: ClientIntent,
}

/// The status can only be requested once, immediately after the
/// handshake, before any ping. The server won't respond otherwise.
///
/// `packet_id`: 0x0
/// TODO: idempotency on network side? server doesn't respond unless packet is received, before any ping
struct PacketServerStatusRequest;

type PacketID = i16;

// TODO: extract handshake packet ID, then parse payload into intermediate data types
fn process_packet_id(byte: u8, state: ProtocolState) -> Result<(), PacketProcessingError> {
    match state {
        ProtocolState::Handshaking => {
            // let packet = process_handshake_packet(&packets);
        }
        ProtocolState::Status => {}
        ProtocolState::Login => {}
        ProtocolState::Configuration => {}
        ProtocolState::Play => {}
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
pub(crate) struct ConnectionState {
    pub(crate) server_state: ProtocolState,
    pub(crate) client_intent: Option<ClientIntent>,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self {
            server_state: ProtocolState::Handshaking,
            client_intent: None,
        }
    }
}

/// Converts raw bytes from packet into intermediate data types
/// for server use.
///
/// This is done AFTER packet ID has been parsed on SERVERBOUND
/// direction.
fn process_handshake_packet(
    cursor: &mut Cursor<&[u8]>,
    connection_state: &mut ConnectionState,
) -> Result<PacketServerHandshake, PacketProcessingError> {
    let protocol_version = read_var_int(cursor);
    let strlen_in_bytes = read_var_int(cursor);
    let server_address = read_string(cursor, strlen_in_bytes as usize)?;
    let server_port = read_u16_short_be(cursor)?;
    let intent = read_var_int(cursor);
    let intent = ClientIntent::try_from(intent)?;

    connection_state.client_intent = Some(intent);
    connection_state.server_state = ProtocolState::Status;

    Ok(PacketServerHandshake {
        protocol_version,
        server_address: String::from(server_address),
        server_port,
        intent,
    })
}

fn process_status_request_packet() {}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use crate::data_types::varint::write_var_int;

    use super::*;

    // Status ping sequence - vertical slice
    // 1. C -> S: Handshake with Next State set to 1
    // 2. Client and Server set protocol state to Status
    // 3. ...
    //
    // https://minecraft.wiki/w/Java_Edition_protocol/FAQ#What_does_the_normal_status_ping_sequence_look_like?

    #[test]
    fn packet_handshake_server_should_be_processed() -> Result<(), Box<dyn std::error::Error>> {
        let server_address = String::from("mc.somewhere.com");
        let server_port: u16 = 25565;
        let intent: i32 = 0x01;

        let mut packets: Vec<u8> = Vec::new();
        write_var_int(PROTOCOL_VERSION, &mut packets)?;

        // Minecraft strings need prefixed VarInt for length
        write_var_int(i32::try_from(server_address.len())?, &mut packets)?;
        packets.write_all(server_address.as_bytes())?;

        packets.write_all(&server_port.to_be_bytes())?;
        write_var_int(intent, &mut packets)?;

        let mut actual_conn = ConnectionState {
            server_state: ProtocolState::Handshaking,
            client_intent: None,
        };

        let mut cursor = Cursor::new(packets.as_slice());

        let actual = process_handshake_packet(&mut cursor, &mut actual_conn).unwrap();

        let expected = PacketServerHandshake {
            protocol_version: 777,
            server_address,
            server_port: 25565,
            intent: ClientIntent::Status,
        };
        let expected_conn = ConnectionState {
            server_state: ProtocolState::Status,
            client_intent: Some(ClientIntent::Status),
        };

        assert_eq!(actual, expected);
        assert_eq!(actual_conn, expected_conn);
        Ok(())
    }

    #[test]
    fn packet_status_server_should_be_processed() {
        let connection = ConnectionState {
            server_state: ProtocolState::Status,
            client_intent: Some(ClientIntent::Status),
        };

        let mut has_status_request = false;

        // let actual = process_status_request_packet(&mut has_status_request);

        // CASE 1: not yet requested but has STATUS request -> accept, send STATUS response

        // CASE 2: has been requested, but same STATUS request -> ignore, don't send STATUS response

        // CASE 3: has been requested, then incoming PING request -> accept, send PONG response

        // CASE 4: has not been requested, but incoming PING request -> don't respond
    }
}
