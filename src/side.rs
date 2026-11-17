use bevy::prelude::*;
use crate::{GameState, WIN_W};

// Tower geometry shared by spawning, placement bounds and ground collision
pub const LEFT: f32 = -WIN_W / 2.0;
pub const RIGHT: f32 = WIN_W / 2.0;
pub const EDGE_TO_TOWER: f32 = 48.0; // 48px between base and border
pub const BASE_WIDTH: f32 = 256.0;
use crate::server::ClientInput;

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

    /// Player 1 builds on the left tower, player 2 on the right
    pub fn for_player(player: u8) -> Option<Side> {
        match player {
            1 => Some(Side::Left),
            2 => Some(Side::Right),
            _ => None,
        }
    }
}

/// Which side the local player controls. Gameplay only reads this;
/// only the side-assignment code (currently debug_toggle_side) writes it.
#[derive(Resource, Default, Deref, DerefMut)]
pub struct LocalSide(pub Side);

pub struct SidePlugin;
impl Plugin for SidePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LocalSide>();
    }
}
