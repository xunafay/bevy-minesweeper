#[cfg(feature = "debug")]
use bevy::dev_tools::fps_overlay::FpsOverlayPlugin;
use bevy::{prelude::*, window::PresentMode};

use crate::{
    board::{board_plugin::BoardPlugin, settings::BoardSettings},
    ui::{settings::UiSettings, ui_plugin::UiPlugin},
};

pub mod board;
pub mod ui;
pub mod utils;

fn main() {
    let mut app = App::new();

    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Minesweeper".to_string(),
                    present_mode: PresentMode::AutoNoVsync,
                    ..Default::default()
                }),
                ..Default::default()
            }),
    )
    .insert_resource(UiSettings::default())
    .insert_resource(ClearColor(Color::srgb(0.3, 0.3, 0.3)))
    .add_plugins(UiPlugin)
    .add_plugins(BoardPlugin)
    .add_systems(Startup, spawn);

    #[cfg(feature = "debug")]
    let app = app.add_plugins(FpsOverlayPlugin::default());

    app.run();
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
