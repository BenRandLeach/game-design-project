use bevy::prelude::*;
/* Artificial lag imports
use std::thread;
use std::time::Duration;
*/
use crate::server::IsHost;
use crate::side::Side;
use crate::{GameState, WIN_H};
const SLIDE_SIZE: Vec2 = Vec2::new(1280.0, 720.0);

use crate::client::Client1Money;
use crate::client::Client2Money;
use crate::physics::AngularVelocity;
use crate::physics::Velocity;
use crate::server::ClientInput;
use crate::server::{Player1Money, Player2Money};

const STARTING_MONEY: u64 = 100;
const BLOCK_COST: u64 = 1;
const BANK_COST: u64 = 15;
const BANK_MASS: f32 = 1.5;
const STONE_COST: u64 = 5;
const CANNON_COST: u64 = 20;
const SLINGSHOT_COST: u64 = 15;
pub const BANK_INCOME_PER_SEC: u64 = 1;
const GROUND_Y: f32 = -WIN_H / 2.0;
const TOWER_BASE_Y: f32 = (-WIN_H / 2.0) + 20.0;
// W: 16, H: 48 for OG Block
// W: 96, H: 24 for OG block
const BLOCK_WIDTH: f32 = 96.0;
const BLOCK_HEIGHT: f32 = 24.0;

const LAUNCH_SPEED: Vec2 = Vec2::new(800.0, 500.0);
pub const CANNONBALL_SPRITE_SIZE: Vec2 = Vec2::new(250.0, 150.0);
const MUZZLE_OFFSET: f32 = 36.0; // Past the slingshot's hitbox, or the shot collides with its own slingshot

#[derive(Component)]
pub struct StaticBody;

#[derive(Component)]
pub struct Block {
    pub mass: f32,
    pub angular_velocity: f32,
    pub inertia: f32,
    pub fcoeff: f32,
    pub playerNum: u8,
    pub rotated: bool,
}

#[derive(Component)]
pub struct Slingshot;

// Marks a bank block: heavier, square, and earns its owner money every second
#[derive(Component)]
pub struct BankBlock;

#[derive(Component)]
pub struct BeingPlaced;

#[derive(Component)]
pub struct JustDropped;

#[derive(Component)]
pub struct NewEntity;

#[derive(Resource, Default)]
pub struct CurrentlyActive(pub Option<Entity>);

#[derive(Component)]
pub struct Ammo;

#[derive(Message, Default)]
pub struct Game;

#[derive(Component)]
pub struct GameScreen;

#[derive(Resource)]
pub struct GameScreenImage(Handle<Image>);

#[derive(Resource, Default)]
pub struct NextBlockId(pub u32);

#[derive(Component, Clone, Copy)]
pub struct BlockId(pub u32);

#[derive(Component)]
struct MoneyUi;

#[derive(Component)]
struct MoneyText(u8);

#[derive(Component)]
pub struct ShopBackground;
#[derive(Clone, Copy, PartialEq)]
pub enum BlockKind {
    Wood,
    Stone,
    Bank,
    Cannon,
    Slingshot,
}

#[derive(Resource)]
pub struct SelectedBlock {
    pub p1: BlockKind,
    pub p2: BlockKind,
}

impl Default for SelectedBlock {
    fn default() -> Self {
        SelectedBlock {
            p1: BlockKind::Wood,
            p2: BlockKind::Wood,
        }
    }
}
#[derive(Resource, Default)]
pub struct ShopOpen(pub bool);
pub struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_game)
            .add_systems(OnEnter(GameState::Game), setup_game)
            .add_systems(OnEnter(GameState::Game), (reset_money, spawn_money_ui))
            .add_systems(Update, game_mess_listener)
            .add_systems(Update, swap_to_credits.run_if(in_state(GameState::Game)))
            .add_systems(Update, update_money_ui.run_if(in_state(GameState::Game)))
            .add_systems(
                FixedUpdate,
                (
                    apply_selection,
                    spawn_block,
                    spawn_slingshot,
                    launch_cannonball,
                )
                    .chain() // apply_selection first, so spawn_block sees the updated slot
                    .after(crate::server::server_listener)
                    .after(crate::client::client_listener)
                    .before(crate::physics::fast_fall)
                    .run_if(in_state(GameState::Game))
                    .run_if(resource_exists::<IsHost>),
            )
            .add_systems(Update, toggle_shop.run_if(in_state(GameState::Game)))
            .add_systems(
                OnExit(GameState::Game),
                crate::startscreen::despawn_with::<MoneyUi>,
            )
            .add_systems(Update, shop_click.run_if(in_state(GameState::Game)))
            //.add_systems(Update, move_blocks.run_if(in_state(GameState::Game)))
            //.add_systems(Update, simulate_lag.run_if(in_state(GameState::Game)))
            .insert_resource(CurrentlyActive::default())
            .insert_resource(ShopOpen::default())
            .insert_resource(SelectedBlock::default())
            .add_message::<Game>();
    }
}
/* For testing
fn simulate_lag() {
    thread::sleep(Duration::from_millis(200));
}
*/
fn spawn_money_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let coin = asset_server.load("Coin.png");
    for (player, left_side) in [(1u8, true), (2u8, false)] {
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    top: px(50),
                    left: if left_side { px(16) } else { auto() },
                    right: if left_side { auto() } else { px(16) },
                    align_items: AlignItems::Center,
                    column_gap: px(8),
                    ..default()
                },
                MoneyUi,
            ))
            .with_children(|parent| {
                parent.spawn((
                    ImageNode::new(coin.clone()),
                    Node {
                        width: px(32),
                        height: px(32),
                        ..default()
                    },
                ));
                parent.spawn((
                    Text::new(STARTING_MONEY.to_string()),
                    TextFont {
                        font_size: FontSize::Px(28.0),
                        ..default()
                    },
                    MoneyText(player),
                ));
            });
    }
}
fn update_money_ui(
    p1: Res<Client1Money>,
    p2: Res<Client2Money>,
    mut texts: Query<(&mut Text, &MoneyText)>,
) {
    for (mut text, who) in &mut texts {
        let amount = if who.0 == 1 { p1.money } else { p2.money };
        text.0 = amount.to_string();
    }
}
fn game_mess_listener(
    mut game_mess: MessageReader<Game>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if !game_mess.is_empty() {
        next_state.set(GameState::Game);
        game_mess.clear();
    }
}

fn load_game(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    //mut loading_assets: ResMut<LoadingAssets>,
) {
    //let game_texture_handle = asset_server.load("Tower_Balance_Background.png");

    //loading_assets.0.push(game_texture_handle.clone().untyped());
    //commands.insert_resource(GameScreenImage(game_texture_handle));
}

fn move_blocks(
    input: Res<ButtonInput<KeyCode>>,
    mut active: Query<(&mut Transform, &mut Velocity), With<NewEntity>>,
) {
    let mut vel = Vec3::ZERO;
    if input.pressed(KeyCode::KeyA) {
        vel.x -= 1.0;
    }

    if input.pressed(KeyCode::KeyD) {
        vel.x += 1.;
    }

    if let Ok((mut transform, mut velocity)) = active.single_mut() {
        transform.translation += vel;
        velocity.x = vel.x;
    }
    //for mut block_transform in &mut blocks {
    //    block_transform.translation += vel; }
}

fn spawn_block(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut currently_active: ResMut<CurrentlyActive>,
    being_placed: Query<Entity, With<BeingPlaced>>,
    client_data: Res<ClientInput>,
    mut next_id: ResMut<NextBlockId>,
    mut p1_money: ResMut<Player1Money>,
    mut p2_money: ResMut<Player2Money>,
    selected: Res<SelectedBlock>,
) {
    if being_placed.iter().count() > 0 {
        return;
    }
    if client_data.input != b'B' {
        return;
    }
    let player = client_data.player;
    let Some(side) = Side::for_player(player) else {
        return;
    };
    let kind = match player {
        1 => selected.p1,
        _ => selected.p2,
    };

    let cost = match kind {
        BlockKind::Wood => BLOCK_COST,
        BlockKind::Stone => STONE_COST,
        BlockKind::Bank => BANK_COST,
        BlockKind::Cannon => CANNON_COST,
        BlockKind::Slingshot => SLINGSHOT_COST,
    };

    let wallet = match player {
        1 => &mut p1_money.money,
        _ => &mut p2_money.money,
    };
    if *wallet < cost {
        return;
    }
    *wallet -= cost;

    if kind == BlockKind::Cannon {
        let id = next_id.0;
        next_id.0 += 1;
        commands.spawn((
            Sprite {
                image: asset_server.load("MPCannon.png"),
                ..default()
            },
            Slingshot,
            Block {
                mass: 1.0,
                angular_velocity: 0.0,
                inertia: 0.0,
                playerNum: player,
                fcoeff: 0.75,
                rotated: false,
            },
            BlockId(id),
            JustDropped,
            Velocity::default(),
            AngularVelocity(0.),
            side,
            Transform::from_xyz(side.base_center(), 275., 2.),
        ));
        return;
    }

    let is_bank = kind == BlockKind::Bank;

    if let Some(active_entity) = currently_active.0 {
        commands.entity(active_entity).remove::<NewEntity>();
    }
    let id = next_id.0;
    next_id.0 += 1;
    let mass = if is_bank { BANK_MASS } else { 1.0 };
    let size = crate::physics::block_half_size(is_bank) * 2.0;
    let inertia = mass * ((size.x * size.x) + (size.y * size.y)) / 12.0;

    let image = match kind {
        BlockKind::Bank => "Bank_Block.png",
        BlockKind::Stone => "StoneBlock.png",
        BlockKind::Slingshot => "slingshot.png",
        _ => "MPWoodB.png",
    };

    let mut entity = commands.spawn((
        Sprite {
            image: asset_server.load(image),
            custom_size: is_bank.then_some(size),
            ..default()
        },
        Block {
            mass,
            angular_velocity: 0.0,
            inertia,
            playerNum: player,
            fcoeff: 0.25,
            rotated: false,
        },
        BlockId(id),
        BeingPlaced,
        Velocity(Vec2::new(0., 0.)),
        AngularVelocity(0.),
        side,
        Transform::from_xyz(side.base_center(), 275., 1.),
    ));
    if is_bank {
        entity.insert(BankBlock);
    }
    currently_active.0 = Some(entity.id());
}
fn toggle_shop(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut shop_open: ResMut<ShopOpen>,
    shop_query: Query<Entity, With<ShopBackground>>,
) {
    if !keyboard_input.just_pressed(KeyCode::KeyV) {
        return;
    }

    if shop_open.0 {
        for entity in &shop_query {
            commands.entity(entity).despawn();
        }
        shop_open.0 = false;
    } else {
        commands.spawn((
            Sprite {
                image: asset_server.load("shopbackground.png"),
                ..default()
            },
            ShopBackground,
            Transform::from_xyz(0.0, 0.0, 5.0),
        ));
        shop_open.0 = true;
    }
}
fn apply_selection(client_data: Res<ClientInput>, mut selected: ResMut<SelectedBlock>) {
    let kind = match client_data.input {
        b'w' => BlockKind::Wood,
        b't' => BlockKind::Stone,
        b'k' => BlockKind::Bank,
        b'j' => BlockKind::Cannon,
        b'l' => BlockKind::Slingshot,
        _ => return,
    };
    match client_data.player {
        1 => selected.p1 = kind,
        2 => selected.p2 = kind,
        _ => {}
    }
}
fn shop_click(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    shop_open: Res<ShopOpen>,
    client: Option<Res<crate::client::ClientSocket>>,
) {
    if !shop_open.0 || !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let Ok(window) = windows.single() else { return };
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let Ok((camera, cam_transform)) = camera_q.single() else {
        return;
    };
    let Ok(world) = camera.viewport_to_world_2d(cam_transform, cursor) else {
        return;
    };

    const SHOP_W: f32 = 1080.0;
    const SHOP_H: f32 = 600.0;
    let corner = |px: f32, py: f32| Vec2::new(px - SHOP_W / 2.0, SHOP_H / 2.0 - py);

    let wood_rect = Rect::from_corners(corner(290.0, 140.0), corner(474.0, 304.0));
    let stone_rect = Rect::from_corners(corner(555.0, 140.0), corner(739.0, 304.0));
    let bank_rect = Rect::from_corners(corner(820.0, 140.0), corner(1004.0, 304.0));
    let cannon_rect = Rect::from_corners(corner(425.0, 345.0), corner(609.0, 509.0));
    let slingshot_rect = Rect::from_corners(corner(745.0, 345.0), corner(929.0, 509.0));

    // Which selection byte to send to the host
    let byte: &[u8] = if wood_rect.contains(world) {
        b"w"
    } else if stone_rect.contains(world) {
        b"t"
    } else if bank_rect.contains(world) {
        b"k"
    } else if cannon_rect.contains(world) {
        b"j"
    } else if slingshot_rect.contains(world) {
        b"l"
    } else {
        return;
    };

    if let Some(c) = &client {
        let _ = c.send(byte);
    }
}
fn spawn_slingshot(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    client_data: Res<ClientInput>,
    mut next_id: ResMut<NextBlockId>,
) {
    if client_data.input != b'N' {
        return;
    }
    let player = client_data.player;
    let Some(side) = Side::for_player(player) else {
        return;
    };
    let id = next_id.0;
    next_id.0 += 1;
    commands.spawn((
        Sprite {
            image: asset_server.load("MPCannon.png"),
            ..default()
        },
        Slingshot,
        Block {
            mass: 1.0,
            angular_velocity: 0.0,
            inertia: 0.0,
            playerNum: player,
            fcoeff: 0.75,
            rotated: false,
        },
        BlockId(id),
        JustDropped,
        Velocity::default(),
        AngularVelocity(0.),
        side,
        Transform::from_xyz(side.base_center(), 275., 2.),
    ));
}

fn launch_cannonball(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    slingshots: Query<(&Transform, &Side, &Block), With<Slingshot>>,
    client_data: Res<ClientInput>,
    mut next_id: ResMut<NextBlockId>,
) {
    if client_data.input != b'M' {
        return;
    }
    let player = client_data.player;
    // Only the shooter's own cannons fire
    for (transform, side, _) in slingshots.iter().filter(|(_, _, b)| b.playerNum == player) {
        let direction = if *side == Side::Left { 1.0 } else { -1.0 };
        let launch_vel = Vec2::new(LAUNCH_SPEED.x * direction, LAUNCH_SPEED.y);
        let position = transform.translation + Vec3::X * MUZZLE_OFFSET * direction;
        let id = next_id.0;
        next_id.0 += 1;
        commands.spawn((
            Sprite {
                image: asset_server.load("cannonball2.png"),
                custom_size: Some(CANNONBALL_SPRITE_SIZE),
                ..default()
            },
            Ammo,
            Block {
                mass: 1.0,
                angular_velocity: 0.0,
                inertia: 0.0,
                playerNum: player,
                fcoeff: 0.75,
                rotated: false,
            },
            BlockId(id),
            Velocity(launch_vel),
            AngularVelocity(0.),
            Transform::from_translation(position),
        ));
    }
}

fn swap_to_credits(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        // also need to clear all blocks here!
        next_state.set(GameState::Credits);
    }
}
fn reset_money(mut p1: ResMut<Player1Money>, mut p2: ResMut<Player2Money>) {
    p1.money = STARTING_MONEY;
    p2.money = STARTING_MONEY;
}

fn setup_game(
    mut commands: Commands,

    asset_server: Res<AssetServer>,
    mut camera: Single<&mut Transform, With<Camera>>,
    client_data: ResMut<ClientInput>,
) {
    /*commands.spawn((
        Sprite {
            image: asset_server.load("Tower_outline.png"),
            color: Color::srgba(1.0, 1.0, 1.0, 1.0),
            custom_size: Some(SLIDE_SIZE),
            ..default()
        },
        Transform::from_xyz(0., 0., 0.1),
    ));*/

    commands.spawn((
        Sprite {
            image: asset_server.load("MPBR.png"),
            color: Color::srgba(1.0, 1.0, 1.0, 0.9),
            custom_size: Some(SLIDE_SIZE),
            ..default()
        },
        Transform::from_xyz(0., 0., 0.),
    ));
    /*let base_mass = 1.0;
    let base_inertia = base_mass * ((256.0 * 256.0) + (20.0 * 20.0)) / 12.0;

    for x in [-464.0, 464.0] {
        commands.spawn((
            Block {
                mass: base_mass,
                angular_velocity: 0.0,
                inertia: base_inertia,
                fcoeff: 0.25,
            },
            Velocity(Vec2::ZERO),
            AngularVelocity(0.0),
            StaticBody,
            Transform::from_xyz(x, TOWER_BASE_Y - 10.0, 0.0),
        ));
    }*/
    camera.translation.x = 0.;
}
//             Transform::from_xyz(x, TOWER_BASE_Y - 10.0, 0.0),
//         ));
//     }*/
//     camera.translation.x = 0.;
// }
//             Block {
//                 mass: base_mass,
//                 angular_velocity: 0.0,
//                 inertia: base_inertia,
//                 fcoeff: 0.25,
//             },
//             Velocity(Vec2::ZERO),
//             AngularVelocity(0.0),
//             StaticBody,
//             Transform::from_xyz(x, TOWER_BASE_Y - 10.0, 0.0),
//         ));
//     }*/
//     camera.translation.x = 0.;
// }
