use bevy::prelude::*;

#[derive(Resource)]
pub struct UiSettings {
    pub tile_size: f32,
    pub tile_spacing: f32,
}
