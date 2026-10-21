use crate::GameState;
use bevy::prelude::*;
use bevy::{color::palettes::css::DARK_BLUE, window::WindowEvent::PanGesture};
use std::{
    io::ErrorKind,
    net::{SocketAddr, UdpSocket},
};

#[derive(Resource,Default)]
pub struct ServerInput{
    pub input: u8,
}

pub struct ClientPlugin;
impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Game), network_ui)
            .add_systems(
                FixedUpdate,
                do_send_button.run_if(resource_exists::<ClientSocket>),
            )
            .add_systems(
                FixedUpdate,
                client_listener.run_if(resource_exists::<ClientSocket>),
            );
    }
}

#[derive(Resource)]
struct ClientSocket {
    socket: UdpSocket,
    serv_addy: SocketAddr,
}
#[derive(Component)]
pub struct statusText;

#[derive(Component)]
pub struct SendButton;

pub fn stop_server_input(mut input: ResMut<ServerInput>)
{
    input.input=b'0';
}

pub fn connect(commands: &mut Commands, text: &str) -> String {
    let full = if text.contains(':') {
        text.to_string()
    } else {
        format!("{text}:7777")
    };
    let Ok(serv_addy) = full.parse::<SocketAddr>() else {
        return format!("'{text}' is not a valid IP");
    };
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(s) => s,
        Err(e) => return format!("couldnt open socket: {e}"),
    };
    socket.set_nonblocking(true).ok();
    commands.insert_resource(ClientSocket { socket, serv_addy });
    format!("Ready to talk to {serv_addy}")
}
//draws the send button and ststus line
fn network_ui(mut commands: Commands, client: Option<Res<ClientSocket>>) {
    let status_mess = match &client {
        Some(c) => format!("connected to {}", c.serv_addy),
        None => String::from("not connected"),
    };
    let parent = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: px(10),
                left: px(10),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                ..default()
            },
            DespawnOnExit(GameState::Game),
        ))
        .id();
    let send_button = commands
        .spawn((
            Button,
            SendButton,
            BackgroundColor(DARK_BLUE.into()),
        ))
        .id();
    //let send_label = commands.spawn(Text::new("Send")).id();
    let status = commands.spawn((Text::new(status_mess), statusText)).id();
}
fn do_send_button(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    client: Option<Res<ClientSocket>>,
    mut status: Single<&mut Text, With<statusText>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyB)
    {
        status.0 = match &client {
            Some(c) => match c.socket.send_to(b"B", c.serv_addy) {
                Ok(n) => format!("sent {n} bytes"),
                Err(e) => format!("failure to send : {e}"),
            },
            None => String::from("Not connected, enter IP on the start screen"),
        };
    }
    else if keyboard_input.pressed(KeyCode::KeyS)
    {
        status.0 = match &client {
            Some(c) => match c.socket.send_to(b"S", c.serv_addy) {
                Ok(n) => format!("sent {n} bytes"),
                Err(e) => format!("failure to send : {e}"),
            },
            None => String::from("Not connected, enter IP on the start screen"),
        };
    }
    else if keyboard_input.just_pressed(KeyCode::KeyR)
    {
        status.0 = match &client {
            Some(c) => match c.socket.send_to(b"R", c.serv_addy) {
                Ok(n) => format!("sent {n} bytes"),
                Err(e) => format!("failure to send : {e}"),
            },
            None => String::from("Not connected, enter IP on the start screen"),
        };
    }
    else if keyboard_input.pressed(KeyCode::KeyA)
    {
        status.0 = match &client {
            Some(c) => match c.socket.send_to(b"A", c.serv_addy) {
                Ok(n) => format!("sent {n} bytes"),
                Err(e) => format!("failure to send : {e}"),
            },
            None => String::from("Not connected, enter IP on the start screen"),
        };
    }
    else if keyboard_input.pressed(KeyCode::KeyD)
    {
        status.0 = match &client {
            Some(c) => match c.socket.send_to(b"D", c.serv_addy) {
                Ok(n) => format!("sent {n} bytes"),
                Err(e) => format!("failure to send : {e}"),
            },
            None => String::from("Not connected, enter IP on the start screen"),
        };
    }
    else if keyboard_input.just_pressed(KeyCode::KeyM)
    {
        status.0 = match &client {
            Some(c) => match c.socket.send_to(b"M", c.serv_addy) {
                Ok(n) => format!("sent {n} bytes"),
                Err(e) => format!("failure to send : {e}"),
            },
            None => String::from("Not connected, enter IP on the start screen"),
        };
    }
    else if keyboard_input.just_pressed(KeyCode::KeyN)
    {
        status.0 = match &client {
            Some(c) => match c.socket.send_to(b"N", c.serv_addy) {
                Ok(n) => format!("sent {n} bytes"),
                Err(e) => format!("failure to send : {e}"),
            },
            None => String::from("Not connected, enter IP on the start screen"),
        };
    }
    
}

fn client_listener(
    mut client: ResMut<ClientSocket>,
    mut status: Query<&mut Text, With<statusText>>,mut input: ResMut<ServerInput>
) {
    let mut buf = [0u8; 1024];
    loop {
        match client.socket.recv_from(&mut buf) {
            Ok((n, from)) => {
                let reply = String::from_utf8_lossy(&buf[..n]);
                input.input=buf[0];
                println!("client got {n} bytes from {from}: {:?}", &buf[..n]);
                for mut t in &mut status {
                    t.0 = format!("got a reply form {from}: {reply}");
                }
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => break,
            Err(e) => {
                println!("recv error: {e}");
                break;
            }
        }
    }
}
