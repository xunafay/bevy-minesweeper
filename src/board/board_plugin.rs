use bevy::{color::palettes::css::*, log, prelude::*};

use crate::{
    BoardSettings, UiSettings,
    board::{
        board::Board,
        coordinates::Coordinates,
        tile::{tile_state::TileState, tile_type::TileType},
        tile_map::TileMap,
    },
    utils::app_state::AppState,
};

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state::<AppState>(AppState::default())
            .add_systems(
                OnEnter(AppState::InGame),
                (Self::clear_board, Self::create_board).chain(),
            )
            .add_systems(
                Update,
                (
                    Self::left_click_tile,
                    Self::right_click_tile,
                    Self::victory_validation,
                    Self::defeat_validation,
                    Self::update_board,
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

impl BoardPlugin {
    pub fn victory_validation(board: Single<&Board>, mut next_state: ResMut<NextState<AppState>>) {
        if board.tile_map.has_won() {
            next_state.set(AppState::Victory);
        }
    }

    pub fn right_click_tile(
        mouse_input: Res<ButtonInput<MouseButton>>,
        windows: Query<&Window>,
        camera: Query<(&Camera, &GlobalTransform)>,
        mut tiles: Query<(Entity, &GlobalTransform)>,
        mut board: Single<&mut Board>,
        ui_settings: Res<UiSettings>,
    ) {
        if !mouse_input.just_pressed(MouseButton::Right) {
            return;
        }
        let window = windows.single().expect("No window found");
        let (camera, camera_transform) = camera.single().expect("No camera found");

        let cursor = window.cursor_position();
        if cursor.is_none() {
            return;
        }
        let cursor = cursor.unwrap();

        let world_position = camera
            .viewport_to_world_2d(camera_transform, cursor)
            .expect("Failed to convert viewport to world");

        let Some((_, coords)) = board.find_colliding_tile_coords(
            world_position,
            &mut tiles.transmute_lens(),
            &ui_settings,
        ) else {
            log::info!("No tile found at position {:?}", world_position);
            return;
        };

        if let Some(tile) = board.tile_map.at_mut(coords) {
            if tile.state == TileState::Revealed || tile.state == TileState::Exploded {
                return;
            }

            tile.toggle_flag();
        }
    }

    pub fn left_click_tile(
        mouse_input: Res<ButtonInput<MouseButton>>,
        windows: Query<&Window>,
        camera: Query<(&Camera, &GlobalTransform)>,
        mut tiles: Query<(Entity, &GlobalTransform, &Coordinates)>,
        mut board: Single<&mut Board>,
        ui_settings: Res<crate::UiSettings>,
    ) {
        if !mouse_input.just_pressed(MouseButton::Left) {
            return;
        }

        let window = windows.single().expect("No window found");
        let (camera, camera_transform) = camera.single().expect("No camera found");

        let cursor = window.cursor_position();
        if cursor.is_none() {
            return;
        }
        let cursor = cursor.unwrap();

        let world_position = camera
            .viewport_to_world_2d(camera_transform, cursor)
            .expect("Failed to convert viewport to world");

        let Some((_, coords)) = board.find_colliding_tile_coords(
            world_position,
            &mut tiles.transmute_lens(),
            &ui_settings,
        ) else {
            log::info!("No tile found at position {:?}", world_position);
            return;
        };

        if let Some(tile) = board.tile_map.at_mut(coords) {
            if tile.state == TileState::Flagged || tile.state == TileState::Exploded {
                return;
            }

            if tile.state == TileState::Revealed {
                let revealed_bombs = board.tile_map.reveal_neighbors(coords);
                if !revealed_bombs.is_empty() {
                    log::info!(
                        "Revealed {} bombs around {:?}",
                        revealed_bombs.len(),
                        coords
                    );
                }
                return;
            }

            tile.reveal();

            match tile.r#type {
                TileType::Empty => {
                    board
                        .tile_map
                        .reveal_empty_neighbors(coords, &mut Vec::new());
                }
                _ => {}
            }
        }
    }

    pub fn clear_board(mut commands: Commands, board_query: Query<Entity, With<Board>>) {
        for entity in &board_query {
            commands.entity(entity).despawn();
        }
    }

    pub fn defeat_validation(board: Single<&Board>, mut next_state: ResMut<NextState<AppState>>) {
        if board.tile_map.has_lost() {
            next_state.set(AppState::Defeat);
        }
    }

    pub fn update_board(
        board: Single<&mut Board>,
        mut tile_background: Query<
            (
                Entity,
                &GlobalTransform,
                &Coordinates,
                &mut Sprite,
                &Children,
                &TileBackground,
            ),
            Without<TileForeground>,
        >,
        mut tile_foregrounds: Query<&mut Sprite, With<TileForeground>>,
        asset_server: ResMut<AssetServer>,
        mut commands: Commands,
    ) {
        let explosion: Handle<Image> = asset_server.load("icons/explosion.png");
        let flag: Handle<Image> = asset_server.load("icons/flag.png");
        let cover: Handle<Image> = asset_server.load("icons/cover.png");
        let uncovered: Handle<Image> = asset_server.load("icons/uncovered.png");
        let bomb: Handle<Image> = asset_server.load("icons/bomb.png");
        let font: Handle<Font> = asset_server.load("fonts/ChakraPetch-Regular.ttf");

        for (entity, _, coords, mut sprite, children, _) in &mut tile_background {
            if let Some(tile) = board.tile_map.at(*coords) {
                let mut tile_foreground = tile_foregrounds
                    .get_mut(children[0])
                    .expect("Failed to get tile top sprite");

                match tile.state {
                    TileState::Hidden => sprite.image = cover.clone(),
                    TileState::Flagged => tile_foreground.image = flag.clone(),
                    TileState::Exploded => tile_foreground.image = explosion.clone(),
                    TileState::Revealed => {
                        sprite.image = uncovered.clone();
                        match tile.r#type {
                            TileType::Bomb => tile_foreground.image = bomb.clone(),
                            TileType::Neighbour(n) => {
                                commands.entity(entity).with_child((
                                    Text2d::new(n.to_string()),
                                    TextColor(match n {
                                        1 => BLUE.into(),
                                        2 => GREEN.into(),
                                        3 => ORANGE.into(),
                                        _ => RED.into(),
                                    }),
                                    TextFont::from_font(font.clone()).with_font_size(24.0),
                                    TextLayout::new_with_justify(JustifyText::Center),
                                ));
                            }
                            TileType::Empty => {}
                        }
                    }
                }
            }
        }
    }

    pub fn create_board(
        mut commands: Commands,
        asset_server: ResMut<AssetServer>,
        ui_settings: Res<UiSettings>,
        board_settings: Res<BoardSettings>,
    ) {
        let cover: Handle<Image> = asset_server.load("icons/cover.png");

        let mut tile_map = TileMap::empty(board_settings.board_width, board_settings.board_height);
        tile_map.set_bombs(board_settings.mine_count);
        log::info!("{}", tile_map.console_output());
        log::info!(
            "Board size: {}x{}, Bombs: {}",
            board_settings.board_width,
            board_settings.board_height,
            board_settings.mine_count
        );
        log::info!("Tile size: {}", ui_settings.tile_size);

        commands
            .spawn((
                Sprite::from_color(Color::WHITE, Vec2::ONE),
                Transform {
                    translation: Vec3::new(
                        -(tile_map.width as f32
                            * (ui_settings.tile_size + ui_settings.tile_spacing))
                            / 2.0,
                        -(tile_map.height as f32
                            * (ui_settings.tile_size + ui_settings.tile_spacing))
                            / 2.0,
                        0.0,
                    ),
                    ..Default::default()
                },
                Board {
                    tile_map: tile_map.clone(),
                },
            ))
            .with_children(|commands| {
                for y in 0..tile_map.height {
                    for x in 0..tile_map.width {
                        let tile = tile_map.map[y as usize][x as usize];
                        let position = Vec3::new(
                            x as f32 * (ui_settings.tile_size + ui_settings.tile_spacing),
                            y as f32 * (ui_settings.tile_size + ui_settings.tile_spacing),
                            0.0,
                        );

                        let box_size = Vec2::new(ui_settings.tile_size, ui_settings.tile_size);
                        commands
                            .spawn((
                                Sprite {
                                    custom_size: Some(box_size),
                                    image: cover.clone(),
                                    ..Default::default()
                                },
                                TileBackground,
                                Transform::from_translation(position),
                                Name::new(format!("Tile ({}, {})", x, y)),
                                Coordinates { x, y },
                            ))
                            .with_children(|commands| {
                                commands.spawn((
                                    Sprite {
                                        custom_size: Some(box_size),
                                        ..Default::default()
                                    },
                                    TileForeground,
                                    Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                                ));
                            });
                    }
                }
            });
    }
}

#[derive(Component)]
pub struct TileBackground;

#[derive(Component)]
pub struct TileForeground;
