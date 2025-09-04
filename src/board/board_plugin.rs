use bevy::{color::palettes::css::*, log, prelude::*, render::render_resource::Texture};

use crate::{
    BoardSettings, UiSettings,
    board::{
        board::Board, coordinates::Coordinates, flag::Flag, tile::Tile, tile_cover::TileCover,
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
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

impl BoardPlugin {
    pub fn victory_validation(
        mut tiles: Query<(&Coordinates, &Children), With<Tile>>,
        covers: Query<Entity, With<TileCover>>,
        flags: Query<Entity, With<Flag>>,
        board: Single<&Board>,
        mut next_state: ResMut<NextState<AppState>>,
    ) {
        let mut covered_tiles = 0;
        let mut bomb_tiles = 0;
        let mut flagged_bombs = 0;

        for (coords, children) in tiles.iter_mut() {
            let is_covered = children.iter().any(|c| covers.get(c).is_ok());
            let has_flag = children.iter().any(|c| flags.get(c).is_ok());
            let tile = board.tile_map.get_tile_at_coords(*coords).unwrap();
            if is_covered {
                covered_tiles += 1;
            }
            if tile == Tile::Bomb {
                bomb_tiles += 1;
                if has_flag {
                    flagged_bombs += 1;
                }
            }
        }

        if covered_tiles == bomb_tiles && flagged_bombs == bomb_tiles && bomb_tiles > 0 {
            next_state.set(AppState::Victory);
        }
    }

    pub fn right_click_tile(
        mouse_input: Res<ButtonInput<MouseButton>>,
        windows: Query<&Window>,
        camera: Query<(&Camera, &GlobalTransform)>,
        mut tiles: Query<(Entity, &GlobalTransform, &Children), With<Tile>>,
        covers: Query<Entity, With<TileCover>>,
        flags: Query<Entity, With<Flag>>,
        board: Single<&Board>,
        asset_server: Res<AssetServer>,
        mut commands: Commands,
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

        let clicked_tile =
            board.find_colliding_tile(world_position, &mut tiles.transmute_lens(), &ui_settings);
        if clicked_tile.is_none() {
            log::info!("No tile found at position {:?}", world_position);
            return;
        }

        let (entity, _, children) = tiles.get(clicked_tile.unwrap()).unwrap();
        if children.iter().all(|c| covers.get(c).is_err()) {
            return;
        }

        let flag: Handle<Image> = asset_server.load("icons/flag.png");

        for child in children.iter() {
            if let Ok(flag_entity) = flags.get(child) {
                commands.entity(flag_entity).despawn();
                return;
            }
        }

        let flag_entity = commands
            .spawn((
                Sprite::from_image(flag),
                Flag,
                Transform {
                    translation: Vec3::new(0.0, 0.0, 2.0),
                    ..Default::default()
                },
            ))
            .id();
        commands.entity(entity).add_child(flag_entity);
    }

    pub fn left_click_tile(
        mouse_input: Res<ButtonInput<MouseButton>>,
        mut tiles: Query<(Entity, &mut Tile, &GlobalTransform, &Coordinates, &Children)>,
        mut covered: Query<Entity, With<TileCover>>,
        mut sprites: Query<&mut Sprite>,
        windows: Query<&Window>,
        camera: Query<(&Camera, &GlobalTransform)>,
        mut commands: Commands,
        asset_server: Res<AssetServer>,
        board: Single<&Board>,
        ui_settings: Res<crate::UiSettings>,
        mut next_state: ResMut<NextState<AppState>>,
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

        let clicked_tile =
            board.find_colliding_tile(world_position, &mut tiles.transmute_lens(), &ui_settings);
        if clicked_tile.is_none() {
            log::info!("No tile found at position {:?}", world_position);
            return;
        }

        let (entity, tile, _, coords, children) = tiles.get(clicked_tile.unwrap()).unwrap();

        if mouse_input.just_pressed(MouseButton::Left) {
            board.reveal_tile(&mut covered.transmute_lens(), &children, &mut commands);

            match *tile {
                Tile::Empty => {
                    board.reveal_empty_neighbors(
                        *coords,
                        &mut tiles.transmute_lens(),
                        &mut covered.transmute_lens(),
                        &board,
                        &mut commands,
                        &mut Vec::new(),
                    );
                }
                Tile::Neighbour(n) => {
                    // reveal neighboring tiles if this tile was already revealed
                }
                Tile::Bomb => {
                    let explosion: Handle<Image> = asset_server.load("icons/explosion.png");

                    for child in children.iter() {
                        if let Ok(mut sprite) = sprites.get_mut(child) {
                            sprite.image = explosion.clone();
                        }
                    }

                    commands.entity(entity).insert(Sprite::from_color(
                        BLACK,
                        Vec2::splat(ui_settings.tile_size),
                    ));

                    // Despawn all CoveredTile entities to reveal the board
                    for cover in &covered {
                        commands.entity(cover).despawn();
                    }

                    next_state.set(AppState::Defeat);
                }
            }
        }
    }

    pub fn clear_board(mut commands: Commands, board_query: Query<Entity, With<Board>>) {
        for entity in &board_query {
            commands.entity(entity).despawn_recursive();
        }
    }

    pub fn create_board(
        mut commands: Commands,
        asset_server: ResMut<AssetServer>,
        ui_settings: Res<UiSettings>,
        board_settings: Res<BoardSettings>,
    ) {
        let font: Handle<Font> = asset_server.load("fonts/ChakraPetch-Regular.ttf");
        let bomb: Handle<Image> = asset_server.load("icons/bomb.png");
        let cover: Handle<Image> = asset_server.load("icons/cover.png");
        let uncovered: Handle<Image> = asset_server.load("icons/uncovered.png");

        let mut tile_map = TileMap::empty(board_settings.board_width, board_settings.board_height);
        tile_map.set_bombs(board_settings.mine_count);
        log::info!("{}", tile_map.console_output());

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

                        let box_size = Vec2::new(ui_settings.tile_size, ui_settings.tile_spacing);
                        commands
                            .spawn((
                                Sprite {
                                    custom_size: Some(box_size),
                                    image: uncovered.clone(),
                                    ..Default::default()
                                },
                                Transform::from_translation(position),
                                Name::new(format!("Tile ({}, {})", x, y)),
                                Coordinates { x, y },
                                tile,
                            ))
                            .with_children(|commands| {
                                commands.spawn((
                                    Sprite::from_image(cover.clone()),
                                    Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                                    TileCover,
                                ));
                                match tile {
                                    Tile::Bomb => {
                                        commands.spawn(Sprite::from_image(bomb.clone()));
                                    }
                                    Tile::Neighbour(n) => {
                                        commands.spawn((
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
                                    _ => {}
                                };
                            });
                    }
                }
            });
    }
}
