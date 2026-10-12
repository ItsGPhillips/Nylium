use std::{
    fmt::Debug,
    io::{self, Cursor},
};

use thiserror::Error;

use crate::data_types::{
    ClientIntent, UShort, read_u8_byte, read_u16_short_be, string::read_string,
    varint::read_var_int,
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
pub enum PacketParseError {
    #[error("failed to process strings")]
    InvalidUTF8(#[from] std::str::Utf8Error),
    Io(#[from] io::Error),
    #[error("failed to process intent from request")]
    InvalidIntent,
    #[error("packet ID not known by server or unimplemented")]
    UnknownPacketId,
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl From<u8> for PacketParseError {
    fn from(value: u8) -> Self {
        todo!()
    }
}

#[derive(Debug, PartialEq)]
enum IncomingPacket {
    Handshake(ServerHandshakePacket),
    Status(ServerStatusPacket),
    Login(ServerLoginPacket),
    Config(ServerConfigPacket),
    Play(ServerPlayPacket),
}

#[derive(Debug, PartialEq)]
struct ServerHandshakePacket {
    protocol_version: i32,
    server_address: String,
    server_port: UShort,
    intent: ClientIntent,
}

#[derive(Debug, PartialEq)]
enum ServerStatusPacket {
    Status,
    Ping,
}

enum ServerStatusPacketId {
    Status = 0x00,
    Ping = 0x01,
}

impl TryFrom<u8> for ServerStatusPacketId {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::Status),
            0x01 => Ok(Self::Ping),
            other => Err(other),
        }
    }
}

#[derive(Debug, PartialEq)]
enum ServerLoginPacket {}

#[derive(Debug, PartialEq)]
enum ServerConfigPacket {}

#[derive(Debug, PartialEq)]
enum ServerPlayPacket {}

/// The status can only be requested once, immediately after the
/// handshake, before any ping. The server won't respond otherwise.
///
/// `packet_id`: 0x0
/// TODO: idempotency on network side? server doesn't respond unless packet is received, before any ping
struct PacketServerStatusRequest;

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

#[allow(clippy::too_many_lines)]
fn parse_incoming_packet(
    raw_bytes: &[u8],
    connection_state: &mut ConnectionState,
) -> Result<IncomingPacket, PacketParseError> {
    let ConnectionState {
        server_state,
        client_intent,
    } = connection_state;

    let packet_id = raw_bytes[0];
    let packet_payload = &raw_bytes[1..];
    let mut cursor = Cursor::new(packet_payload);

    match server_state {
        ProtocolState::Handshaking => match packet_id {
            0x00 => {
                let data = parse_handshake_packet_payload(&mut cursor, connection_state)?;
                Ok(IncomingPacket::Handshake(data))
            }
            _ => Err(PacketParseError::UnknownPacketId),
        },
        ProtocolState::Status => {
            let packet_id = ServerStatusPacketId::try_from(packet_id)?;

            match packet_id {
                ServerStatusPacketId::Status => {
                    send_status_response_packet();
                    Ok(IncomingPacket::Status(ServerStatusPacket::Status))
                }
                ServerStatusPacketId::Ping => todo!(),
            }
        }
        ProtocolState::Login => todo!(),
        ProtocolState::Configuration => todo!(),
        ProtocolState::Play => todo!(),
    }
}

trait PacketSender {
    fn send();
}

struct ServerResponsePacketSender {}

impl PacketSender for ServerResponsePacketSender {
    fn send() {}
}

fn send_status_response_packet() {}

/// Converts raw bytes from packet into intermediate data types
/// for server use.
///
/// This is done AFTER packet ID has been parsed on SERVERBOUND
/// direction.
fn parse_handshake_packet_payload(
    cursor: &mut Cursor<&[u8]>,
    connection_state: &mut ConnectionState,
) -> Result<ServerHandshakePacket, PacketParseError> {
    let protocol_version = read_var_int(cursor);
    let strlen_in_bytes = read_var_int(cursor);
    let server_address = read_string(cursor, strlen_in_bytes as usize)?;
    let server_port = read_u16_short_be(cursor)?;
    let intent = read_var_int(cursor);
    let intent = ClientIntent::try_from(intent)?;

    connection_state.client_intent = Some(intent);
    connection_state.server_state = ProtocolState::Status;

    Ok(ServerHandshakePacket {
        protocol_version,
        server_address: String::from(server_address),
        server_port,
        intent,
    })
}

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
    fn packet_server_handshake_payload_should_be_processed()
    -> Result<(), Box<dyn std::error::Error>> {
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

        let actual = parse_handshake_packet_payload(&mut cursor, &mut actual_conn).unwrap();

        let expected = ServerHandshakePacket {
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

    // We're only handling packet processing individually,
    // not the sequencing yet

    // Status ping sequence - vertical slice
    // 2. ...
    // 3. C -> S: Status Request
    // 4. S -> C: Status Response
    //
    // https://minecraft.wiki/w/Java_Edition_protocol/FAQ#What_does_the_normal_status_ping_sequence_look_like?
    #[test]
    fn packet_server_status_should_be_processed() -> Result<(), Box<dyn std::error::Error>> {
        let default_conn = ConnectionState::default();

        // CASE 0: in handshaking state, should not accept STATUS request

        let mut connection = ConnectionState {
            server_state: ProtocolState::Status,
            client_intent: Some(ClientIntent::Status),
        };

        let packets: Vec<u8> = vec![0x0];

        let actual = parse_incoming_packet(&packets, &mut connection)?;

        assert_eq!(actual, IncomingPacket::Status(ServerStatusPacket::Status));

        // CASE 1: not yet requested but has STATUS request -> accept, send STATUS response

        // CASE 2: has been requested, but same STATUS request -> ignore, don't send STATUS response

        // CASE 3: has been requested, then incoming PING request -> accept, send PONG response

        // CASE 4: has not been requested, but incoming PING request -> don't respond

        // CASE 5: has done handshake but incorrect packet ID -> don't respond
        Ok(())
    }
}
