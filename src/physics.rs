use crate::game::NewEntity;
use crate::game::Block;
use crate::game::BeingPlaced;
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
            )
                .chain()
                .run_if(in_state(GameState::Game)),
        );
    }
}

// Semi-implicit Euler: update velocity first, then move using the new velocity.
fn integrate_velocity(time: Res<Time>, mut bodies: Query<(&mut Transform, &mut Velocity), Without<BeingPlaced>>) {
    let dt = time.delta_secs();
    for (mut transform, mut velocity) in &mut bodies {
        velocity.y += GRAVITY * dt;
        transform.translation.x += velocity.x * dt;
        transform.translation.y += velocity.y * dt;
    }
}

// Ground-plane clamp only; block-vs-block collision is future work.
fn resolve_ground_collision(mut bodies: Query<(&mut Transform, &mut Velocity)>) {
    for (mut transform, mut velocity) in &mut bodies {
        let bottom = transform.translation.y - BLOCK_HALF_HEIGHT;
        let top = transform.translation.y + BLOCK_HALF_HEIGHT;
        let right = transform.translation.x + BLOCK_HALF_WIDTH;
        let left = transform.translation.x - BLOCK_HALF_WIDTH;
        if(right > (LEFT + EDGE_TO_TOWER) && left < (LEFT + EDGE_TO_TOWER + BASE_WIDTH) && bottom < TOWER_BASE_Y) {
            transform.translation.y = TOWER_BASE_Y + BLOCK_HALF_HEIGHT;
            velocity.y = 0.0;
        }
        /*if top < GROUND_Y {
            REMOVE BLOCK ENTITY
        }*/
    }
}
pub fn bound_placement(mut bodies: Query<(Entity,&mut Transform, &mut Velocity), 
 With<BeingPlaced>>,
mut commands: Commands,keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    for (entity, mut transform, mut velocity) in &mut bodies {
        let right = transform.translation.x + BLOCK_HALF_WIDTH;
        let left = transform.translation.x - BLOCK_HALF_WIDTH;
        //if(left player)
        if left < (LEFT + EDGE_TO_TOWER) {
            transform.translation.x = LEFT + EDGE_TO_TOWER + BLOCK_HALF_WIDTH;
            velocity.x = 0.0;
        }
        else if right > (LEFT + EDGE_TO_TOWER + BASE_WIDTH) {
            transform.translation.x = LEFT + EDGE_TO_TOWER - BLOCK_HALF_WIDTH + BASE_WIDTH;
            velocity.x = 0.0;
        }
        if keyboard_input.just_pressed(KeyCode::KeyS) {
            commands.entity(entity).remove::<BeingPlaced>();           
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
}
pub fn resolve_block_collision(
    mut active: Query<(&mut Transform, &mut Velocity), With<NewEntity>>,
    other_blocks: Query<&Transform, (With<Block>, Without<NewEntity>)>,
) {
    if let Ok((mut active_transform, mut active_velocity)) = active.single_mut() {

        for other_transform in &other_blocks {
            let active_box = Aabb2d::new(
            active_transform.translation.truncate(),
            Vec2::new(BLOCK_HALF_WIDTH, BLOCK_HALF_HEIGHT),
            );
            let other_box = Aabb2d::new(
                other_transform.translation.truncate(),
                Vec2::new(BLOCK_HALF_WIDTH, BLOCK_HALF_HEIGHT),
            );
            let dx = (active_box.center().x - other_box.center().x).abs();
            let dy = (active_box.center().y - other_box.center().y).abs();
            //println!("Current active block position: {:?}", active_transform.translation);
            if active_box.intersects(&other_box) {
                if dy > dx && active_velocity.y < 0.0 {
                    active_transform.translation.y =
                        other_transform.translation.y + BLOCK_HALF_HEIGHT * 2.0;
                    active_velocity.y = 0.0;
                }
                else if dx > dy && active_velocity.x < 0.0 {
                    active_transform.translation.x =
                        other_transform.translation.x + BLOCK_HALF_WIDTH * 2.0;
                    active_velocity.x = 0.0;
                }
                else if dx > dy && active_velocity.x > 0.0 {
                    active_transform.translation.x =
                        other_transform.translation.x - BLOCK_HALF_WIDTH * 2.0;
                    active_velocity.x = 0.0;
                }
                
            }
            
        }
    }
}