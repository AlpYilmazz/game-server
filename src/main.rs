use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
};

use bytepack::{
    base::{ByteSize, ConstByteSize},
    pack::BytePack,
    unpack::ByteUnpack,
};
use bytepack_proc_macro::{BytePack, ByteSize, ByteUnpack, ConstByteSize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{tcp::OwnedWriteHalf, TcpListener, UdpSocket},
};

enum PlayerId {
    Player1 = 1,
    Player2 = 2,
}

struct PeerIdentity {
    pub id: PlayerId,
    pub name: String,
}

struct Peer {
    pub name: String,
    pub addr: SocketAddr,
    pub socket: OwnedWriteHalf,
}

struct Session {
    pub player1: Peer,
    pub player2: Peer,
}

struct PeerIdentityRequest {
    pub id: u8, // 1 or 2
    pub name: String,
}

const KEY_ACTION_PRESS: u8 = 0;
const KEY_ACTION_RELEASE: u8 = 1;

#[derive(ConstByteSize, ByteSize, BytePack)]
struct PlayerCommand {
    pub key: i32,
    pub action: u8,
}

#[derive(ConstByteSize, ByteSize, BytePack)]
struct ServerMessage {
    pub player_id: u8,
    pub frame_number: u64,
    pub command: PlayerCommand,
}

#[derive(ConstByteSize, ByteSize, BytePack, ByteUnpack)]
struct PlayerInput {
    pub buttons: u8,
    pub move_stick_x: u8,
    pub move_stick_y: u8,
}

#[derive(ConstByteSize, ByteSize, BytePack, ByteUnpack)]
struct InputPacket {
    pub buttons: u32, // 4
    pub num_inputs: u8, // 1
    pub inputs: [PlayerInput; 3], // 9
}

#[tokio::main]
async fn main() {
    let sessions: HashMap<String, Session> = HashMap::new();
    let total_connections = Arc::new(AtomicU8::new(0));

    // let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    let sock = Arc::new(UdpSocket::bind("127.0.0.1:8080").await.unwrap());
    loop {
        let mut buffer = vec![0; 1024];
        
        let (bytes_read, addr) = sock.recv_from(&mut buffer).await.unwrap();
        
        println!("Received");
        
        let packet_buf = buffer[..bytes_read].as_ref().to_vec();
        let sock = Arc::clone(&sock);
        tokio::spawn(async move {
            let packet_buf = packet_buf;

            // 2-7 frame delay
            let delay = (rand::random::<u64>() % 10) + 50;
            tokio::time::sleep(std::time::Duration::from_millis((delay as f32 * 16.6) as u64)).await;

            let Ok(_) = sock.send_to(&packet_buf, addr).await else {
                println!("Connection Closed [write]");
                return;
            };
            
            println!("Sent, [delay: {delay}]");
        });
        
        // let (socket, addr) = listener.accept().await.unwrap();
        // let (mut read_socket, mut write_socket) = socket.into_split();
        // total_connections.fetch_add(1, Ordering::AcqRel);
        // println!("New Connection");

        // let total_connections = Arc::clone(&total_connections);
        // tokio::spawn(async move {
        //     let mut buffer = vec![0; 1024];
        //     loop {
        //         let packet_buf = &mut buffer[..InputPacket::const_byte_size()];

        //         let Ok(_) = read_socket.read_exact(packet_buf).await else {
        //             println!("Connection Closed [read]");
        //             total_connections.fetch_sub(1, Ordering::AcqRel);
        //             return;
        //         };

        //         println!("Received");

        //         // 2-7 frame delay
        //         let delay = (rand::random::<u64>() % 5) + 2;
        //         tokio::time::sleep(std::time::Duration::from_millis((delay as f32 * 16.6) as u64)).await;

        //         let Ok(_) = write_socket.write_all(packet_buf).await else {
        //             println!("Connection Closed [write]");
        //             total_connections.fetch_sub(1, Ordering::AcqRel);
        //             return;
        //         };

        //         println!("Sent, [delay: {delay}]");
        //     }
        // });
    }
}
