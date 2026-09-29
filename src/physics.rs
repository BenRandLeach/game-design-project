use bevy::prelude::*;
use crate::{GameState, WIN_H};

const GRAVITY: f32 = -980.0;
const GROUND_Y: f32 = -WIN_H / 2.0;

// TODO(physics): hardcoded for WoodBlock.png (48px); replace with a
// per-entity half-extent once AABB/SAT collision lands.
const BLOCK_HALF_HEIGHT: f32 = 24.0;

#[derive(Component, Deref, DerefMut, Default)]
pub struct Velocity(pub Vec2);

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (integrate_velocity, resolve_ground_collision)
                .chain()
                .run_if(in_state(GameState::Game)),
        );
    }
}

// Semi-implicit Euler: update velocity first, then move using the new velocity.
fn integrate_velocity(time: Res<Time>, mut bodies: Query<(&mut Transform, &mut Velocity)>) {
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
