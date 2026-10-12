use std::{io::ErrorKind, net::UdpSocket};
use bevy::prelude::*;

pub struct ClientPlugin;
impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_client_socket)
            .add_systems(Update, client_listener);
    }
}

#[derive(Resource)]
struct ClientSocket {
    socket: UdpSocket,
    sent_request: bool,
}

fn setup_client_socket(mut commands: Commands) {
    let socket = UdpSocket::bind("127.0.0.1:0").expect("couldn't bind client socket");
    socket.set_nonblocking(true).expect("couldn't set nonblocking");
    commands.insert_resource(ClientSocket { socket, sent_request: false });
}

fn client_listener(mut client: ResMut<ClientSocket>) {
    if !client.sent_request {
        let n = client.socket
            .send_to(b"hii", "127.0.0.1:7777")
            .expect("couldn't send to server");
        println!("sent {n} bytes to server");
        client.sent_request = true;
    }

    let mut buf = [0u8; 1024];
    match client.socket.recv_from(&mut buf) {
        Ok((n, from)) => {
            println!("got {n} bytes from {from}: {:?}", &buf[..n]);
        }
        Err(e) if e.kind() == ErrorKind::WouldBlock => {}
        Err(e) => panic!("recv error: {e}"),
    }
}
