use crate::client::ServerInput;
use crate::client::client_listener;
use crate::client::stop_server_input;
use crate::game::Ammo;
use crate::game::BeingPlaced;
use crate::game::Block;
use crate::game::CurrentlyActive;
use crate::game::JustDropped;
use crate::game::Slingshot;
use crate::server::ClientInput;
use crate::server::server_listener;
use crate::server::stop_input;
use crate::side::Side;
use crate::{GameState, WIN_H, WIN_W};
use bevy::input::mouse::MouseButton::Right;
use bevy::math::bounding::{Aabb2d, BoundingVolume, IntersectsVolume};
use bevy::prelude::*;
use bevy::time::Fixed;
use std::collections::HashMap;

const GRAVITY: f32 = -980.0;
const RESTITUTION: f32 = 0.0; // Between 0.0 - 1.0, how bouncy collisions are. 1.0 keeps all the energy,
// so anything above a small value stops stacks from ever settling.
const BOUNCE_MIN_SPEED: f32 = 60.0; // Slower impacts than this don't bounce, or resting blocks would jitter
const LANDING_NORMAL: f32 = 0.7; // Contacts at least this close to vertical count as a block landing
const SUBSTEPS: u32 = 4; // Physics steps per tick, keeps fast blocks from sinking deep into each other
const SOLVER_ITERATIONS: u32 = 12; // More iterations = stiffer towers, but slower
const POSITION_SLOP: f32 = 0.25; // Overlap we allow, so resting blocks keep their contact between steps
const POSITION_CORRECTION: f32 = 0.4; // Fraction of the overlap removed per step
const GROUND_CONTACT_DEPTH: f32 = 32.0; // Corners deeper than this already fell past the base
const CONTACT_TOLERANCE: f32 = 0.5; // Lets perfectly aligned corners count as touching
const GROUND_Y: f32 = -WIN_H / 2.0;
const LEFT: f32 = -WIN_W / 2.0;
const RIGHT: f32 = WIN_W / 2.0;
const TOWER_BASE_Y: f32 = (-WIN_H / 2.0) + 20.0;

// TODO(physics): hardcoded for WoodBlock.png (48px); replace with a
// per-entity half-extent once AABB/SAT collision lands.
const BLOCK_HALF_HEIGHT: f32 = 24.0;
const BLOCK_HALF_WIDTH: f32 = 8.0;

pub struct Obb {
    // Obb object
    pub center: Vec2,    // Center position (x,y)
    pub half_dims: Vec2, // (BLOCK_HALF_HEIGHT, BLOCK_HALF_WIDTH)
    pub axes: [Vec2; 2], // Both unique angles turned into 2 new local axes
}

#[derive(Component, Deref, DerefMut, Default)]
pub struct Velocity(pub Vec2);

#[derive(Component, Default)]
pub struct AngularVelocity(pub f32); // in rad / s

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Time::<Fixed>::from_hz(64.0));
        app.add_systems(
            FixedUpdate,
            (
                fast_fall,
                simulate,
                bound_placement,
                rotate_block,
                remove_lost_entities,
            )
                .chain()
                // Read this tick's network input, so every copy of it arrives before physics uses it
                .after(server_listener)
                .after(client_listener)
                .run_if(in_state(GameState::Game))
                .run_if(resource_exists::<crate::server::IsHost>),
        );
    }
}

// Create an obb with block stats
pub fn create_obb(
    transform: &Transform,
    hwidth: f32,
    hheight: f32, //h = half
) -> Obb {
    let angle = transform.rotation.to_euler(EulerRot::XYZ).2; // Finds the angle the block is rotated at
    let u = Vec2::new(angle.cos(), angle.sin()); // Local width axis
    let v = Vec2::new(-angle.sin(), angle.cos()); // Local height axis

    Obb {
        center: transform.translation.truncate(),
        half_dims: Vec2::new(hwidth, hheight),
        axes: [u, v],
    }
}

// Find the interval a block stretches when projecting it onto a test axis
pub fn project_obb(obb: &Obb, axis: Vec2) -> (f32, f32) {
    let cp = obb.center.dot(axis); // project center onto current test axis
    let ua = obb.axes[0].dot(axis); // project width axis onto test axis
    let va = obb.axes[1].dot(axis); // project height axis onto text axis
    let r = (obb.half_dims.x * (ua).abs()) + (obb.half_dims.y * (va).abs()); // Find our radius, which is how far our block
    (cp - r, cp + r) // interval of stretch on the test axis                // stretches from the center (in each direction)
} // once projected onto the test axis

// Finds the minimum overlap between two intervals
pub fn interval_overlap(
    ntv1: (f32, f32), // naturally [1] >= [0]
    ntv2: (f32, f32),
) -> f32 {
    // returns penetration depth
    ntv1.1.min(ntv2.1) - ntv1.0.max(ntv2.0) // min(max1, max2) - max(min1, min2) where ntv1 = (min1,max1)   (minimum)
} // ntv2 = (min2,max2)   (overlap)

fn block_inertia(mass: f32, width: f32, height: f32) -> f32 {
    mass * (width * width + height * height) / 3.0
}

fn corners(o: &Obb) -> [Vec2; 4] {
    let (u, v) = (o.axes[0] * o.half_dims.x, o.axes[1] * o.half_dims.y);
    [
        o.center + u + v,
        o.center + u - v,
        o.center - u + v,
        o.center - u - v,
    ]
}

fn inside(o: &Obb, p: Vec2) -> bool {
    let d = p - o.center;
    d.dot(o.axes[0]).abs() <= o.half_dims.x + CONTACT_TOLERANCE
        && d.dot(o.axes[1]).abs() <= o.half_dims.y + CONTACT_TOLERANCE
}

// All the corners touching the other block, a block needs at least two to sit flat
// Each point comes with an id for which corner it is, so it can be recognized next step
fn contact_points(a: &Obb, b: &Obb) -> Vec<(u8, Vec2)> {
    let pts: Vec<(u8, Vec2)> = (0u8..)
        .zip(corners(a))
        .filter(|(_, p)| inside(b, *p))
        .chain((4u8..).zip(corners(b)).filter(|(_, p)| inside(a, *p)))
        .collect();
    if pts.is_empty() {
        vec![(8, (a.center + b.center) * 0.5)]
    } else {
        pts
    }
}

// Finds if there is a collision using obbs
pub fn obb_collision(
    transform1: &Transform,
    transform2: &Transform,
    width1: f32,
    width2: f32,
    height1: f32,
    height2: f32,
) -> (f32, Vec2) {
    let obb1 = create_obb(transform1, width1, height1); // these are the two blocks
    let obb2 = create_obb(transform2, width2, height2); // were comparing, obb1 is actively closing in all obb's obb2
    // and hopefully colliding with
    let mut min_touch = f32::MAX; // min penetration: < 0 means no collision, = 0 means just touching, > 0 means colliding
    let mut min_axis = obb1.axes[0]; //THIS WILL BE OUR NORMAL!!!
    //To be precise it is our axis of minimum penetration, but serves as our penetration normal
    let axes = [
        // aka what direction we are sending the momentum through
        obb1.axes[0],
        obb1.axes[1],
        obb2.axes[0],
        obb2.axes[1],
    ];
    // decides min_touch and min_axis by testing all 4 axes
    for axis in axes {
        let interval_1 = project_obb(&obb1, axis);
        let interval_2 = project_obb(&obb2, axis);
        let touch = interval_overlap(interval_1, interval_2); // penetration 
        if (touch <= 0.0) {
            return (touch, axis);
        }
        if (touch < min_touch) {
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

// Copy of a body the solver can freely work on, written back to its entity after the tick
struct Body {
    entity: Entity,
    transform: Transform,
    half_size: Vec2,
    velocity: Vec2,
    ang_vel: f32,
    inv_mass: f32,
    inv_inertia: f32,
    fcoeff: f32,
    ammo: bool,
    touched: bool,
}

// A single point where two bodies touch. The normal points from a to b.
// a is None for the tower bases, which never move.
struct Contact {
    a: Option<usize>,
    b: usize,
    ra: Vec2, // Contact point relative to each body's center
    rb: Vec2,
    normal: Vec2,
    fcoeff: f32,
    bounce: f32,  // Speed the bodies should separate at
    impact: bool, // Moving together fast, rather than resting
    move_a: f32,  // 0 stops this contact from moving a body at all
    move_b: f32,
    normal_impulse: f32, // Total impulse so far, kept so a later iteration can take some back
    tangent_impulse: f32,
    id: ContactId,
}

// Identifies the same contact across steps: the two bodies, and which corner is touching
type ContactId = (Option<Entity>, Entity, u8);

// Normal and tangent impulse of every contact on the last step. Resting contacts need about the
// same impulse every step, so starting from these is what keeps a tower from slowly tipping over.
type ContactCache = HashMap<ContactId, (f32, f32)>;

// Overlap of a pair of bodies, removed once per step rather than once per contact point
struct Overlap {
    a: Option<usize>,
    b: usize,
    normal: Vec2,
    depth: f32,
}

fn point_velocity(body: &Body, r: Vec2) -> Vec2 {
    body.velocity + body.ang_vel * r.perp()
}

fn relative_velocity(c: &Contact, bodies: &[Body]) -> Vec2 {
    let va = c.a.map_or(Vec2::ZERO, |a| point_velocity(&bodies[a], c.ra));
    point_velocity(&bodies[c.b], c.rb) - va
}

fn apply_impulse(c: &Contact, bodies: &mut [Body], impulse: Vec2) {
    if let Some(a) = c.a {
        let body = &mut bodies[a];
        body.velocity -= impulse * body.inv_mass * c.move_a;
        body.ang_vel -= c.ra.perp_dot(impulse) * body.inv_inertia * c.move_a;
    }
    let body = &mut bodies[c.b];
    body.velocity += impulse * body.inv_mass * c.move_b;
    body.ang_vel += c.rb.perp_dot(impulse) * body.inv_inertia * c.move_b;
}

// How much mass the contact "feels" along a direction, rotation makes bodies easier to push
fn effective_mass(c: &Contact, bodies: &[Body], direction: Vec2) -> f32 {
    let term = |body: &Body, r: Vec2, scale: f32| {
        let rn = r.perp_dot(direction);
        (body.inv_mass + rn * rn * body.inv_inertia) * scale
    };
    let a = c.a.map_or(0.0, |a| term(&bodies[a], c.ra, c.move_a));
    1.0 / (a + term(&bodies[c.b], c.rb, c.move_b))
}

fn add_contact(
    contacts: &mut Vec<Contact>,
    cache: &ContactCache,
    bodies: &mut [Body],
    (a, b): (Option<usize>, usize),
    (corner, point): (u8, Vec2),
    normal: Vec2,
    fcoeff: f32,
) {
    let center = |i: usize| bodies[i].transform.translation.truncate();
    let id = (a.map(|a| bodies[a].entity), bodies[b].entity, corner);
    let (normal_impulse, tangent_impulse) = cache.get(&id).copied().unwrap_or_default();
    let mut contact = Contact {
        a,
        b,
        ra: a.map_or(Vec2::ZERO, |a| point - center(a)),
        rb: point - center(b),
        normal,
        fcoeff,
        bounce: 0.0,
        impact: false,
        move_a: 1.0,
        move_b: 1.0,
        normal_impulse,
        tangent_impulse,
        id,
    };
    let approach = relative_velocity(&contact, bodies).dot(normal);
    if approach < -BOUNCE_MIN_SPEED {
        contact.bounce = -RESTITUTION * approach;
        contact.impact = true;
        // Shock propagation: a block landing on the stack stops against it as if it was solid. Passing
        // the hit down through every corner of the tower leaves errors that show up as spin on top.
        let landing = normal.y.abs() > LANDING_NORMAL
            && a.is_some_and(|a| !bodies[a].ammo)
            && !bodies[b].ammo;
        if landing && normal.y > 0.0 {
            contact.move_a = 0.0;
        } else if landing {
            contact.move_b = 0.0;
        }
    }
    if let Some(a) = a {
        bodies[a].touched = true;
    }
    bodies[b].touched = true;
    contacts.push(contact);
}

// Every overlapping pair of bodies
fn find_body_contacts(
    bodies: &mut [Body],
    cache: &ContactCache,
    contacts: &mut Vec<Contact>,
    overlaps: &mut Vec<Overlap>,
) {
    for i in 0..bodies.len() {
        for j in (i + 1)..bodies.len() {
            let (t1, t2) = (bodies[i].transform, bodies[j].transform);
            let (h1, h2) = (bodies[i].half_size, bodies[j].half_size);
            let (depth, normal) = obb_collision(&t1, &t2, h1.x, h2.x, h1.y, h2.y);
            if depth <= 0.0 {
                continue;
            }
            let fcoeff = (bodies[i].fcoeff * bodies[j].fcoeff).sqrt();
            let obb1 = create_obb(&t1, h1.x, h1.y);
            let obb2 = create_obb(&t2, h2.x, h2.y);
            for point in contact_points(&obb1, &obb2) {
                add_contact(contacts, cache, bodies, (Some(i), j), point, normal, fcoeff);
            }
            overlaps.push(Overlap {
                a: Some(i),
                b: j,
                normal,
                depth,
            });
        }
    }
}

// Both towers physically exist, so land on whichever base is underneath
fn find_ground_contacts(
    bodies: &mut [Body],
    cache: &ContactCache,
    contacts: &mut Vec<Contact>,
    overlaps: &mut Vec<Overlap>,
) {
    for i in 0..bodies.len() {
        let obb = create_obb(
            &bodies[i].transform,
            bodies[i].half_size.x,
            bodies[i].half_size.y,
        );
        let fcoeff = bodies[i].fcoeff;
        let mut deepest: f32 = 0.0;
        for (id, corner) in (0u8..).zip(corners(&obb)) {
            let depth = TOWER_BASE_Y - corner.y;
            let over_base = [Side::Left, Side::Right]
                .iter()
                .any(|base| corner.x > base.base_left() && corner.x < base.base_right());
            if over_base && depth > 0.0 && depth < GROUND_CONTACT_DEPTH {
                add_contact(
                    contacts,
                    cache,
                    bodies,
                    (None, i),
                    (id, corner),
                    Vec2::Y,
                    fcoeff,
                );
                deepest = deepest.max(depth);
            }
        }
        if deepest > 0.0 {
            overlaps.push(Overlap {
                a: None,
                b: i,
                normal: Vec2::Y,
                depth: deepest,
            });
        }
    }
}

// Sequential impulses: go over every contact repeatedly, so the push of the base
// has time to travel all the way up a tower
fn solve_velocities(contacts: &mut [Contact], bodies: &mut [Body]) {
    for c in contacts.iter() {
        apply_impulse(
            c,
            bodies,
            c.normal * c.normal_impulse + c.normal.perp() * c.tangent_impulse,
        );
    }
    for _ in 0..SOLVER_ITERATIONS {
        contacts.reverse(); // Going back and forth stops the first corner from taking the whole hit
        for c in contacts.iter_mut() {
            let speed = relative_velocity(c, bodies).dot(c.normal);
            let total = (c.normal_impulse
                + (c.bounce - speed) * effective_mass(c, bodies, c.normal))
            .max(0.0);
            apply_impulse(c, bodies, c.normal * (total - c.normal_impulse));
            c.normal_impulse = total;

            // Coulomb friction: can only resist sliding up to fcoeff * how hard the bodies press together.
            // Off during an impact, while the normal impulse is still settling the block has a brief false
            // spin, and friction would lock that in as the block rolling off its corner.
            let tangent = c.normal.perp();
            let slide = relative_velocity(c, bodies).dot(tangent);
            let limit = if c.impact {
                0.0
            } else {
                c.fcoeff * c.normal_impulse
            };
            let total = (c.tangent_impulse - slide * effective_mass(c, bodies, tangent))
                .clamp(-limit, limit);
            apply_impulse(c, bodies, tangent * (total - c.tangent_impulse));
            c.tangent_impulse = total;
        }
    }
}

fn solve_positions(overlaps: &[Overlap], bodies: &mut [Body]) {
    for overlap in overlaps {
        let inv_a = overlap.a.map_or(0.0, |a| bodies[a].inv_mass);
        let inv_b = bodies[overlap.b].inv_mass;
        let push = overlap.normal * (overlap.depth - POSITION_SLOP).max(0.0) * POSITION_CORRECTION
            / (inv_a + inv_b);
        if let Some(a) = overlap.a {
            bodies[a].transform.translation -= (push * inv_a).extend(0.0);
        }
        bodies[overlap.b].transform.translation += (push * inv_b).extend(0.0);
    }
}

// Semi-implicit Euler: update velocity first, then move using the new velocity.
fn integrate_velocity(body: &mut Body, dt: f32) {
    body.velocity.y += GRAVITY * dt;
    body.transform.translation += (body.velocity * dt).extend(0.0);
    body.transform.rotate_z(body.ang_vel * dt);
}

pub fn simulate(
    time: Res<Time>,
    mut commands: Commands,
    mut cache: Local<ContactCache>,
    mut query: Query<
        (
            Entity,
            &Block,
            &mut Transform,
            &mut Velocity,
            &mut AngularVelocity,
            Has<Slingshot>,
            Has<Ammo>,
        ),
        Without<BeingPlaced>,
    >,
) {
    let mut bodies: Vec<Body> = query
        .iter()
        .map(
            |(entity, block, transform, velocity, ang_vel, slingshot, ammo)| {
                let half_size = half_size(slingshot, ammo);
                Body {
                    entity,
                    transform: *transform,
                    half_size,
                    velocity: velocity.0,
                    ang_vel: ang_vel.0,
                    inv_mass: 1.0 / block.mass,
                    inv_inertia: 1.0 / block_inertia(block.mass, half_size.x, half_size.y),
                    fcoeff: block.fcoeff,
                    ammo,
                    touched: false,
                }
            },
        )
        .collect();

    let dt = time.delta_secs() / SUBSTEPS as f32;
    let mut contacts = Vec::new();
    let mut overlaps = Vec::new();
    for _ in 0..SUBSTEPS {
        for body in &mut bodies {
            integrate_velocity(body, dt);
        }
        contacts.clear();
        overlaps.clear();
        find_body_contacts(&mut bodies, &cache, &mut contacts, &mut overlaps);
        find_ground_contacts(&mut bodies, &cache, &mut contacts, &mut overlaps);
        solve_velocities(&mut contacts, &mut bodies);
        solve_positions(&overlaps, &mut bodies);
        cache.clear();
        // An impact only lasts one step, starting the next step with its impulse would launch the block
        cache.extend(
            contacts
                .iter()
                .filter(|c| !c.impact)
                .map(|c| (c.id, (c.normal_impulse, c.tangent_impulse))),
        );
    }

    for body in &bodies {
        let Ok((_, _, mut transform, mut velocity, mut ang_vel, _, _)) = query.get_mut(body.entity)
        else {
            continue;
        };
        *transform = body.transform;
        velocity.0 = body.velocity;
        ang_vel.0 = body.ang_vel;
        if body.touched {
            commands.entity(body.entity).remove::<JustDropped>();
        }
    }
}

fn remove_lost_entities(mut commands: Commands, bodies: Query<(&Transform, Entity), With<Block>>) {
    for (transform, entity) in &bodies {
        if transform.translation.y < GROUND_Y - 48.0
            || transform.translation.x < LEFT
            || transform.translation.x > RIGHT
        {
            commands.entity(entity).despawn();
            println!("Entity Cleared! {:?} ", entity);
        }
    }
}

// Bounds placement horizontally, and moves block while choosing placement
pub fn bound_placement(
    mut bodies: Query<(Entity, &mut Transform, &mut Velocity, &Side), With<BeingPlaced>>,
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    client_data: ResMut<ClientInput>,
    server_data: ResMut<ServerInput>,
) {
    for (entity, mut transform, mut velocity, side) in &mut bodies {
        transform.translation.x += velocity.x;
        const MOVE_SPEED: f32 = 5.0;
        let mut dir = 0.;
        if keyboard_input.pressed(KeyCode::KeyA)
            || client_data.input == b'A'
            || server_data.input == b'A'
        {
            dir += -1.;
        }

        if keyboard_input.pressed(KeyCode::KeyD)
            || client_data.input == b'D'
            || server_data.input == b'D'
        {
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

        if keyboard_input.pressed(KeyCode::KeyS)
            || client_data.input == b'S'
            || server_data.input == b'S'
        {
            velocity.x = 0.0;
            commands.entity(entity).remove::<BeingPlaced>();
            commands.entity(entity).insert(JustDropped);
        }
    }
    if (client_data.input == b'S') {
        stop_input(client_data);
    }
    if (server_data.input == b'S') {
        stop_server_input(server_data);
    }
}

// Rotate current block being placed by 90 degrees (pi/2 radians)
pub fn rotate_block(
    mut blocks: Query<&mut Transform, With<BeingPlaced>>,
    mut client_data: ResMut<ClientInput>,
    mut server_data: ResMut<ServerInput>,
) {
    let rotate = client_data.input == b'R' || server_data.input == b'R';
    if client_data.input == b'R' {
        client_data.input = b'0';
    }
    if server_data.input == b'R' {
        server_data.input = b'0';
    }
    if rotate {
        if let Ok(mut transform) = blocks.single_mut() {
            transform.rotation *= Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
        }
    }
}

pub fn fast_fall(
    time: Res<Time>,
    active: Res<CurrentlyActive>,
    mut blocks: Query<&mut Velocity, With<JustDropped>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    client_data: ResMut<ClientInput>,
    server_data: ResMut<ServerInput>,
) {
    const ACCEL_INCREASE: f32 = 2000.;
    const MAX_FALL_SPEED: f32 = 10000.;

    if !(keyboard.pressed(KeyCode::KeyS) || client_data.input == b'S' || server_data.input == b'S')
    {
        return;
    } else {
        let dt = time.delta_secs();

        let Some(entity) = active.0 else { return };
        if let Ok(mut velocity) = blocks.get_mut(entity) {
            //for mut velocity in &mut blocks {
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
                    active_transform.translation.y = other_top + active_half_height;
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

// Half extents of a body's hitbox
fn half_size(slingshot: bool, ammo: bool) -> Vec2 {
    if slingshot {
        Vec2::new(22.0, 22.0)
    } else if ammo {
        Vec2::new(12.0, 12.0)
    } else {
        Vec2::new(BLOCK_HALF_WIDTH, BLOCK_HALF_HEIGHT)
    }
}

// a bunch of behavior tests that don't need to be run in the game, but are useful for checking the physics code
#[cfg(test)]
mod tests {
    use super::*;
    use crate::side::Side;
    use std::time::Duration;

    const TICK: f64 = 1.0 / 64.0;

    fn world_and_schedule() -> (World, Schedule) {
        let mut world = World::new();
        world.insert_resource(Time::<()>::default());
        let mut schedule = Schedule::default();
        schedule.add_systems((simulate, remove_lost_entities).chain());
        (world, schedule)
    }

    fn run(world: &mut World, schedule: &mut Schedule, ticks: u32) {
        for _ in 0..ticks {
            world
                .resource_mut::<Time>()
                .advance_by(Duration::from_secs_f64(TICK));
            schedule.run(world);
        }
    }

    fn block(world: &mut World, pos: Vec2, vel: Vec2) -> Entity {
        world
            .spawn((
                Block {
                    mass: 1.0,
                    angular_velocity: 0.0,
                    inertia: 0.0,
                    fcoeff: 0.25,
                },
                Velocity(vel),
                AngularVelocity(0.0),
                Transform::from_xyz(pos.x, pos.y, 1.0),
            ))
            .id()
    }

    fn angle(world: &World, e: Entity) -> f32 {
        world
            .get::<Transform>(e)
            .unwrap()
            .rotation
            .to_euler(EulerRot::XYZ)
            .2
    }

    fn pos(world: &World, e: Entity) -> Vec2 {
        world.get::<Transform>(e).unwrap().translation.truncate()
    }

    fn base_x() -> f32 {
        Side::Left.base_center()
    }

    #[test]
    fn sliding_block_slows_down() {
        let (mut world, mut schedule) = world_and_schedule();
        let b = block(
            &mut world,
            Vec2::new(base_x(), TOWER_BASE_Y + 24.0),
            Vec2::new(100.0, 0.0),
        );
        run(&mut world, &mut schedule, 64);
        let v = world.get::<Velocity>(b).unwrap().0;
        println!("slide: v={v}");
        assert!(v.x.abs() < 1.0, "still moving at {}", v.x);
    }

    #[test]
    fn projectile_keeps_speed_in_the_air() {
        let (mut world, mut schedule) = world_and_schedule();
        let b = block(&mut world, Vec2::new(-200.0, 300.0), Vec2::new(200.0, 0.0));
        run(&mut world, &mut schedule, 32);
        let v = world.get::<Velocity>(b).unwrap().0;
        println!("air: v={v}");
        assert!((v.x - 200.0).abs() < 1.0, "x speed changed to {}", v.x);
    }

    #[test]
    fn dropped_block_settles() {
        let (mut world, mut schedule) = world_and_schedule();
        let b = block(
            &mut world,
            Vec2::new(base_x(), TOWER_BASE_Y + 124.0),
            Vec2::ZERO,
        );
        run(&mut world, &mut schedule, 128);
        let v = world.get::<Velocity>(b).unwrap().0;
        let w = world.get::<AngularVelocity>(b).unwrap().0;
        println!("settle: y={} v={v} w={w}", pos(&world, b).y);
        assert!(v.length() < 20.0 && w.abs() < 0.1);
        assert!((pos(&world, b).y - (TOWER_BASE_Y + 24.0)).abs() < 2.0);
    }

    #[test]
    fn straight_tower_stands() {
        let (mut world, mut schedule) = world_and_schedule();
        let mut tower = Vec::new();
        for i in 0..5 {
            let y = TOWER_BASE_Y + 24.0 + 48.0 * i as f32;
            tower.push(block(&mut world, Vec2::new(base_x(), y), Vec2::ZERO));
        }
        run(&mut world, &mut schedule, 64 * 5);
        for (i, &b) in tower.iter().enumerate() {
            let expected = TOWER_BASE_Y + 24.0 + 48.0 * i as f32;
            let p = pos(&world, b);
            println!(
                "tower {i}: dy={} dx={} angle={}",
                p.y - expected,
                p.x - base_x(),
                angle(&world, b)
            );
        }
        for (i, &b) in tower.iter().enumerate() {
            let expected = TOWER_BASE_Y + 24.0 + 48.0 * i as f32;
            let p = pos(&world, b);
            assert!(
                (p.y - expected).abs() < 3.0 && (p.x - base_x()).abs() < 3.0,
                "block {i} moved"
            );
            assert!(angle(&world, b).abs() < 0.05, "block {i} tilted");
        }
    }

    #[test]
    fn tilted_block_falls_flat() {
        let (mut world, mut schedule) = world_and_schedule();
        let b = block(
            &mut world,
            Vec2::new(base_x(), TOWER_BASE_Y + 60.0),
            Vec2::ZERO,
        );
        world.get_mut::<Transform>(b).unwrap().rotation = Quat::from_rotation_z(0.6);
        run(&mut world, &mut schedule, 64 * 4);
        let a = angle(&world, b).abs();
        let w = world.get::<AngularVelocity>(b).unwrap().0;
        println!("tilt: angle={a} w={w}");
        // Past its tipping angle a tall block should end up lying on its long side
        assert!((a - std::f32::consts::FRAC_PI_2).abs() < 0.1 && w.abs() < 0.2);
    }

    #[test]
    fn overhanging_block_topples() {
        let (mut world, mut schedule) = world_and_schedule();
        let lower = block(
            &mut world,
            Vec2::new(base_x(), TOWER_BASE_Y + 24.0),
            Vec2::ZERO,
        );
        // Centre of mass 12px past the edge of the 8px half-width block below
        let upper = block(
            &mut world,
            Vec2::new(base_x() + 12.0, TOWER_BASE_Y + 72.0),
            Vec2::ZERO,
        );
        run(&mut world, &mut schedule, 64 * 2);
        println!(
            "topple: lower={} upper={} angle={}",
            pos(&world, lower),
            pos(&world, upper),
            angle(&world, upper)
        );
        assert!(
            angle(&world, upper).abs() > 0.3,
            "upper block balanced on nothing"
        );
    }

    #[test]
    fn supported_offset_block_stays() {
        let (mut world, mut schedule) = world_and_schedule();
        let lower = block(
            &mut world,
            Vec2::new(base_x(), TOWER_BASE_Y + 24.0),
            Vec2::ZERO,
        );
        // Centre of mass still 4px inside the block below
        let upper = block(
            &mut world,
            Vec2::new(base_x() + 4.0, TOWER_BASE_Y + 72.0),
            Vec2::ZERO,
        );
        run(&mut world, &mut schedule, 64 * 3);
        println!(
            "offset: lower={} upper={} angle={}",
            pos(&world, lower),
            pos(&world, upper),
            angle(&world, upper)
        );
        assert!(angle(&world, upper).abs() < 0.1, "upper tilted");
        assert!(
            (pos(&world, upper).x - (base_x() + 4.0)).abs() < 3.0,
            "upper slid"
        );
    }

    #[test]
    fn fast_drop_lands_on_top_of_block() {
        let (mut world, mut schedule) = world_and_schedule();
        let lower = block(
            &mut world,
            Vec2::new(base_x(), TOWER_BASE_Y + 24.0),
            Vec2::ZERO,
        );
        run(&mut world, &mut schedule, 16);
        let upper = block(
            &mut world,
            Vec2::new(base_x(), TOWER_BASE_Y + 200.0),
            Vec2::new(0.0, -2000.0),
        );
        run(&mut world, &mut schedule, 64 * 2);
        println!(
            "fast drop: lower={} upper={} angle={}",
            pos(&world, lower),
            pos(&world, upper),
            angle(&world, upper)
        );
        // A hit this hard can nudge the whole stack, but the block has to stay on it
        assert!(
            (pos(&world, upper).x - pos(&world, lower).x).abs() < 3.0,
            "shoved sideways"
        );
        assert!(
            (pos(&world, upper).y - (TOWER_BASE_Y + 72.0)).abs() < 3.0,
            "not resting on top"
        );
    }

    #[test]
    fn block_landing_on_block_stops_fast_falling() {
        let (mut world, mut schedule) = world_and_schedule();
        block(
            &mut world,
            Vec2::new(base_x(), TOWER_BASE_Y + 24.0),
            Vec2::ZERO,
        );
        let upper = block(
            &mut world,
            Vec2::new(base_x(), TOWER_BASE_Y + 90.0),
            Vec2::ZERO,
        );
        world.entity_mut(upper).insert(JustDropped);
        run(&mut world, &mut schedule, 32);
        assert!(world.get::<JustDropped>(upper).is_none());
    }

    #[test]
    fn held_block_ignores_the_tower() {
        let (mut world, mut schedule) = world_and_schedule();
        block(
            &mut world,
            Vec2::new(base_x(), TOWER_BASE_Y + 24.0),
            Vec2::ZERO,
        );
        let held = block(
            &mut world,
            Vec2::new(base_x(), TOWER_BASE_Y + 40.0),
            Vec2::ZERO,
        );
        world.entity_mut(held).insert(BeingPlaced);
        run(&mut world, &mut schedule, 16);
        assert_eq!(pos(&world, held), Vec2::new(base_x(), TOWER_BASE_Y + 40.0));
    }

    #[test]
    fn block_past_the_base_does_not_snap_back_up() {
        let (mut world, mut schedule) = world_and_schedule();
        let b = block(
            &mut world,
            Vec2::new(base_x(), TOWER_BASE_Y - 60.0),
            Vec2::ZERO,
        );
        run(&mut world, &mut schedule, 2);
        assert!(pos(&world, b).y < TOWER_BASE_Y - 60.0);
    }

    #[test]
    fn full_height_tower_stands() {
        let (mut world, mut schedule) = world_and_schedule();
        let mut tower = Vec::new();
        for i in 0..13 {
            let y = TOWER_BASE_Y + 24.0 + 48.0 * i as f32;
            tower.push(block(&mut world, Vec2::new(base_x(), y), Vec2::ZERO));
        }
        run(&mut world, &mut schedule, 64 * 10);
        let top = *tower.last().unwrap();
        println!(
            "13 tall: top={} angle={}",
            pos(&world, top),
            angle(&world, top)
        );
        for (i, &b) in tower.iter().enumerate() {
            let p = pos(&world, b);
            assert!(
                (p.x - base_x()).abs() < 5.0 && angle(&world, b).abs() < 0.05,
                "block {i} fell, at {p}"
            );
        }
    }

    #[test]
    fn cannonball_knocks_the_tower() {
        let (mut world, mut schedule) = world_and_schedule();
        let x = Side::Right.base_center();
        let mut tower = Vec::new();
        for i in 0..6 {
            let y = TOWER_BASE_Y + 24.0 + 48.0 * i as f32;
            tower.push(block(&mut world, Vec2::new(x, y), Vec2::ZERO));
        }
        let ball = block(
            &mut world,
            Vec2::new(x - 100.0, TOWER_BASE_Y + 220.0),
            Vec2::new(800.0, 0.0),
        );
        world.entity_mut(ball).insert(Ammo);
        run(&mut world, &mut schedule, 64);
        let moved = tower
            .iter()
            .filter(|&&b| {
                world
                    .get::<Transform>(b)
                    .is_none_or(|t| (t.translation.x - x).abs() > 5.0)
            })
            .count();
        println!("cannonball: {moved} of 6 blocks knocked");
        assert!(moved > 0);
    }

    // Drops a block onto one sitting on the base. Returns the most it spun and the fastest it rebounded.
    fn drop_on_block(
        world: &mut World,
        schedule: &mut Schedule,
        offset: f32,
        speed: f32,
    ) -> (Entity, Entity, f32, f32) {
        let lower = block(world, Vec2::new(base_x(), TOWER_BASE_Y + 24.0), Vec2::ZERO);
        run(world, schedule, 16);
        let start = Vec2::new(base_x() + offset, TOWER_BASE_Y + 100.0);
        let upper = block(world, start, Vec2::new(0.0, -speed));
        let (mut max_spin, mut max_rebound) = (0f32, 0f32);
        for _ in 0..(64 * 2) {
            run(world, schedule, 1);
            let Some(velocity) = world.get::<Velocity>(upper) else {
                break;
            };
            max_rebound = max_rebound.max(velocity.y);
            max_spin = max_spin.max(world.get::<AngularVelocity>(upper).unwrap().0.abs());
        }
        (lower, upper, max_spin, max_rebound)
    }

    #[test]
    fn supported_block_lands_without_spinning_or_bouncing() {
        for (offset, speed) in [
            (0.0, 600.0),
            (0.0, 1200.0),
            (4.0, 600.0),
            (4.0, 1200.0),
            (-4.0, 1200.0),
        ] {
            let (mut world, mut schedule) = world_and_schedule();
            let (lower, upper, max_spin, max_rebound) =
                drop_on_block(&mut world, &mut schedule, offset, speed);
            println!(
                "landing offset={offset} speed={speed}: spin={max_spin} rebound={max_rebound}"
            );
            assert!(
                max_spin < 1.0,
                "offset {offset} speed {speed} spun at {max_spin}"
            );
            assert!(
                max_rebound < 10.0,
                "offset {offset} speed {speed} bounced at {max_rebound}"
            );
            assert!(
                (pos(&world, upper).x - (base_x() + offset)).abs() < 2.0,
                "offset {offset} speed {speed} moved"
            );
            assert!(
                (pos(&world, lower).x - base_x()).abs() < 1.0,
                "block below was disturbed"
            );
        }
    }

    #[test]
    fn landing_past_the_edge_still_topples() {
        let (mut world, mut schedule) = world_and_schedule();
        let (_, upper, _, _) = drop_on_block(&mut world, &mut schedule, 10.0, 600.0);
        assert!(
            world
                .get::<Transform>(upper)
                .is_none_or(|t| t.translation.y < TOWER_BASE_Y + 60.0)
        );
    }

    fn rotate_world(held_block: bool) -> (World, Schedule, Entity) {
        let mut world = World::new();
        world.init_resource::<ClientInput>();
        world.init_resource::<ServerInput>();
        let held = block(&mut world, Vec2::ZERO, Vec2::ZERO);
        if held_block {
            world.entity_mut(held).insert(BeingPlaced);
        }
        let mut schedule = Schedule::default();
        schedule.add_systems(rotate_block);
        (world, schedule, held)
    }

    #[test]
    fn one_press_rotates_a_quarter_turn_on_the_host() {
        let (mut world, mut schedule, held) = rotate_world(true);
        // The host gets the same press through both channels, sometimes a tick apart
        world.resource_mut::<ClientInput>().input = b'R';
        schedule.run(&mut world);
        world.resource_mut::<ServerInput>().input = b'R';
        schedule.run(&mut world);
        schedule.run(&mut world);
        assert!(
            (angle(&world, held) - std::f32::consts::FRAC_PI_2).abs() < 0.01,
            "angle {}",
            angle(&world, held)
        );
    }

    #[test]
    fn one_press_rotates_a_quarter_turn_when_both_channels_arrive_together() {
        let (mut world, mut schedule, held) = rotate_world(true);
        world.resource_mut::<ClientInput>().input = b'R';
        world.resource_mut::<ServerInput>().input = b'R';
        schedule.run(&mut world);
        schedule.run(&mut world);
        assert!(
            (angle(&world, held) - std::f32::consts::FRAC_PI_2).abs() < 0.01,
            "angle {}",
            angle(&world, held)
        );
    }

    #[test]
    fn rotate_with_no_held_block_does_not_carry_over() {
        let (mut world, mut schedule, held) = rotate_world(false);
        world.resource_mut::<ClientInput>().input = b'R';
        world.resource_mut::<ServerInput>().input = b'R';
        schedule.run(&mut world);
        world.entity_mut(held).insert(BeingPlaced);
        schedule.run(&mut world);
        assert!(
            angle(&world, held).abs() < 0.01,
            "stale R rotated the next block"
        );
    }
}
