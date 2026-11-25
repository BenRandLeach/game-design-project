use crate::client::{SendButton, statusText};
use crate::game::{Ammo, BANK_INCOME_PER_SEC, BankBlock, BeingPlaced, Block, BlockId, Slingshot,Stone};
use crate::{GameState, server};
use bevy::{prelude::*, transform};
use std::collections::VecDeque;
use std::{
    io::ErrorKind,
    net::{SocketAddr, UdpSocket},
};
pub struct ServerPlugin;
use bevy::time::common_conditions::on_timer;
use std::time::Duration;
#[derive(Resource, Default)]
pub struct Player1Address(pub Option<SocketAddr>);

#[derive(Resource, Default)]
pub struct Player2Address(pub Option<SocketAddr>);

#[derive(Resource, Default)]
pub struct ClientInput {
    // The key being handled this tick, and who pressed it
    pub input: u8,
    pub player: u8,
    // Keys waiting for a later tick, so presses from both players are never overwritten
    pub queue: VecDeque<(u8, u8)>,
}

#[derive(Resource, Default)]
pub struct Player1Money {
    pub money: u64,
}

#[derive(Resource, Default)]
pub struct Player2Money {
    pub money: u64,
}

#[derive(Resource, Default)]
pub struct Player1PowerUp {
    pub moneyPowerUp: bool,
    pub moneyPowerUpUsed: bool,
    pub moneyTimer: f32,
    pub attackSpeedPowerUp: bool,
    pub attackSpeedPowerUpUsed: bool,
}

#[derive(Resource, Default)]
pub struct Player2PowerUp {
    pub moneyPowerUp: bool,
    pub moneyPowerUpUsed: bool,
    pub moneyTimer: f32,
    pub attackSpeedPowerUp: bool,
    pub attackSpeedPowerUpUsed: bool,
}
#[derive(Resource)]
pub struct IsHost;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        // app.add_plugins(DefaultPlugins)
        app.add_systems(Startup, setup_server_socket)
            // app.add_systems(Update, server_listener);
            .add_systems(
                FixedUpdate,
                get_players
                    .run_if(resource_exists::<ServerSocket>.and_then(not(check_if_players_exist))),
            )
            .add_systems(
                FixedUpdate,
                server_listener
                    .run_if(resource_exists::<ServerSocket>.and_then(check_if_players_exist)),
            )
            .add_systems(
                FixedUpdate,
                passive_money.run_if(
                    resource_exists::<ServerSocket>
                        .and_then(check_if_players_exist)
                        .and_then(on_timer(Duration::from_secs(5))),
                ),
            )
            .add_systems(
                FixedUpdate,
                bank_income.run_if(
                    resource_exists::<ServerSocket>
                        .and_then(check_if_players_exist)
                        .and_then(on_timer(Duration::from_secs(1))),
                ),
            )
            .add_systems(
                FixedUpdate,
                turnOffP1MoneyPower.run_if(
                    resource_exists::<ServerSocket>
                    .and_then(check_if_players_exist)
                ),
            )
            .add_systems(
                FixedUpdate,
                turnOffP2MoneyPower.run_if(
                    resource_exists::<ServerSocket>
                    .and_then(check_if_players_exist)
                ),
            )
            .add_systems(
                FixedUpdate,
                do_send_money
                    .run_if(resource_exists::<ServerSocket>.and_then(check_if_players_exist)),
            )
            .add_systems(
                FixedUpdate,
                // After the whole physics chain, so moves/rotations made this tick are sent
                do_send_blockpos
                    .after(crate::physics::remove_lost_entities)
                    .run_if(resource_exists::<ServerSocket>.and_then(check_if_players_exist)),
            )
            .add_systems(
                FixedUpdate,
                // Every system has seen this tick's key; drop it so it can't be handled twice
                stop_input
                    .after(crate::physics::remove_lost_entities)
                    .run_if(resource_exists::<ServerSocket>),
            );
    }
}

#[derive(Resource)]
pub struct ServerSocket {
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

fn turnOffP1MoneyPower(mut p1Power:ResMut<Player1PowerUp>)
{
    if p1Power.moneyPowerUp==true
    {
        p1Power.moneyTimer-=0.0156;
        if p1Power.moneyTimer<0.0
        {
            p1Power.moneyPowerUp=false;
        }
    }
    
}

fn turnOffP2MoneyPower(mut p2Power:ResMut<Player2PowerUp>)
{
    if p2Power.moneyPowerUp==true
    {
        p2Power.moneyTimer-=0.0156;
        if p2Power.moneyTimer<0.0
        {
            p2Power.moneyPowerUp=false;
        }
    }
}

fn passive_money(mut p1: ResMut<Player1Money>, mut p2: ResMut<Player2Money>, p1Power:Res<Player1PowerUp>, p2Power:Res<Player2PowerUp>) {
    if p1Power.moneyPowerUp==true
    {
        *&mut p1.money = p1.money + 2;
    }
    else{
    *&mut p1.money = p1.money + 1;
    }
    if p2Power.moneyPowerUp==true
    {
        *&mut p2.money = p2.money + 2;
    }
    else{
        *&mut p2.money = p2.money + 1;
    }
    
}

// Each placed bank block pays its owner every second
fn bank_income(
    banks: Query<&Block, (With<BankBlock>, Without<BeingPlaced>)>,
    mut p1: ResMut<Player1Money>,
    mut p2: ResMut<Player2Money>,
    p1Power:Res<Player1PowerUp>, 
    p2Power:Res<Player2PowerUp>,
) {
    let mut p1multiplier=1;
    let mut p2multiplier=1;
    //println!("P1: {} P2: {}",p1Power.moneyPowerUp,p2Power.moneyPowerUp);
    if p1Power.moneyPowerUp==true{
        p1multiplier=2;
    }
    if p2Power.moneyPowerUp==true{
        p2multiplier=2;
    }
    for bank in &banks {
        match bank.playerNum {
            1 => p1.money += BANK_INCOME_PER_SEC*p1multiplier,
            2 => p2.money += BANK_INCOME_PER_SEC*p2multiplier,
            _ => {}
        }
    }
}

fn check_if_players_exist(
    mut player1: Res<Player1Address>,
    mut player2: Res<Player2Address>,
) -> bool {
    if player1.0.is_none() || player2.0.is_none() {
        return false;
    }
    return true;
}

fn get_players(
    mut server: ResMut<ServerSocket>,
    mut player1: ResMut<Player1Address>,
    mut player2: ResMut<Player2Address>,
) {
    let mut buf = [0u8; 1024];
    loop {
        match server.socket.recv_from(&mut buf) {
            Ok((n, from)) => {
                if player1.0 == Some(from) || player2.0 == Some(from) {
                    continue;
                }
                if player1.0.is_none() {
                    player1.0 = Some(from);
                    println!("Player 1 connected: {from}");
                } else if player2.0.is_none() {
                    player2.0 = Some(from);
                    println!("Player 2 connected: {from}");
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

fn setup_server_socket(mut commands: Commands) {
    match UdpSocket::bind("0.0.0.0:7777") {
        Ok(socket) => {
            socket
                .set_nonblocking(true)
                .expect("couldnt set non blocking");
            println!("i am the host!");
            commands.insert_resource(ServerSocket::new(socket));
            commands.insert_resource(IsHost);
        }
        Err(e) => {
            println!("not the host, the port is taken: {e}");
        }
    }
    //let socket = UdpSocket::bind("0.0.0.0:7777").expect("couldn't bind");
    //socket.set_nonblocking(true).expect("couldn't set nonblocking");
    //commands.insert_resource(ServerSocket(socket));
}

pub fn stop_input(mut input: ResMut<ClientInput>) {
    input.input = b'0';
    input.player = 0;
}

pub fn server_listener(
    mut server: ResMut<ServerSocket>,
    mut input: ResMut<ClientInput>,
    player1: Res<Player1Address>,
    player2: Res<Player2Address>,
) {
    let mut buf = [0u8; 1024];
    loop {
        match server.socket.recv_from(&mut buf) {
            Ok((n, from)) => {
                //server.client_addy = Some(from);

                //println!("server got {n} bytes from {from}: {:?}", &buf[..n]);
                // echo back to whoever sent it
                let key = buf[0];
                let player = if player1.0 == Some(from) {
                    1
                } else if player2.0 == Some(from) {
                    2
                } else {
                    0
                };
                // Held keys (A/D/S) arrive every frame; one queued copy per player is enough
                let held = matches!(key, b'A' | b'D' | b'S');
                if player != 0 && key != b'Q' && !(held && input.queue.contains(&(key, player))) {
                    input.queue.push_back((key, player));
                }
                if let Some(addr) = player1.0 {
                    if let Err(e) = server.socket.send_to(&buf[..n], addr) {
                        println!("server couldnt reply: {e}");
                    }
                }
                if let Some(addr2) = player2.0 {
                    if let Err(e) = server.socket.send_to(&buf[..n], addr2) {
                        println!("server couldnt reply: {e}");
                    }
                }
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => break, //empty
            Err(e) => {
                println!("recv error: {e}");
                break;
            }
        }
    }
    // Hand out one key per tick
    if let Some((key, player)) = input.queue.pop_front() {
        input.input = key;
        input.player = player;
    }
}

fn do_send_money(
    server: Res<ServerSocket>,
    player1: Res<Player1Address>,
    player2: Res<Player2Address>,
    p1: Res<Player1Money>,
    p2: Res<Player2Money>,
) {
    let msg = format!("{},{},{}", "$", 1, p1.money);
    let bytes = msg.as_bytes();
    if let Some(addr) = player1.0 {
        if let Err(e) = server.socket.send_to(bytes, addr) {
            println!("server couldnt p1 money to player 1: {e}");
        }
    }
    if let Some(addr2) = player2.0 {
        if let Err(e) = server.socket.send_to(bytes, addr2) {
            println!("server couldnt send p1 money to player 2: {e}");
        }
    }
    let msg = format!("{},{},{}", "$", 2, p2.money);
    let bytes = msg.as_bytes();
    if let Some(addr) = player1.0 {
        if let Err(e) = server.socket.send_to(bytes, addr) {
            println!("server couldnt send p2 money to player 1: {e}");
        }
    }
    if let Some(addr2) = player2.0 {
        if let Err(e) = server.socket.send_to(bytes, addr2) {
            println!("server couldnt send p2 money to player 2: {e}");
        }
    }
}

fn do_send_blockpos(
    server: Res<ServerSocket>,
    player1: Res<Player1Address>,
    player2: Res<Player2Address>,
    blocks: Query<
        (
            &BlockId,
            &Transform,
            Has<BankBlock>,
            Has<Slingshot>,
            Has<Ammo>,
            Has<Stone>,
        ),
        With<Block>,
    >,
) {
    for (id, transform, bank, slingshot, ammo,stone) in &blocks {
        // Kind tells clients which sprite to draw: 0 wood, 1 bank, 2 cannon, 3 cannonball
        let kind = if bank {
            1
        } else if slingshot {
            2
        } else if ammo {
            3
        } else if stone{
            4
        } 
        else {
            0
        };
        let pos = transform.translation;
        let rot = transform.rotation.to_scaled_axis().z;
        let msg = format!("{},{},{},{},{}", id.0, pos.x, pos.y, rot, kind);
        let bytes = msg.as_bytes();
        if let Some(addr) = player1.0 {
            if let Err(e) = server.socket.send_to(bytes, addr) {
                println!("server couldnt send block pos to player 1: {e}");
            }
        }
        if let Some(addr2) = player2.0 {
            if let Err(e) = server.socket.send_to(bytes, addr2) {
                println!("server couldnt send block pos to player 2: {e}");
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
