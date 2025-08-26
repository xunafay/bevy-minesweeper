use bevy::prelude::*;

use crate::{ui::menu_data::MenuData, utils::app_state::AppState};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, Self::menu.run_if(in_state(AppState::MainMenu)))
            .add_systems(OnEnter(AppState::MainMenu), Self::setup_menu)
            .add_systems(OnExit(AppState::MainMenu), Self::cleanup_menu);
    }
}

impl UiPlugin {
    pub fn menu(
        mut next_state: ResMut<NextState<AppState>>,
        mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
    ) {
        for interaction in &mut interaction_query {
            match *interaction {
                Interaction::Pressed => {
                    next_state.set(AppState::InGame);
                }
                Interaction::Hovered => {}
                Interaction::None => {}
            }
        }
    }

    pub fn setup_menu(mut commands: Commands) {
        let button_entity = commands
            .spawn((
                Node {
                    // center button
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![(
                    Button,
                    Node {
                        width: Val::Px(150.),
                        height: Val::Px(65.),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    children![(
                        Text::new("Play"),
                        TextFont {
                            font_size: 33.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    )]
                )],
            ))
            .id();

        commands.insert_resource(MenuData { button_entity });
    }
    pub fn cleanup_menu(mut commands: Commands, menu_data: Res<MenuData>) {
        commands.entity(menu_data.button_entity).despawn();
    }
}
