//! This module manages the TCP server and how/where the packets are managed/sent.

use crate::config;
use crate::packet::{Packet, PacketId};
use log::{debug, error, warn};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::mpsc::error;

/// Global buffer size when allocating a new packet (in bytes).
const BUFFER_SIZE: usize = 1024;

/// Listens for every incoming TCP connection.
pub async fn listen() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::Settings::new();
    let server_address = format!("0.0.0.0:{}", config.server_port);
    let listener = TcpListener::bind(server_address).await?;

    loop {
        let (socket, addr) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, addr).await {
                warn!("Error handling connection from {addr}: {e}");
            }
        });
    }
}

/// State of each connection. (e.g.: handshake, play, ...)
#[derive(Debug, PartialEq)]
enum ConnectionState {
    Handshake,
    Status,
    Login,
    Play,
    Transfer,
}

impl ConnectionState {
    /// By default, the connection state is Handshaking.
    fn new() -> Self {
        Self::Handshake
    }
}

/// Allows correspondence between a Packet and a ConnectionState.
/// The meaning of a packet depends both on its packet ID and the current state of the connection.
///
/// # Usage
///
/// ```
/// let state = ConnectionState::new(); // This is of state ConnectionState::Handshake
///
/// // Let's assume we have a packet named `packet` of ID 0x00.
///
/// let new_state = ConnectionState::try_from_packet(state, &packet); // This will be Ok(ConnectionState::Login) if valid.
/// ```
impl ConnectionState {
    fn try_from_packet(last_state: Self, packet: &Packet) -> Result<Self, ConnectionStateError> {
        match last_state {
            ConnectionState::Handshake => {
                if packet.get_id().get_value() == 0x00 {
                    Ok(Self::Login)
                } else {
                    Err(ConnectionStateError::InvalidId)
                }
            }
            ConnectionState::Status => todo!(),
            ConnectionState::Login => todo!(),
            ConnectionState::Play => todo!(),
            ConnectionState::Transfer => todo!(),
        }
    }
}

#[derive(Error, Debug)]
pub enum ConnectionStateError {
    #[error("Failed to get correspondance between packet ID and connection state.")]
    InvalidId,
}

// TODO: Make a struct MinecraftServer. Make a connections: Arc Mutex Hashmap u32 Connection and next_id: u32.
// And the logic to listen for packets etc.
// Or think about some other design pattern to be the basis of our server to listen for packets.

/// Represents a TCP connection with a client.
struct Connection {
    state: ConnectionState,
    stream: TcpStream,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            state: ConnectionState::new(),
            stream,
        }
    }
}

/// Handles each connection
async fn handle_connection(
    mut socket: TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("New connection: {addr}");
    // TODO: Maybe have a bigger/dynamic buffer?
    let mut buf = [0; BUFFER_SIZE];

    loop {
        let n = socket.read(&mut buf).await?;
        if n == 0 {
            debug!("Connection closed: {addr}");
            return Ok(());
        }

        let response = handle_packet(&buf[..n]).await?;
        socket.write_all(&response).await?;
    }
}

/// Takes a packet buffer and returns a reponse.
async fn handle_packet(buffer: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    print!("\n\n\n"); // So that each logged packet is clearly visible.

    let packet = Packet::new(buffer)?;
    debug!("NEW PACKET ({}): {}", packet.len(), packet);

    let packet_id: PacketId = packet.get_id();
    debug!("PACKET ID: {}", packet_id.get_value());

    // create a response

    let mut response = Vec::new();
    response.extend_from_slice(b"Received: ");
    response.extend_from_slice(buffer);

    print!("\n\n\n");
    Ok(response)
}
