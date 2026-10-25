use crate::GameState;
use crate::client::{SendButton, statusText};
use bevy::prelude::*;
use std::{
    io::ErrorKind,
    net::{SocketAddr, UdpSocket},
};
pub struct ServerPlugin;


#[derive(Resource, Default)]
pub struct Player1Address(pub Option<SocketAddr>);


#[derive(Resource, Default)]
pub struct Player2Address(pub Option<SocketAddr>);

#[derive(Resource,Default)]
pub struct ClientInput{
    pub input: u8,
}

#[derive(Resource,Default)]
pub struct Player1Money{
    pub money: u64,
}

#[derive(Resource,Default)]
pub struct Player2Money{
    pub money: u64,
}



impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_server_socket)
            // app.add_systems(Update, server_listener);
            .add_systems(FixedUpdate,get_players)
            .add_systems(
                FixedUpdate,
                server_listener.run_if(resource_exists::<ServerSocket>.and_then(resource_exists::<Player1Address>).and_then(resource_exists::<Player2Address>)),
            );
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

fn get_players(mut server: ResMut<ServerSocket>,mut player1: ResMut<Player1Address>,mut player2:ResMut<Player2Address>)
{
    let mut buf = [0u8; 1024];
    loop {
        match server.socket.recv_from(&mut buf) {
            Ok((n, from)) => {
                if player1.0.is_none()
                {
                    player1.0=Some(from);
                }
                else if player2.0.is_none()
                {
                    player2.0=Some(from);
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

pub fn stop_input(mut input: ResMut<ClientInput>)
{
    input.input=b'0';
}


pub fn server_listener(mut server: ResMut<ServerSocket>,mut input: ResMut<ClientInput>,player1: Res<Player1Address>,player2:Res<Player2Address>){
    let mut buf = [0u8; 1024];
    loop {
        match server.socket.recv_from(&mut buf) {
            Ok((n, from)) => {
                //server.client_addy = Some(from);
                
                //println!("server got {n} bytes from {from}: {:?}", &buf[..n]);
                // echo back to whoever sent it
                input.input=buf[0];
                if let Some(addr) = player1.0{
                if let Err(e) = server.socket.send_to(&buf[..n], addr) {
                    println!("server couldnt reply: {e}");
                }}
                if let Some(addr) = player2.0{
                if let Err(e) = server.socket.send_to(&buf[..n], addr) {
                    println!("server couldnt reply: {e}");
                }}
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => break, //empty
            Err(e) => {
                println!("recv error: {e}");
                break;
            }
        }
    }
}
/*fn do_send_button(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    server: Option<Res<ServerSocket>>,
    mut status: Single<&mut Text, With<statusText>>,mut input: ResMut<ClientInput>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyB)
    {
        input.input=b'B';
        status.0 = match &server {
            Some(s) => match s.client_addy {
                Some(addr) => match s.socket.send_to(b"B", addr) {
                    Ok(n) => format!("sent {n} bytes"),
                    Err(e) => format!("failure to send : {e}"),
                },
                None => String::from("no client connected yet"),
            },
            None => String::from("not hosting"),
        };
    }
    if keyboard_input.just_pressed(KeyCode::KeyR)
    {
        input.input=b'R';
        status.0 = match &server {
            Some(s) => match s.client_addy {
                Some(addr) => match s.socket.send_to(b"R", addr) {
                    Ok(n) => format!("sent {n} bytes"),
                    Err(e) => format!("failure to send : {e}"),
                },
                None => String::from("no client connected yet"),
            },
            None => String::from("not hosting"),
        };
    }
    if keyboard_input.just_pressed(KeyCode::KeyS)
    {
        input.input=b'S';
        status.0 = match &server {
            Some(s) => match s.client_addy {
                Some(addr) => match s.socket.send_to(b"S", addr) {
                    Ok(n) => format!("sent {n} bytes"),
                    Err(e) => format!("failure to send : {e}"),
                },
                None => String::from("no client connected yet"),
            },
            None => String::from("not hosting"),
        };
    }
}*/
