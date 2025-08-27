use bevy::prelude::*;

use crate::{board::board_plugin::BoardPlugin, ui::ui_plugin::UiPlugin};

pub mod board;
pub mod ui;
pub mod utils;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(UiPlugin)
        .add_plugins(BoardPlugin)
        .add_systems(Startup, spawn)
        .run();
}

pub const TILE_SIZE: f32 = 32.0;
pub const TILE_SPACING: f32 = 0.0;

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
