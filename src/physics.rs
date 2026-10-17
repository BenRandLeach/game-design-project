use crate::game::NewEntity;
use crate::game::Block;
use crate::game::BeingPlaced;
use crate::game::JustDropped;
use crate::server::ClientInput;
use crate::server::stop_input;
use crate::client::ServerInput;
use crate::client::stop_server_input;
use crate::{GameState, WIN_H, WIN_W};
use bevy::math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume};
use bevy::input::mouse::MouseButton::Right;
use bevy::prelude::*;

const GRAVITY: f32 = -980.0;
const GROUND_Y: f32 = -WIN_H / 2.0;
const LEFT: f32 = -WIN_W / 2.0;
const RIGHT: f32 = WIN_W / 2.0;
const TOWER_BASE_Y: f32 = (-WIN_H / 2.0) + 20.0;
const EDGE_TO_TOWER: f32 = 48.0; // 48px between base and border, 8px is half the width of a block
const BASE_WIDTH: f32 = 256.0;

// TODO(physics): hardcoded for WoodBlock.png (48px); replace with a
// per-entity half-extent once AABB/SAT collision lands.
const BLOCK_HALF_HEIGHT: f32 = 24.0;
const BLOCK_HALF_WIDTH: f32 = 8.0;

pub struct Obb{
    pub center: Vec2,
    pub half_dims: Vec2,
    pub axes: [Vec2; 2],
}

#[derive(Component, Deref, DerefMut, Default)]
pub struct Velocity(pub Vec2);

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                integrate_velocity,
                resolve_ground_collision,
                resolve_block_collision,
                bound_placement,
                rotate_block,
                check_obb_collisions,
            )
                .chain()
                .run_if(in_state(GameState::Game)),
        );
    }
}
pub fn create_obb(
    transform: &Transform,
    hwidth: f32,
    hheight:f32,
) -> Obb {
    let angle = transform.rotation.to_euler(EulerRot::XYZ).2;
    let u = Vec2::new(angle.cos(), angle.sin());
    let v = Vec2::new(-angle.sin(), angle.cos());

    Obb {
        center: transform.translation.truncate(),
        half_dims: Vec2::new(hwidth, hheight),
        axes: [u, v],
    }
}
pub fn project_obb(
    obb: &Obb, 
    axis: Vec2
) -> (f32, f32) {
    let cp = obb.center.dot(axis);
    let ua = obb.axes[0].dot(axis);
    let va = obb.axes[1].dot(axis);
    let r = (obb.half_dims.x * (ua).abs()) + (obb.half_dims.y * (va).abs());
    (cp - r, cp + r)
}

pub fn interval_overlap(
    ntv1: (f32, f32),  // where [1] >= [0]
    ntv2: (f32, f32)
) -> f32 {
    ntv1.1.min(ntv2.1) - ntv1.0.max(ntv2.0)
}

pub fn obb_collision(
    transform1: &Transform,
    transform2: &Transform,
    width: f32,
    height: f32
) -> f32 {
    let obb1 = create_obb(transform1, width, height);
    let obb2 = create_obb(transform2, width, height);

    let interval_1 = project_obb(&obb1, obb1.axes[0]);
    let interval_2 = project_obb(&obb2, obb1.axes[0]);
    let touch1 = interval_overlap(interval_1, interval_2);

    let interval_1 = project_obb(&obb1, obb1.axes[1]);
    let interval_2 = project_obb(&obb2, obb1.axes[1]);
    let touch2 = interval_overlap(interval_1, interval_2);

    let interval_1 = project_obb(&obb1, obb2.axes[0]);
    let interval_2 = project_obb(&obb2, obb2.axes[0]);
    let touch3 = interval_overlap(interval_1, interval_2);

    let interval_1 = project_obb(&obb1, obb2.axes[1]);
    let interval_2 = project_obb(&obb2, obb2.axes[1]);
    let touch4 = interval_overlap(interval_1, interval_2);

    let min1 = touch1.min(touch2);
    let min2 = touch3.min(touch4);
    min1.min(min2)
}
// Semi-implicit Euler: update velocity first, then move using the new velocity.
fn integrate_velocity(time: Res<Time>, mut bodies: Query<(&mut Transform, &mut Velocity), (With<JustDropped>, Without<BeingPlaced>),>,) {
    let dt = time.delta_secs();
    for (mut transform, mut velocity) in &mut bodies {
        velocity.y += GRAVITY * dt;
        transform.translation.x += velocity.x * dt;
        transform.translation.y += velocity.y * dt;
    }
}

// Ground-plane clamp only; block-vs-block collision is future work.
fn resolve_ground_collision(mut bodies: Query<(Entity, &mut Transform, &mut Velocity)>,
mut commands: Commands,keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    for (entity, mut transform, mut velocity) in &mut bodies {
        let rotation = transform.rotation.to_euler(EulerRot::XYZ).2;
            let (half_width, half_height) =
                if (rotation.abs() - std::f32::consts::FRAC_PI_2).abs() < 0.1 {
                    (BLOCK_HALF_HEIGHT, BLOCK_HALF_WIDTH)
                } else {
                    (BLOCK_HALF_WIDTH, BLOCK_HALF_HEIGHT)
                };
        let bottom = transform.translation.y - half_height;
        let top = transform.translation.y + half_height;
        let right = transform.translation.x + half_width;
        let left = transform.translation.x - half_width;
        if right > (LEFT + EDGE_TO_TOWER) && left < (LEFT + EDGE_TO_TOWER + BASE_WIDTH) && bottom < TOWER_BASE_Y {
            transform.translation.y = TOWER_BASE_Y + half_height;
            velocity.y = 0.0;
            commands.entity(entity).remove::<JustDropped>();
        }
        /*if top < GROUND_Y {
            REMOVE BLOCK ENTITY
        }*/
    }
}
pub fn bound_placement(mut bodies: Query<(Entity,&mut Transform, &mut Velocity), With<BeingPlaced>>,
mut commands: Commands,keyboard_input: Res<ButtonInput<KeyCode>>,client_data: ResMut<ClientInput>,server_data:ResMut<ServerInput>,
) {
    for (entity, mut transform, mut velocity) in &mut bodies {
        transform.translation.x += velocity.x;
        let right = transform.translation.x + BLOCK_HALF_WIDTH;
        let left = transform.translation.x - BLOCK_HALF_WIDTH;
        //if(left player)
        if right > (LEFT + EDGE_TO_TOWER + BASE_WIDTH) {
            //transform.translation.x = LEFT + EDGE_TO_TOWER - BLOCK_HALF_WIDTH + BASE_WIDTH;
            velocity.x = -2.0;
        }
        else if left < (LEFT + EDGE_TO_TOWER) {
            //transform.translation.x = LEFT + EDGE_TO_TOWER + BLOCK_HALF_WIDTH;
            velocity.x = 2.0;
        }
        if keyboard_input.just_pressed(KeyCode::KeyS) || client_data.input==b'S'|| server_data.input==b'S' {
            velocity.x = 0.0;
            commands.entity(entity).remove::<BeingPlaced>();       
            commands.entity(entity).insert(JustDropped);    
        }
        /*if(right player)
        if(left < (RIGHT - EDGE_TO_TOWER - BASE_WIDTH)) {
            transform.translation.x = LEFT + EDGE_TO_TOWER + BASE_WIDTH;
            velocity.x = 0.0;
        }
        else if(right > (RIGHT - EDGE_TO_TOWER)) {
            transform.translation.x = RIGHT - EDGE_TO_TOWER - BLOCK_HALF_WIDTH;
            velocity.x = 0.0;
        }*/
    }
    if(client_data.input==b'S'){
    stop_input(client_data);}
    if(server_data.input==b'S')
    {
        stop_server_input(server_data);
    }
}
pub fn rotate_block(
    mut blocks: Query<&mut Transform, With<BeingPlaced>>,
    keyboard: Res<ButtonInput<KeyCode>>,client_data: ResMut<ClientInput>,server_data:ResMut<ServerInput>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) || client_data.input==b'R' || server_data.input==b'R' {
        if let Ok(mut transform) = blocks.single_mut() {
            stop_input(client_data);
            stop_server_input(server_data);
            //println!("Hi");
            //transform.rotate_local_z(std::f32::consts::FRAC_PI_2);
            //println!("Before {:?} ", transform.rotation.to_euler(EulerRot::XYZ).2);
            transform.rotation *= Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
            //println!("After {:?} ", transform.rotation.to_euler(EulerRot::XYZ).2);
        }
    }
}
pub fn resolve_block_collision(
    mut commands: Commands,
    mut active: Query<(Entity, &mut Transform, &mut Velocity), With<JustDropped>>,
    other_blocks: Query<&Transform, (With<Block>, Without<JustDropped>)>,
) {
    if let Ok((entity, mut active_transform, mut active_velocity)) = active.single_mut() {

        for other_transform in &other_blocks {
            let rotation = active_transform.rotation.to_euler(EulerRot::XYZ).2;
            let other_rotation = other_transform.rotation.to_euler(EulerRot::XYZ).2;
            let (active_half_width, active_half_height) =
                if (rotation.abs() - std::f32::consts::FRAC_PI_2).abs() < 0.1 {
                    (BLOCK_HALF_HEIGHT, BLOCK_HALF_WIDTH)
                } else {
                    (BLOCK_HALF_WIDTH, BLOCK_HALF_HEIGHT)
                };
            let (other_half_width, other_half_height) =
                if (other_rotation.abs() - std::f32::consts::FRAC_PI_2).abs() < 0.1 {
                    (BLOCK_HALF_HEIGHT, BLOCK_HALF_WIDTH)
                } else {
                    (BLOCK_HALF_WIDTH, BLOCK_HALF_HEIGHT)
                };
                //println!("width: {:?} ", half_width);
            let active_box = Aabb2d::new(
            active_transform.translation.truncate(),
            Vec2::new(active_half_width, active_half_height),
            );
            let other_box = Aabb2d::new(
                other_transform.translation.truncate(),
                Vec2::new(other_half_width, other_half_height),
            );
            //let dx = (active_box.center().x - other_box.center().x).abs();
            //let dy = (active_box.center().y - other_box.center().y).abs();
            //println!("Current active block position: {:?}", active_transform.translation);
            let active_right = active_box.center().x + active_half_width;
            let active_left = active_box.center().x - active_half_width;
            let active_bottom = active_box.center().y - active_half_height;
            let other_top = other_box.center().y + other_half_height;
            let other_right = other_box.center().x + other_half_width;
            let other_left = other_box.center().x - other_half_width;
            let within_horiz = active_left < other_right && active_right > other_left; 
            if active_box.intersects(&other_box) && within_horiz {
                if active_velocity.y < 0.0 {
                    active_transform.translation.y =
                        other_top + active_half_height;
                    active_velocity.y = 0.0;
                    commands.entity(entity).remove::<JustDropped>();
                }
                /*else if active_box.intersects(&other_box) && active_velocity.x < 0.0 && active_bottom < other_top &&  {
                    active_transform.translation.x =
                        other_transform.translation.x + other_half_width * 2.0;
                    active_velocity.x = 0.0;
                }
                else if active_box.intersects(&other_box) && active_velocity.x > 0.0 && active_bottom < other_top {
                    active_transform.translation.x =
                        other_transform.translation.x - other_half_width * 2.0;
                    active_velocity.x = 0.0;
                } */
            }
        }
    }
}
pub fn check_obb_collisions(
    blocks: Query<(Entity, &Transform), With<Block>>,
) {
    let mut combinations = blocks.iter_combinations();
    
    while let Some([(entity1, transform1), (entity2, transform2)]) = combinations.fetch_next() {
        let penetration = obb_collision(
            transform1,
            transform2,
            BLOCK_HALF_WIDTH,
            BLOCK_HALF_HEIGHT,
        );

        if penetration > 0.0 {
            println!("Collision detected: {}", penetration);
        }
    }   
}