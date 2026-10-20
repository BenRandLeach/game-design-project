use crate::game::NewEntity;
use crate::game::Block;
use crate::game::Slingshot;
use crate::game::Ammo;
use crate::game::BeingPlaced;
use crate::game::JustDropped;
use crate::game::CurrentlyActive;
use crate::server::ClientInput;
use crate::server::stop_input;
use crate::client::ServerInput;
use crate::client::stop_server_input;
use crate::side::Side;
use crate::{GameState, WIN_H};
use bevy::math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume};
use bevy::input::mouse::MouseButton::Right;
use bevy::prelude::*;
use bevy::time::Fixed;

const GRAVITY: f32 = -980.0;
const RESTITUTION: f32 = 1.0; // Between 0.0 - 1.0 , decides elasticity in collisions, 
                            // the higher the resitution, the more momentum the incoming object gives away.
const GROUND_Y: f32 = -WIN_H / 2.0;
const TOWER_BASE_Y: f32 = (-WIN_H / 2.0) + 20.0;

// TODO(physics): hardcoded for WoodBlock.png (48px); replace with a
// per-entity half-extent once AABB/SAT collision lands.
const BLOCK_HALF_HEIGHT: f32 = 24.0;
const BLOCK_HALF_WIDTH: f32 = 8.0;

pub struct Obb{    // Obb object
    pub center: Vec2,   // Center position (x,y)
    pub half_dims: Vec2, // (BLOCK_HALF_HEIGHT, BLOCK_HALF_WIDTH)
    pub axes: [Vec2; 2], // Both unique angles turned into 2 new local axes
}

#[derive(Component, Deref, DerefMut, Default)]
pub struct Velocity(pub Vec2);

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_hz(60.0));
        app.add_systems(
            FixedUpdate,
            (
                fast_fall,
                integrate_velocity,
                check_obb_collisions,
                resolve_ground_collision,
                //resolve_block_collision,
                bound_placement,
                rotate_block,
                //check_obb_collisions,
            )
                .chain()
                .run_if(in_state(GameState::Game)),
        );
    }
}

// Create an obb with block stats
pub fn create_obb(
    transform: &Transform,  
    hwidth: f32,    
    hheight:f32,  //h = half
) -> Obb {
    let angle = transform.rotation.to_euler(EulerRot::XYZ).2; // Finds the angle the block is rotated at
    let u = Vec2::new(angle.cos(), angle.sin());  // Local width axis
    let v = Vec2::new(-angle.sin(), angle.cos());  // Local height axis

    Obb {
        center: transform.translation.truncate(),   
        half_dims: Vec2::new(hwidth, hheight),
        axes: [u, v],
    }
}

// Find the interval a block stretches when projecting it onto a test axis
pub fn project_obb(
    obb: &Obb, 
    axis: Vec2
) -> (f32, f32) {
    let cp = obb.center.dot(axis);  // project center onto current test axis
    let ua = obb.axes[0].dot(axis);  // project width axis onto test axis
    let va = obb.axes[1].dot(axis);  // project height axis onto text axis
    let r = (obb.half_dims.x * (ua).abs()) + (obb.half_dims.y * (va).abs());  // Find our radius, which is how far our block
    (cp - r, cp + r)  // interval of stretch on the test axis                // stretches from the center (in each direction)
}                                                                           // once projected onto the test axis

// Finds the minimum overlap between two intervals
pub fn interval_overlap(   
    ntv1: (f32, f32),  // naturally [1] >= [0]
    ntv2: (f32, f32)
) -> f32 { // returns penetration depth
    ntv1.1.min(ntv2.1) - ntv1.0.max(ntv2.0)  // min(max1, max2) - max(min1, min2) where ntv1 = (min1,max1)   (minimum)
}                                                                                    // ntv2 = (min2,max2)   (overlap)

// Finds if there is a collision using obbs
pub fn obb_collision(
    transform1: &Transform,
    transform2: &Transform,
    width1: f32,
    width2: f32,
    height1: f32,
    height2: f32
) -> (f32, Vec2) {
    
    let obb1 = create_obb(transform1, width1, height1);    // these are the two blocks
    let obb2 = create_obb(transform2, width2, height2);   // were comparing, obb1 is actively closing in all obb's obb2
                                                       // and hopefully colliding with
    let mut min_touch = f32::MAX;    // min penetration: < 0 means no collision, = 0 means just touching, > 0 means colliding
    let mut min_axis = obb1.axes[0]; //THIS WILL BE OUR NORMAL!!!
                                     //To be precise it is our axis of minimum penetration, but serves as our penetration normal
    let axes = [                     // aka what direction we are sending the momentum through
        obb1.axes[0], 
        obb1.axes[1], 
        obb2.axes[0], 
        obb2.axes[1],
    ]; 
    // decides min_touch and min_axis by testing all 4 axes
    for axis in axes{
        let interval_1 = project_obb(&obb1, axis);
        let interval_2 = project_obb(&obb2, axis);
        let touch = interval_overlap(interval_1, interval_2);  // penetration 
        if(touch <= 0.0){
            return (touch, axis);
        }
        if(touch < min_touch) {
            min_touch = touch; 
            min_axis = axis;    
        }  
    }
    // If we find a negative axis, make it positive
    let direction = obb2.center - obb1.center; 
    if direction.dot(min_axis) < 0.0 {
        min_axis = -min_axis;
    }

    (min_touch, min_axis)
}

// Semi-implicit Euler: update velocity first, then move using the new velocity.
fn integrate_velocity(
    time: Res<Time>, 
    mut bodies: Query<
        (&mut Transform, &mut Velocity), 
        (With<Block>, Without<BeingPlaced>),
    >,
) {
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
cannonballs: Query<(), With<Ammo>>,
) {
    for (entity, mut transform, mut velocity) in &mut bodies {
        let rotation = transform.rotation.to_euler(EulerRot::XYZ).2;
            let (half_width, half_height) =
            if(cannonballs.contains(entity)){
                (12.0, 12.0)
            }
            else{
                if (rotation.abs() - std::f32::consts::FRAC_PI_2).abs() < 0.1 {
                    (BLOCK_HALF_HEIGHT, BLOCK_HALF_WIDTH)
                } else {
                    (BLOCK_HALF_WIDTH, BLOCK_HALF_HEIGHT)
                }
            };
        let bottom = transform.translation.y - half_height;
        let top = transform.translation.y + half_height;
        let right = transform.translation.x + half_width;
        let left = transform.translation.x - half_width;
        // Both towers physically exist, so land on whichever base is underneath
        for base in [Side::Left, Side::Right] {
            if right > base.base_left() && left < base.base_right() && bottom < TOWER_BASE_Y {
                transform.translation.y = TOWER_BASE_Y + half_height;
                velocity.y = 0.0;
                commands.entity(entity).remove::<JustDropped>();
                break;
            }
        }
        /*if top < GROUND_Y {
            REMOVE BLOCK ENTITY
        }*/
    }
}

// Bounds placement horizontally, and moves block while choosing placement
pub fn bound_placement(mut bodies: Query<(Entity,&mut Transform, &mut Velocity, &Side), With<BeingPlaced>>,
mut commands: Commands,keyboard_input: Res<ButtonInput<KeyCode>>,client_data: ResMut<ClientInput>,server_data:ResMut<ServerInput>,
) {
    for (entity, mut transform, mut velocity, side) in &mut bodies {
        transform.translation.x += velocity.x;
        const MOVE_SPEED: f32 = 5.0;
        let mut dir = 0.;
        if keyboard_input.pressed(KeyCode::KeyA) || client_data.input==b'A'|| server_data.input==b'A' {
            dir += -1.;
        }

        if keyboard_input.pressed(KeyCode::KeyD) || client_data.input==b'D'|| server_data.input==b'D' {
            dir += 1.;
        }
        transform.translation.x += dir * MOVE_SPEED;

        // Bounds come from the block's own side, not the local player's
        let min_x = side.base_left() + (BLOCK_HALF_WIDTH * 2.);
        let max_x = side.base_right() - (BLOCK_HALF_WIDTH * 2.);
        transform.translation.x = transform.translation.x.clamp(min_x, max_x);

        // Bounce off the edges so auto-move keeps going back and forth
        if transform.translation.x >= max_x {
            velocity.x = -velocity.x.abs();
        } else if transform.translation.x <= min_x {
            velocity.x = velocity.x.abs();
        }

        if keyboard_input.pressed(KeyCode::KeyS) || client_data.input==b'S'|| server_data.input==b'S' {
            velocity.x = 0.0;
            commands.entity(entity).remove::<BeingPlaced>();       
            commands.entity(entity).insert(JustDropped);    
        }
    }
    if(client_data.input==b'S'){
    stop_input(client_data);}
    if(server_data.input==b'S')
    {
        stop_server_input(server_data);
    }
}

// Rotate current block being placed by 90 degrees (pi/2 radians)
pub fn rotate_block(
    mut blocks: Query<&mut Transform, With<BeingPlaced>>,
    keyboard: Res<ButtonInput<KeyCode>>,client_data: ResMut<ClientInput>,server_data:ResMut<ServerInput>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) || client_data.input==b'R' || server_data.input==b'R' {
        if let Ok(mut transform) = blocks.single_mut() {
            stop_input(client_data);
            stop_server_input(server_data);
            transform.rotation *= Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
        }
    }
}

pub fn fast_fall(
    time: Res<Time>,
    active: Res<CurrentlyActive>,
    mut blocks: Query<&mut Velocity, With<JustDropped>>,
    keyboard: Res<ButtonInput<KeyCode>>, client_data: ResMut<ClientInput>, server_data:ResMut<ServerInput>,
) {
    const ACCEL_INCREASE: f32 = 2000.;
    const MAX_FALL_SPEED: f32 = 10000.;

    if !(keyboard.pressed(KeyCode::KeyS) || client_data.input==b'S' || server_data.input==b'S') {
        return;
    }
    else {
        let dt = time.delta_secs();

        let Some(entity) = active.0 else { return };
        if let Ok(mut velocity) = blocks.get_mut(entity) { //for mut velocity in &mut blocks {
            velocity.y = (velocity.y - dt * ACCEL_INCREASE).max(-MAX_FALL_SPEED);
        }
    }
}

// Stops blocks from falling above by using AABB's
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

// Constantly check all pairs of blocks for collisions, and resolve them if there is one
pub fn check_obb_collisions(
    mut blocks: Query<(Entity, &Block, &mut Transform, &mut Velocity)>,
    slingshots: Query<(), With<Slingshot>>,
    cannonballs: Query<(), With<Ammo>>,
) {
    let mut combinations = blocks.iter_combinations_mut();
    
    while let Some([(entity1, block1, mut transform1, mut velocity1),
                    (entity2, block2, mut transform2, mut velocity2)]) = combinations.fetch_next() {
        let (width1, height1) = 
        if slingshots.contains(entity1) {
            (15.0, 15.0)
        } 
        else if cannonballs.contains(entity1){
            (10.0, 10.0)
        }
        else{
            (BLOCK_HALF_WIDTH, BLOCK_HALF_HEIGHT)
        };
        let (width2, height2) =
        if slingshots.contains(entity2) {
            (15.0, 15.0)
        }
        else if cannonballs.contains(entity2){
            (12.0, 12.0)
        }
        else {
            (BLOCK_HALF_WIDTH, BLOCK_HALF_HEIGHT)
        };
        let (penetration, normal) = obb_collision(
            &transform1,
            &transform2,
            width1,
            width2,
            height1,
            height2
        );

        if penetration > 0.0 {
            let relative_velocity = velocity2.0 - velocity1.0;
            let velocity_along_normal = relative_velocity.dot(normal);
            let impulse_num = -1.0 * ((1.0 + RESTITUTION)* velocity_along_normal);
            let impulse_den = (1.0/block1.mass) + (1.0/block2.mass);
            let impulse = impulse_num/impulse_den;
            let impulse_vec = impulse * normal;
            if(velocity_along_normal < 0.0){
                velocity1.0 -= impulse_vec / block1.mass;
                velocity2.0 += impulse_vec / block2.mass; 
            }
            transform1.translation -= (normal * penetration / 2.0).extend(0.0);
            transform2.translation += (normal * penetration / 2.0).extend(0.0);
        }
    }   
}