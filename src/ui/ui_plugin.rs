use bevy::{log, prelude::*, state::commands};

use crate::{board::settings::BoardSettings, ui::menu_data::MenuData, utils::app_state::AppState};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, Self::menu)
            .add_systems(OnEnter(AppState::MainMenu), Self::setup_main_menu)
            .add_systems(OnExit(AppState::MainMenu), Self::cleanup_menu)
            .add_systems(OnExit(AppState::Defeat), Self::cleanup_menu)
            .add_systems(OnEnter(AppState::Defeat), Self::setup_defeat_menu)
            .add_systems(OnExit(AppState::Victory), Self::cleanup_menu)
            .add_systems(OnEnter(AppState::Victory), Self::setup_victory_menu);
    }
}

#[derive(Component)]
pub struct MenuRoot;

#[derive(Component, PartialEq, Eq, Debug)]
pub enum ButtonType {
    Easy,
    Medium,
    Hard,
    MainMenu,
}

impl UiPlugin {
    pub fn menu(
        mut next_state: ResMut<NextState<AppState>>,
        current_state: Res<State<AppState>>,
        mut interaction_query: Query<
            (&Interaction, &ButtonType),
            (Changed<Interaction>, With<Button>),
        >,
        mut commands: Commands,
    ) {
        for (interaction, button_type) in &mut interaction_query {
            match *interaction {
                Interaction::Pressed => match current_state.get() {
                    AppState::MainMenu => {
                        log::info!(
                            "Starting game from main menu with difficulty: {:?}",
                            button_type
                        );
                        match button_type {
                            ButtonType::Easy => {
                                commands.insert_resource(BoardSettings {
                                    board_width: 9,
                                    board_height: 9,
                                    mine_count: 10,
                                });
                            }
                            ButtonType::Medium => {
                                commands.insert_resource(BoardSettings {
                                    board_width: 16,
                                    board_height: 16,
                                    mine_count: 40,
                                });
                            }
                            ButtonType::Hard => {
                                commands.insert_resource(BoardSettings {
                                    board_width: 30,
                                    board_height: 16,
                                    mine_count: 99,
                                });
                            }
                            _ => {}
                        }

                        next_state.set(AppState::InGame);
                    }
                    AppState::InGame => {}
                    AppState::Victory => {
                        log::info!("Restarting game from victory menu");
                        next_state.set(AppState::MainMenu);
                    }
                    AppState::Defeat => {
                        log::info!("Restarting game from defeat menu");
                        next_state.set(AppState::MainMenu);
                    }
                },
                Interaction::Hovered => {}
                Interaction::None => {}
            }
        }
    }

    pub fn cleanup_menu(mut commands: Commands, menu_data: Query<Entity, With<MenuRoot>>) {
        log::info!("Cleaning up menu");
        for entity in &menu_data {
            commands.entity(entity).despawn();
        }
    }

    pub fn setup_victory_menu(mut commands: Commands) {
        commands.spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Start,
                ..default()
            },
            MenuRoot,
            children![(
                Text::new("Victory!"),
                TextFont {
                    font_size: 50.0,
                    ..default()
                },
                TextColor(Color::srgb(0.1, 0.9, 0.1)),
            )],
        ));

        commands.spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::End,
                ..default()
            },
            MenuRoot,
            children![(
                Button,
                ButtonType::MainMenu,
                Node {
                    width: Val::Px(150.),
                    height: Val::Px(65.),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                children![(
                    Text::new("Restart"),
                    TextFont {
                        font_size: 33.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                )]
            )],
        ));
    }

    pub fn setup_defeat_menu(mut commands: Commands) {
        commands.spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Start,
                ..default()
            },
            MenuRoot,
            children![(
                Text::new("Defeat!"),
                TextFont {
                    font_size: 50.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.1, 0.1)),
            )],
        ));

        commands.spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::End,
                ..default()
            },
            MenuRoot,
            children![(
                Button,
                ButtonType::MainMenu,
                Node {
                    width: Val::Px(150.),
                    height: Val::Px(65.),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                children![(
                    Text::new("Main Menu"),
                    TextFont {
                        font_size: 33.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                )]
            )],
        ));
    }

    pub fn setup_main_menu(mut commands: Commands) {
        commands.spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                display: Display::Flex,
                row_gap: Val::Px(20.),
                ..default()
            },
            MenuRoot,
            children![
                (
                    Button,
                    ButtonType::Easy,
                    Node {
                        width: Val::Px(150.),
                        height: Val::Px(65.),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    children![(
                        Text::new("Easy"),
                        TextFont {
                            font_size: 33.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    )]
                ),
                (
                    Button,
                    ButtonType::Medium,
                    Node {
                        width: Val::Px(150.),
                        height: Val::Px(65.),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    children![(
                        Text::new("Medium"),
                        TextFont {
                            font_size: 33.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    )]
                ),
                (
                    Button,
                    ButtonType::Hard,
                    Node {
                        width: Val::Px(150.),
                        height: Val::Px(65.),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    children![(
                        Text::new("Hard"),
                        TextFont {
                            font_size: 33.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    )]
                ),
            ],
        ));
    }
}
