use bevy::prelude::*;

use std::ops::{Deref, DerefMut};

use rand::{Rng, rng};

use crate::board::{
    coordinates::Coordinates,
    tile::{tile::Tile, tile_state::TileState, tile_type::TileType},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TileMap {
    pub bomb_count: u16,
    pub width: u16,
    pub height: u16,
    pub map: Vec<Vec<Tile>>,
}

impl TileMap {
    pub fn empty(width: u16, height: u16) -> Self {
        let map: Vec<Vec<Tile>> = (0..height)
            .map(|_| (0..width).map(|_| Tile::default()).collect::<Vec<Tile>>())
            .collect();

        Self {
            bomb_count: 0,
            width,
            height,
            map,
        }
    }

    pub fn console_output(&self) -> String {
        let mut buffer = format!(
            "Map ({}, {}) with {} bombs:\n",
            self.width, self.height, self.bomb_count
        );

        let line: String = (0..=(self.width + 1)).into_iter().map(|_| '-').collect();
        buffer = format!("{}{}\n", buffer, line);

        for line in self.iter().rev() {
            buffer = format!("{}|", buffer);
            for tile in line.iter() {
                buffer = format!("{}{}", buffer, tile.r#type.console_draw());
            }
            buffer = format!("{}|\n", buffer);
        }
        format!("{}{}", buffer, line)
    }

    pub fn scan_map_at(&self, coordinates: Coordinates) -> impl Iterator<Item = Coordinates> {
        SQUARE_COORDINATES
            .iter()
            .copied()
            .map(move |tuple| coordinates + tuple)
    }

    pub fn is_bomb_at(&self, coordinates: Coordinates) -> bool {
        if coordinates.x >= self.width || coordinates.y >= self.height {
            return false;
        }

        self.map[coordinates.y as usize][coordinates.x as usize]
            .r#type
            .is_bomb()
    }

    pub fn bomb_count_at(&self, coordinates: Coordinates) -> u8 {
        if coordinates.x >= self.width || coordinates.y >= self.height {
            return 0;
        }

        if self.is_bomb_at(coordinates) {
            return 0;
        }

        self.scan_map_at(coordinates)
            .filter(|&coord| self.is_bomb_at(coord))
            .count() as u8
    }

    pub fn set_bombs(&mut self, bomb_count: u16) {
        self.bomb_count = bomb_count;
        let mut remaining_bombs = bomb_count;
        let mut rng = rng();

        while remaining_bombs > 0 {
            let (x, y) = (
                rng.random_range(0..self.width) as usize,
                rng.random_range(0..self.height) as usize,
            );
            if let TileType::Empty = self[y][x].r#type {
                self[y][x].r#type = TileType::Bomb;
                remaining_bombs -= 1;
            }
        }

        for y in 0..self.height {
            for x in 0..self.width {
                let coords = Coordinates { x, y };
                if self.is_bomb_at(coords) {
                    continue;
                }
                let num = self.bomb_count_at(coords);
                if num == 0 {
                    continue;
                }
                let tile = &mut self[y as usize][x as usize];
                tile.r#type = TileType::Neighbour(num);
            }
        }
    }

    pub(crate) fn at(&self, coord: Coordinates) -> Option<&Tile> {
        if coord.x >= self.width || coord.y >= self.height {
            return None;
        }

        Some(&self.map[coord.y as usize][coord.x as usize])
    }

    pub fn at_mut(&mut self, coord: Coordinates) -> Option<&mut Tile> {
        if coord.x >= self.width || coord.y >= self.height {
            return None;
        }

        Some(&mut self.map[coord.y as usize][coord.x as usize])
    }

    pub fn coords_in_bounds(&self, coordinates: Coordinates) -> bool {
        coordinates.x < self.width && coordinates.y < self.height
    }

    pub fn reveal_all(&mut self) {
        for row in self.map.iter_mut() {
            for tile in row.iter_mut() {
                tile.reveal();
            }
        }
    }

    pub fn reveal_empty_neighbors(
        &self,
        coordinates: Coordinates,
        revealed: &mut Vec<Coordinates>,
    ) {
        self.scan_map_at(coordinates).for_each(|coord| {
            if !self.coords_in_bounds(coord) {
                return;
            }

            if let Some(tile) = self.at(coord) {
                if tile.r#type.is_empty() && !revealed.contains(&coord) {
                    revealed.push(coord);
                    self.reveal_empty_neighbors(coord, revealed);
                } else if tile.r#type.is_neighbour() && !revealed.contains(&coord) {
                    revealed.push(coord);
                }
            }
        });
    }

    /// Reveals the neighbors of a given coordinate, returning a list of all revealed unflagged
    /// bombs
    pub fn reveal_neighbors(&self, coordinates: Coordinates) -> Vec<Coordinates> {
        let mut revealed = Vec::new();
        self.scan_map_at(coordinates).for_each(|coord| {
            if !self.coords_in_bounds(coord) {
                return;
            }

            if let Some(tile) = self.at(coord) {
                if tile.r#type.is_bomb() && !revealed.contains(&coord) {
                    revealed.push(coord);
                } else if tile.r#type.is_empty() && !revealed.contains(&coord) {
                    revealed.push(coord);
                    self.reveal_empty_neighbors(coord, &mut revealed);
                } else if tile.r#type.is_neighbour() && !revealed.contains(&coord) {
                    revealed.push(coord);
                }
            }
        });

        revealed
    }

    pub fn has_won(&self) -> bool {
        let mut hidden_tiles = 0;
        let mut flagged_bombs = 0;

        for row in self.map.iter() {
            for tile in row.iter() {
                if tile.state == TileState::Hidden {
                    hidden_tiles += 1;
                    if tile.r#type.is_bomb() {
                        flagged_bombs += 1;
                    }
                }
            }
        }

        hidden_tiles == self.bomb_count && flagged_bombs == self.bomb_count
    }

    pub fn has_lost(&self) -> bool {
        for row in self.map.iter() {
            for tile in row.iter() {
                if tile.state == TileState::Exploded {
                    return true;
                }
            }
        }

        false
    }
}

impl Deref for TileMap {
    type Target = Vec<Vec<Tile>>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

impl DerefMut for TileMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}

/// Delta coordinates for all 8 square neighbors
pub const SQUARE_COORDINATES: [(i8, i8); 8] = [
    // Bottom left
    (-1, -1),
    // Bottom
    (0, -1),
    // Bottom right
    (1, -1),
    // Left
    (-1, 0),
    // Right
    (1, 0),
    // Top Left
    (-1, 1),
    // Top
    (0, 1),
    // Top right
    (1, 1),
];
