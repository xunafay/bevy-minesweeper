use bevy::prelude::*;

use crate::{
    board::{board_plugin::BoardPlugin, settings::BoardSettings},
    ui::{settings::UiSettings, ui_plugin::UiPlugin},
};

pub mod board;
pub mod ui;
pub mod utils;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(UiSettings {
            tile_size: 32.0,
            tile_spacing: 0.0,
        })
        .insert_resource(BoardSettings {
            board_width: 16,
            board_height: 16,
            mine_count: 40,
        })
        .insert_resource(ClearColor(Color::srgb(0.3, 0.3, 0.3)))
        .add_plugins(UiPlugin)
        .add_plugins(BoardPlugin)
        .add_systems(Startup, spawn)
        .run();
}

pub fn spawn(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Transform {
            translation: Vec3::new(0.0, 0.0, 100.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
            ..Default::default()
        },
    ));
}
