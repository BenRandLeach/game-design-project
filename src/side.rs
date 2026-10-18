use bevy::prelude::*;
use crate::{GameState, WIN_W};

// Tower geometry shared by spawning, placement bounds and ground collision
pub const LEFT: f32 = -WIN_W / 2.0;
pub const RIGHT: f32 = WIN_W / 2.0;
pub const EDGE_TO_TOWER: f32 = 48.0; // 48px between base and border
pub const BASE_WIDTH: f32 = 256.0;

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Side {
    #[default]
    Left,
    Right,
}

impl Side {
    pub fn opposite(self) -> Side {
        match self {
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        }
    }

    /// x of the left edge of this side's tower base
    pub fn base_left(self) -> f32 {
        match self {
            Side::Left => LEFT + EDGE_TO_TOWER,
            Side::Right => RIGHT - EDGE_TO_TOWER - BASE_WIDTH,
        }
    }

    /// x of the right edge of this side's tower base
    pub fn base_right(self) -> f32 {
        self.base_left() + BASE_WIDTH
    }

    pub fn base_center(self) -> f32 {
        self.base_left() + BASE_WIDTH / 2.0
    }
}

/// Which side the local player controls. Gameplay only reads this;
/// only the side-assignment code (currently debug_toggle_side) writes it.
#[derive(Resource, Default, Deref, DerefMut)]
pub struct LocalSide(pub Side);

pub struct SidePlugin;
impl Plugin for SidePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LocalSide>()
            .add_systems(Update, debug_toggle_side.run_if(in_state(GameState::Game)));
    }
}

// TEMP: replace with network side assignment (e.g. set LocalSide in client::connect)
fn debug_toggle_side(keys: Res<ButtonInput<KeyCode>>, mut side: ResMut<LocalSide>) {
    if keys.just_pressed(KeyCode::KeyO) {
        side.0 = side.0.opposite();
        info!("Switched to {:?} side", side.0);
    }
}
