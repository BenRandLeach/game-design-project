use crate::GameState;

use crate::game::BlockId;
use crate::server::server_listener;
use bevy::prelude::*;
use bevy::{color::palettes::css::DARK_BLUE, window::WindowEvent::PanGesture};
use std::{
    io::ErrorKind,
    net::{SocketAddr, UdpSocket},
};

#[derive(Resource, Default)]
pub struct ServerInput {
    pub input: u8,
}
#[derive(Resource, Default)]
pub struct Client1Money {
    pub money: u64,
}

#[derive(Resource, Default)]
pub struct Client2Money {
    pub money: u64,
}

pub struct ClientPlugin;
impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Game), network_ui)
        .add_systems(
                OnEnter(GameState::Game),start_connect.after(network_ui).run_if(resource_exists::<ClientSocket>),
            )
            .add_systems(
                // Not FixedUpdate: a frame can run 0 or 2 ticks, which drops or doubles key presses
                Update,
                do_send_button.run_if(resource_exists::<ClientSocket>),
            )
            .add_systems(
                FixedUpdate,
                // After the server, so the host gets its own echo on the same tick
                client_listener
                    .after(server_listener)
                    .run_if(resource_exists::<ClientSocket>),
            );
    }
}

#[derive(Component)]
struct MoneyText(u8);

#[derive(Resource)]
pub struct ClientSocket {
    socket: UdpSocket,
    serv_addy: SocketAddr,
}
#[derive(Component)]
pub struct statusText;

#[derive(Component)]
pub struct SendButton;

pub fn stop_server_input(mut input: ResMut<ServerInput>) {
    input.input = b'0';
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
//draws the send button and status line
fn network_ui(mut commands: Commands, client: Option<Res<ClientSocket>>,mut p1: ResMut<Client1Money>, 
    mut p2: ResMut<Client2Money>,) {
    *&mut p1.money=100;
    *&mut p2.money=100;
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
        .spawn((Button, SendButton, BackgroundColor(DARK_BLUE.into())))
        .id();
    //let send_label = commands.spawn(Text::new("Send")).id();
    let status = commands.spawn((Text::new(status_mess), statusText)).id();
}
fn start_connect(client: Option<Res<ClientSocket>>,
    mut status: Single<&mut Text, With<statusText>>,)
    {
        status.0 = match &client {
            Some(c) => match c.socket.send_to(b"Q", c.serv_addy) {
                Ok(n) => format!("sent {n} bytes"),
                Err(e) => format!("failure to send : {e}"),
            },
            None => String::from("Not connected, enter IP on the start screen"),
        };
        status.0 = match &client {
            Some(c) => match c.socket.send_to(b"Q", c.serv_addy) {
                Ok(n) => format!("sent {n} bytes"),
                Err(e) => format!("failure to send : {e}"),
            },
            None => String::from("Not connected, enter IP on the start screen"),
        };
    }
fn do_send_button(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    client: Option<Res<ClientSocket>>,
    mut status: Single<&mut Text, With<statusText>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyB) {
        status.0 = match &client {
            Some(c) => match c.socket.send_to(b"B", c.serv_addy) {
                Ok(n) => format!("sent {n} bytes"),
                Err(e) => format!("failure to send : {e}"),
            },
            None => String::from("Not connected, enter IP on the start screen"),
        };
    } else if keyboard_input.pressed(KeyCode::KeyS) {
        status.0 = match &client {
            Some(c) => match c.socket.send_to(b"S", c.serv_addy) {
                Ok(n) => format!("sent {n} bytes"),
                Err(e) => format!("failure to send : {e}"),
            },
            None => String::from("Not connected, enter IP on the start screen"),
        };
    } else if keyboard_input.just_pressed(KeyCode::KeyR) {
        status.0 = match &client {
            Some(c) => match c.socket.send_to(b"R", c.serv_addy) {
                Ok(n) => format!("sent {n} bytes"),
                Err(e) => format!("failure to send : {e}"),
            },
            None => String::from("Not connected, enter IP on the start screen"),
        };
    } else if keyboard_input.pressed(KeyCode::KeyA) {
        status.0 = match &client {
            Some(c) => match c.socket.send_to(b"A", c.serv_addy) {
                Ok(n) => format!("sent {n} bytes"),
                Err(e) => format!("failure to send : {e}"),
            },
            None => String::from("Not connected, enter IP on the start screen"),
        };
    } else if keyboard_input.pressed(KeyCode::KeyD) {
        status.0 = match &client {
            Some(c) => match c.socket.send_to(b"D", c.serv_addy) {
                Ok(n) => format!("sent {n} bytes"),
                Err(e) => format!("failure to send : {e}"),
            },
            None => String::from("Not connected, enter IP on the start screen"),
        };
    } else if keyboard_input.just_pressed(KeyCode::KeyM) {
        status.0 = match &client {
            Some(c) => match c.socket.send_to(b"M", c.serv_addy) {
                Ok(n) => format!("sent {n} bytes"),
                Err(e) => format!("failure to send : {e}"),
            },
            None => String::from("Not connected, enter IP on the start screen"),
        };
    } else if keyboard_input.just_pressed(KeyCode::KeyN) {
        status.0 = match &client {
            Some(c) => match c.socket.send_to(b"N", c.serv_addy) {
                Ok(n) => format!("sent {n} bytes"),
                Err(e) => format!("failure to send : {e}"),
            },
            None => String::from("Not connected, enter IP on the start screen"),
        };
    }
}

pub fn client_listener(
    mut commands: Commands,
    mut client: ResMut<ClientSocket>,
    mut status: Query<&mut Text, With<statusText>>,
    mut input: ResMut<ServerInput>,
    asset_server: Res<AssetServer>,
    mut blocks: Query<(&BlockId, &mut Transform)>,
    mut p1: ResMut<Client1Money>, 
    mut p2: ResMut<Client2Money>,

) {
    let mut buf = [0u8; 1024];
    let mut spawned_this_call: Vec<u32> = Vec::new();

    loop {
        match client.socket.recv_from(&mut buf) {
            Ok((n, from)) => {
                if n == 0 {
                    continue;
                }
                let first_byte = buf[0];
                if first_byte == b'$'
                {
                    let text = String::from_utf8_lossy(&buf[..n]);
                    let mut parts = text.split(',');
                    let (Some(moneySign_s), Some(playerNum_s), Some(moneyAmount_s)) =
                        (parts.next(), parts.next(), parts.next())
                    else {
                        continue;
                    };
                    let (Ok(moneySign), Ok(playerNum), Ok(moneyAmount)) = (
                        moneySign_s.trim().parse::<char>(),
                        playerNum_s.trim().parse::<u32>(),
                        moneyAmount_s.trim().parse::<u64>(),
                    ) else {
                        continue;
                    };
                    if playerNum==1
                    {
                        *&mut p1.money=moneyAmount;
                    }
                    else
                    {
                        *&mut p2.money=moneyAmount;
                    }
                }

                else if first_byte.is_ascii_digit() {
                    let text = String::from_utf8_lossy(&buf[..n]);
                    let mut parts = text.split(',');

                    let (Some(id_s), Some(x_s), Some(y_s), Some(rot_s)) =
                        (parts.next(), parts.next(), parts.next(), parts.next())
                    else {
                        continue;
                    };

                    let (Ok(id), Ok(x), Ok(y), Ok(rot)) = (
                        id_s.trim().parse::<u32>(),
                        x_s.trim().parse::<f32>(),
                        y_s.trim().parse::<f32>(),
                        rot_s.trim().parse::<f32>(),
                    ) else {
                        continue;
                    };

                    let mut found = false;
                    for (bid, mut transform) in &mut blocks {
                        if bid.0 == id {
                            transform.translation.x = x;
                            transform.translation.y = y;
                            transform.rotation = Quat::from_rotation_z(rot);
                            found = true;
                            break;
                        }
                    }

                    if !found && !spawned_this_call.contains(&id) {
                        commands.spawn((
                            Sprite {
                                image: asset_server.load("WoodBlock.png"),
                                //custom_size: Some(Vec2::new(16.0, 48.0)),
                                ..default()
                            },
                            BlockId(id),
                            Transform::from_xyz(x, y, 1.0)
                                .with_rotation(Quat::from_rotation_z(rot)),
                        ));
                        spawned_this_call.push(id);
                    }
                } else {
                    input.input = first_byte;
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
