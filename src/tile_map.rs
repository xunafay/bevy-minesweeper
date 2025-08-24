use std::ops::{Deref, DerefMut};

use rand::{Rng, rng};

use crate::{coordinates::Coordinates, tile::Tile};

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
            .map(|_| (0..width).map(|_| Tile::Empty).collect::<Vec<Tile>>())
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
                buffer = format!("{}{}", buffer, tile.console_draw());
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

        self.map[coordinates.y as usize][coordinates.x as usize].is_bomb()
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
            if let Tile::Empty = self[y][x] {
                self[y][x] = Tile::Bomb;
                remaining_bombs -= 1;
            }
        }

        // Update neighbors with bomb counts
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
                *tile = Tile::Neighbour(num);
            }
        }
    }

    pub(crate) fn get_tile_at_coords(&self, coord: Coordinates) -> Option<Tile> {
        if coord.x >= self.width || coord.y >= self.height {
            return None;
        }

        Some(self.map[coord.y as usize][coord.x as usize])
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
