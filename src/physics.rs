use crate::game::NewEntity;
use crate::game::Block;
use crate::{GameState, WIN_H};
use bevy::math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume};
use bevy::input::mouse::MouseButton::Right;
use bevy::prelude::*;

const GRAVITY: f32 = -980.0;
const GROUND_Y: f32 = -WIN_H / 2.0;

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
            )
                .chain()
                .run_if(in_state(GameState::Game)),
        );
    }
}

// Semi-implicit Euler: update velocity first, then move using the new velocity.
fn integrate_velocity(time: Res<Time>, mut bodies: Query<(&mut Transform, &mut Velocity), With<NewEntity>>
) {
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
        if bottom < GROUND_Y {
            transform.translation.y = GROUND_Y + BLOCK_HALF_HEIGHT;
            velocity.y = 0.0;
        }
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