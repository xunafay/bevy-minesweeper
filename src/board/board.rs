use bevy::{ecs::system::QueryLens, prelude::*};

use crate::{
    TILE_SIZE, TILE_SPACING,
    board::{coordinates::Coordinates, tile::Tile, tile_map::TileMap},
    utils::bounds2::Bounds2,
};

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
        visited: &mut Vec<Coordinates>,
    ) {
        visited.push(coordinates);
        board.tile_map.scan_map_at(coordinates).for_each(|coord| {
            if visited.iter().any(|c| *c == coord) {
                return;
            }

            visited.push(coord);
            let tile = board.tile_map.get_tile_at_coords(coord);
            if tile.is_none() {
                return;
            }
            let tile = tile.unwrap();

            if tile.is_empty() || tile.is_neighbour() {
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

                // recursively reveal neighbors if the tile is empty
                if tile.is_empty() {
                    self.reveal_empty_neighbors(coord, tiles, covered, board, commands, visited);
                }
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
