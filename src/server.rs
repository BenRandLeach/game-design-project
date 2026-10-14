use crate::GameState;
use bevy::prelude::*;
use std::{
    io::ErrorKind,
    net::{SocketAddr, UdpSocket},
};

pub struct ServerPlugin;
impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_server_socket)
            // app.add_systems(Update, server_listener);
            .add_systems(
                Update,
                server_listener.run_if(resource_exists::<ServerSocket>),
            )
            .add_systems(Update, do_send_button.run_if(in_state(GameState::Game)));
    }
}
#[derive(Resource)]
struct ServerSocket {
    socket: UdpSocket,
    client_addy: Option<SocketAddr>,
}
impl ServerSocket {
    fn new(socket: UdpSocket) -> Self {
        Self {
            socket,
            client_addy: None,
        }
    }
}
fn setup_server_socket(mut commands: Commands) {
    match UdpSocket::bind("0.0.0.0:7777") {
        Ok(socket) => {
            socket
                .set_nonblocking(true)
                .expect("couldnt set non blocking");
            println!("i am the host!");
            commands.insert_resource(ServerSocket::new(socket));
        }
        Err(e) => {
            println!("not the host, the port is taken: {e}");
        }
    }
    //let socket = UdpSocket::bind("0.0.0.0:7777").expect("couldn't bind");
    //socket.set_nonblocking(true).expect("couldn't set nonblocking");
    //commands.insert_resource(ServerSocket(socket));
}

fn server_listener(server: ResMut<ServerSocket>) {
    let mut buf = [0u8; 1024];
    loop {
        match server.socket.recv_from(&mut buf) {
            Ok((n, from)) => {
                server.client_addy = Some(from);
                println!("server got {n} bytes from {from}: {:?}", &buf[..n]);
                // echo back to whoever sent it
                if let Err(e) = server.socket.send_to(&buf[..n], from) {
                    println!("server couldnt reply: {e}");
                }
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => break, //empty
            Err(e) => {
                println!("recv error: {e}");
                break;
            }
        }
    }
}
fn do_send_button(
    buttons: Query<&Interaction, (Changed<Interaction>, With<SendButton>)>,
    server: Option<Res<ServerSocket>>,
    mut status: Single<&mut Text, With<statusText>>,
) {
    for interaction in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        status.0 = match &server {
            Some(s) => match s.client_addy {
                Some(addr) => match s.socket.send_to(b"hii", addr) {
                    Ok(n) => format!("sent {n} bytes"),
                    Err(e) => format!("failure to send : {e}"),
                },
                None => String::from("no client connected yet"),
            },
            None => String::from("not hosting"),
        };
    }
}
