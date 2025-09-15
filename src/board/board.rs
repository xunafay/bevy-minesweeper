use bevy::{ecs::system::QueryLens, prelude::*};

use crate::{
    UiSettings,
    board::{coordinates::Coordinates, tile_map::TileMap},
    utils::bounds2::Bounds2,
};

#[derive(Component)]
pub struct Board {
    pub tile_map: TileMap,
}

impl Board {
    pub fn find_colliding_tile_coords(
        &self,
        point: Vec2,
        tiles: &mut QueryLens<(Entity, &GlobalTransform, &Coordinates)>,
        ui_settings: &UiSettings,
    ) -> Option<(Entity, Coordinates)> {
        for (entity, transform, coords) in &mut tiles.query() {
            let bounds = Bounds2::from_center_size(
                transform.translation().truncate(),
                Vec2::splat(ui_settings.tile_size + ui_settings.tile_spacing),
            );

            if bounds.contains(point) {
                return Some((entity, coords.clone()));
            }
        }

        None
    }
}
