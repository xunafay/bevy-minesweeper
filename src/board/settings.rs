use bevy::prelude::*;

#[derive(Resource)]
pub struct BoardSettings {
    pub board_width: u16,
    pub board_height: u16,
    pub mine_count: u16,
}
