use bevy::{
    color::palettes::css::*,
    ecs::{system::QueryLens, world},
    log,
    prelude::*,
};

use crate::{
    bounds2::Bounds2,
    coordinates::Coordinates,
    tile::Tile,
    tile_map::{SQUARE_COORDINATES, TileMap},
};

pub mod bounds2;
pub mod coordinates;
pub mod tile;
pub mod tile_map;

// minesweeper

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BoardPlugin)
        .add_systems(Startup, spawn)
        .run();
}

pub const TILE_SIZE: f32 = 32.0;
pub const TILE_SPACING: f32 = 2.0;

#[derive(Component)]
pub struct Board {
    pub tile_map: TileMap,
}

impl Board {
    pub fn reveal_empty_neighbors(
        &self,
        coordinates: Coordinates,
        tiles: &mut QueryLens<(Entity, &mut Tile, &GlobalTransform, &Coordinates, &Children)>,
        covered: &mut QueryLens<Entity>,
        board: &Board,
        commands: &mut Commands,
    ) {
        let visited = &mut vec![];
        board.tile_map.scan_map_at(coordinates).for_each(|coord| {
            if visited.contains(&coord) {
                return;
            }
            visited.push(coord);
            let tile = board.tile_map.get_tile_at_coords(coord);
            if tile.is_none() {
                return;
            }
            let tile = tile.unwrap();

            if tile.is_empty() {
                let clicked_tile = tiles
                    .query()
                    .iter()
                    .find(|(_, _, _, c, _)| **c == coord)
                    .map(|(e, _, _, _, _)| e);
                if clicked_tile.is_none() {
                    return;
                }
                let clicked_tile = clicked_tile.unwrap();
                let tiles_query = tiles.query();
                let (_, _, _, _, children) = tiles_query.get(clicked_tile).unwrap();
                board.reveal_tile(covered, children, commands);

                // Recursively reveal neighbors
                self.reveal_empty_neighbors(coord, tiles, covered, board, commands);
            }
        });
    }

    pub fn reveal_tile(
        &self,
        covered: &mut QueryLens<Entity>,
        tile_children: &Children,
        commands: &mut Commands,
    ) {
        for child in tile_children.iter() {
            if let Ok(cover) = covered.query().get(child) {
                commands.entity(cover).despawn();
            }
        }
    }

    pub fn find_colliding_tile(
        &self,
        point: Vec2,
        tiles: &mut QueryLens<(Entity, &GlobalTransform)>,
    ) -> Option<Entity> {
        for (entity, transform) in &mut tiles.query() {
            let bounds = Bounds2::from_center_size(
                transform.translation().truncate(),
                Vec2::splat(TILE_SIZE + TILE_SPACING),
            );

            if bounds.contains(point) {
                return Some(entity);
            }
        }

        None
    }
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

#[derive(Component)]
pub struct CoveredTile;

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::create_board)
            .add_systems(Update, Self::click_tile);
    }
}

impl BoardPlugin {
    pub fn reveal_tile() {}

    pub fn click_tile(
        mouse_input: Res<ButtonInput<MouseButton>>,
        mut tiles: Query<(Entity, &mut Tile, &GlobalTransform, &Coordinates, &Children)>,
        mut covered: Query<Entity, With<CoveredTile>>,
        mut sprites: Query<&mut Sprite>,
        windows: Query<&Window>,
        camera: Query<(&Camera, &GlobalTransform)>,
        mut commands: Commands,
        asset_server: ResMut<AssetServer>,
        board: Single<&Board>,
    ) {
        let window = windows.single().expect("No window found");
        let (camera, camera_transform) = camera.single().expect("No camera found");

        if !mouse_input.just_pressed(MouseButton::Left) {
            return;
        }

        let cursor = window.cursor_position();
        if cursor.is_none() {
            return;
        }
        let cursor = cursor.unwrap();

        let world_position = camera
            .viewport_to_world_2d(camera_transform, cursor)
            .expect("Failed to convert viewport to world");

        let clicked_tile = board.find_colliding_tile(world_position, &mut tiles.transmute_lens());
        if clicked_tile.is_none() {
            log::info!("No tile found at position {:?}", world_position);
            return;
        }

        let (entity, tile, _, coords, children) = tiles.get(clicked_tile.unwrap()).unwrap();
        board.reveal_tile(&mut covered.transmute_lens(), &children, &mut commands);

        match *tile {
            Tile::Empty => {
                log::info!(
                    "Safe! You clicked on an empty tile at ({}, {})",
                    coords.x,
                    coords.y
                );

                board.reveal_empty_neighbors(
                    *coords,
                    &mut tiles.transmute_lens(),
                    &mut covered.transmute_lens(),
                    &board,
                    &mut commands,
                );
            }
            Tile::Neighbour(n) => {
                log::info!(
                    "Safe! You clicked on a neighbor tile with {} bombs around at ({}, {})",
                    n,
                    coords.x,
                    coords.y
                );

                // reveal neighboring tiles if this tile was already revealed
            }
            Tile::Bomb => {
                log::info!(
                    "Boom! You clicked on a bomb at ({}, {})",
                    coords.x,
                    coords.y
                );

                let explosion: Handle<Image> = asset_server.load("icons/explosion.png");

                for child in children.iter() {
                    if let Ok(mut sprite) = sprites.get_mut(child) {
                        sprite.image = explosion.clone();
                    }
                }

                commands
                    .entity(entity)
                    .insert(Sprite::from_color(BLACK, Vec2::splat(TILE_SIZE)));

                // Despawn all CoveredTile entities to reveal the board
                for cover in &covered {
                    commands.entity(cover).despawn();
                }
            }
        }
    }

    pub fn create_board(mut commands: Commands, asset_server: ResMut<AssetServer>) {
        let font: Handle<Font> = asset_server.load("fonts/ChakraPetch-Regular.ttf");
        let bomb: Handle<Image> = asset_server.load("icons/bomb.png");

        let mut tile_map = TileMap::empty(20, 20);
        tile_map.set_bombs(50);
        log::info!("{}", tile_map.console_output());

        commands
            .spawn((
                Sprite::from_color(Color::WHITE, Vec2::ONE),
                Transform {
                    translation: Vec3::new(
                        -(tile_map.width as f32 * (TILE_SIZE + TILE_SPACING)) / 2.0,
                        -(tile_map.height as f32 * (TILE_SIZE + TILE_SPACING)) / 2.0,
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
                            x as f32 * (TILE_SIZE + TILE_SPACING),
                            y as f32 * (TILE_SIZE + TILE_SPACING),
                            0.0,
                        );

                        let box_size = Vec2::new(TILE_SIZE, TILE_SIZE);
                        commands
                            .spawn((
                                Sprite::from_color(GREY, box_size),
                                Transform::from_translation(position),
                                Name::new(format!("Tile ({}, {})", x, y)),
                                Coordinates { x, y },
                                tile,
                            ))
                            .with_children(|commands| {
                                commands.spawn((
                                    Sprite::from_color(LIGHT_GREY, box_size),
                                    Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                                    CoveredTile,
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
