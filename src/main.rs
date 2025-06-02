use std::{collections::HashMap, net::SocketAddr, sync::{atomic::{AtomicU8, Ordering}, Arc}};

use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::{tcp::OwnedWriteHalf, TcpListener}};

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

#[tokio::main]
async fn main() {

    let sessions: HashMap<String, Session> = HashMap::new();
    let total_connections = Arc::new(AtomicU8::new(0));
    
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        let (mut read_socket, mut write_socket) = socket.into_split();
        total_connections.fetch_add(1, Ordering::AcqRel);
        println!("New Connection");
        
        let total_connections = Arc::clone(&total_connections);
        tokio::spawn(async move {
            let mut buffer = vec![0; 1024];
            loop {
                // TODO: fix msg_len is u8 -> max len is 256
                let mut msg_len = 0;
                loop {
                    let mut header: [u8; 3] = [0, 0, 0];
                    let Ok(_) = read_socket.read_exact(&mut header).await else {
                        println!("Connection Closed");
                        total_connections.fetch_sub(1, Ordering::AcqRel);
                        return;
                    };
                    
                    if header[0] == 0x25 && header[1] == 0x25 {
                        msg_len = header[2] as usize;
                        break;
                    }
                };

                let response_msg = {
                    let msg_buf = &mut buffer[..msg_len];
                    read_socket.read_exact(msg_buf).await.unwrap();
    
                    let msg = String::from_utf8(msg_buf.to_vec()).unwrap();
                    println!("Received msg: {}", &msg);
    
                    let total_conn = total_connections.load(Ordering::Acquire);
                    format!("[Total Connections: {total_conn}] Echo: {msg}")
                };

                buffer[0] = 0x25;
                buffer[1] = 0x25;
                buffer[2] = response_msg.len() as u8;
                response_msg.bytes().enumerate().for_each(|(i, byte)| buffer[3 + i] = byte);

                let response_msg_len = response_msg.len() + 3;
                let response_msg_buf = &buffer[..response_msg_len];
                let Ok(_) = write_socket.write_all(response_msg_buf).await else {
                    println!("Connection Closed");
                    total_connections.fetch_sub(1, Ordering::AcqRel);
                    return;
                };
            }
        });
    }
}
