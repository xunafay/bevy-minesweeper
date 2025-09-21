use bevy::prelude::*;

#[derive(Resource)]
pub struct UiSettings {
    pub tile_size: f32,
    pub tile_spacing: f32,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            tile_size: 32.0,
            tile_spacing: 0.0,
        }
    }
}
