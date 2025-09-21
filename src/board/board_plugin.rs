use bevy::{color::palettes::css::*, ecs::system::QueryLens, log, prelude::*};

use crate::{
    BoardSettings, UiSettings,
    board::{
        board::Board,
        board_changed::BoardChanged,
        coordinates::Coordinates,
        sprites::Sprites,
        tile::{tile_state::TileState, tile_type::TileType},
        tile_map::TileMap,
    },
    utils::app_state::AppState,
};

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_state::<AppState>(AppState::default())
            .add_systems(OnEnter(AppState::MainMenu), Self::clear_board)
            .add_event::<BoardChanged>()
            .add_systems(
                OnEnter(AppState::InGame),
                (Self::clear_board, Self::create_board).chain(),
            )
            .add_systems(
                Update,
                (
                    (Self::find_safe_start, Self::left_click_tile).chain(),
                    Self::right_click_tile,
                    Self::victory_validation,
                    Self::defeat_validation,
                    Self::update_board,
                )
                    .run_if(in_state(AppState::InGame)),
            )
            .add_systems(OnEnter(AppState::Defeat), Self::update_board);
    }
}

impl BoardPlugin {
    fn get_tile_coords(
        windows: Query<&Window>,
        mut camera_query_lens: QueryLens<(&Camera, &GlobalTransform)>,
        mut tiles: QueryLens<(Entity, &GlobalTransform, &Coordinates)>,
        board: &Board,
        ui_settings: &Res<UiSettings>,
    ) -> Option<Coordinates> {
        let window = windows.single().expect("No window found");
        let camera_query = camera_query_lens.query();
        let (camera, camera_transform) = camera_query.single().expect("No camera found");
        let cursor = window.cursor_position();
        if cursor.is_none() {
            return None;
        }
        let cursor = cursor.unwrap();
        let world_position = camera
            .viewport_to_world_2d(camera_transform, cursor)
            .expect("Failed to convert viewport to world");
        let Some((_, coords)) = board.find_colliding_tile_coords(
            world_position,
            &mut tiles.query().transmute_lens(),
            &ui_settings,
        ) else {
            log::info!("No tile found at position {:?}", world_position);
            return None;
        };

        Some(coords.clone())
    }

    /// When the first tile is clicked, generate new boards until there is at least one empty tile under the cursor
    pub fn find_safe_start(
        mouse_input: Res<ButtonInput<MouseButton>>,
        windows: Query<&Window>,
        mut camera: Query<(&Camera, &GlobalTransform)>,
        board: Single<&mut Board>,
        mut tiles: Query<(Entity, &GlobalTransform, &Coordinates)>,
        ui_settings: Res<UiSettings>,
    ) {
        if !board.tile_map.is_pristine() {
            return;
        }

        if !mouse_input.just_pressed(MouseButton::Left) {
            return;
        }

        let mut board = board.into_inner();
        let Some(coords) = Self::get_tile_coords(
            windows,
            camera.transmute_lens(),
            tiles.transmute_lens(),
            &board,
            &ui_settings,
        ) else {
            return;
        };

        while let Some(tile) = board.tile_map.at(&coords) {
            if tile.r#type.is_empty() {
                break;
            }

            log::info!("Regenerating board for safe start...");
            let bomb_count = board.tile_map.bomb_count;
            board.tile_map = TileMap::empty(board.tile_map.width, board.tile_map.height);
            board.tile_map.set_bombs(bomb_count);
        }
    }

    pub fn victory_validation(board: Single<&Board>, mut next_state: ResMut<NextState<AppState>>) {
        if board.tile_map.has_won() {
            next_state.set(AppState::Victory);
        }
    }

    pub fn right_click_tile(
        mouse_input: Res<ButtonInput<MouseButton>>,
        windows: Query<&Window>,
        mut camera: Query<(&Camera, &GlobalTransform)>,
        mut tiles: Query<(Entity, &GlobalTransform, &Coordinates)>,
        mut board: Single<&mut Board>,
        ui_settings: Res<UiSettings>,
        mut board_changed_event: EventWriter<BoardChanged>,
    ) {
        if !mouse_input.just_pressed(MouseButton::Right) {
            return;
        }

        let Some(coords) = Self::get_tile_coords(
            windows,
            camera.transmute_lens(),
            tiles.transmute_lens(),
            &board,
            &ui_settings,
        ) else {
            return;
        };

        if let Some(tile) = board.tile_map.at_mut(&coords) {
            if tile.state == TileState::Revealed || tile.state == TileState::Exploded {
                return;
            }

            tile.toggle_flag();
            board_changed_event.write(BoardChanged);
        }
    }

    pub fn left_click_tile(
        mouse_input: Res<ButtonInput<MouseButton>>,
        windows: Query<&Window>,
        mut camera: Query<(&Camera, &GlobalTransform)>,
        mut tiles: Query<(Entity, &GlobalTransform, &Coordinates)>,
        mut board: Single<&mut Board>,
        ui_settings: Res<crate::UiSettings>,
        mut board_changed_event: EventWriter<BoardChanged>,
    ) {
        if !mouse_input.just_pressed(MouseButton::Left) {
            return;
        }

        let Some(coords) = Self::get_tile_coords(
            windows,
            camera.transmute_lens(),
            tiles.transmute_lens(),
            &board,
            &ui_settings,
        ) else {
            return;
        };

        if let Some(tile) = board.tile_map.at_mut(&coords) {
            if tile.state == TileState::Flagged || tile.state == TileState::Exploded {
                return;
            }

            board_changed_event.write(BoardChanged);
            if tile.state == TileState::Revealed {
                let _ = board.tile_map.reveal_neighbors(coords);
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

    pub fn defeat_validation(
        mut board: Single<&mut Board>,
        mut next_state: ResMut<NextState<AppState>>,
    ) {
        if board.tile_map.has_lost() {
            board.tile_map.reveal_all(false);
            next_state.set(AppState::Defeat);
        }
    }

    pub fn update_board(
        board: Single<&mut Board>,
        tile_background: Query<
            (Entity, &GlobalTransform, &Coordinates, &Children),
            With<TileImageState>,
        >,
        mut tile_foregrounds: Query<Entity, With<TileImageMarker>>,
        ui_settings: Res<UiSettings>,
        mut commands: Commands,
        sprites: Res<Sprites>,
        mut change_reader: EventReader<BoardChanged>,
    ) {
        if change_reader.is_empty() {
            return;
        }
        change_reader.clear();
        log::info!("Updating board visuals...");

        let box_size = Vec2::new(ui_settings.tile_size, ui_settings.tile_size);

        for (image_state_entity, _, coords, children) in &tile_background {
            if let Some(tile) = board.tile_map.at(coords) {
                let image_marker_entity = tile_foregrounds
                    .get_mut(children[0])
                    .expect("Failed to get tile top sprite");

                match tile.state {
                    TileState::Hidden => {
                        commands.entity(image_state_entity).insert(Sprite {
                            custom_size: Some(box_size),
                            image: sprites.cover.clone(),
                            ..Default::default()
                        });

                        commands.entity(image_marker_entity).remove::<Sprite>();
                    }
                    TileState::Flagged => {
                        commands.entity(image_marker_entity).insert(Sprite {
                            custom_size: Some(box_size),
                            image: sprites.flag.clone(),
                            ..Default::default()
                        });
                    }
                    TileState::Exploded => {
                        commands.entity(image_marker_entity).insert(Sprite {
                            custom_size: Some(box_size),
                            image: sprites.explosion.clone(),
                            ..Default::default()
                        });
                    }
                    TileState::Revealed => {
                        commands.entity(image_state_entity).insert(Sprite {
                            custom_size: Some(box_size),
                            image: sprites.uncovered.clone(),
                            ..Default::default()
                        });

                        match tile.r#type {
                            TileType::Bomb => {
                                commands.entity(image_marker_entity).insert(Sprite {
                                    custom_size: Some(box_size),
                                    image: sprites.bomb.clone(),
                                    ..Default::default()
                                });
                            }
                            TileType::Neighbour(n) => {
                                commands.entity(image_marker_entity).with_child((
                                    Text2d::new(n.to_string()),
                                    TextColor(match n {
                                        1 => BLUE.into(),
                                        2 => GREEN.into(),
                                        3 => ORANGE.into(),
                                        _ => RED.into(),
                                    }),
                                    TextFont::from_font(sprites.font.clone()).with_font_size(24.0),
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
        let explosion: Handle<Image> = asset_server.load("icons/explosion.png");
        let flag: Handle<Image> = asset_server.load("icons/flag.png");
        let uncovered: Handle<Image> = asset_server.load("icons/uncovered.png");
        let bomb: Handle<Image> = asset_server.load("icons/bomb.png");
        let font: Handle<Font> = asset_server.load("fonts/ChakraPetch-Regular.ttf");
        let cover: Handle<Image> = asset_server.load("icons/cover.png");

        commands.insert_resource(Sprites {
            explosion: explosion.clone(),
            flag: flag.clone(),
            cover: cover.clone(),
            uncovered: uncovered.clone(),
            bomb: bomb.clone(),
            font: font.clone(),
        });

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
                                TileImageState,
                                Transform::from_translation(position),
                                Name::new(format!("Tile ({}, {})", x, y)),
                                Coordinates { x, y },
                            ))
                            .with_children(|commands| {
                                commands.spawn((
                                    TileImageMarker,
                                    Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                                    Visibility::default(),
                                ));
                            });
                    }
                }
            });
    }
}

#[derive(Component)]
pub struct TileImageState;

#[derive(Component)]
pub struct TileImageMarker;
