use crate::board::tile::{tile_state::TileState, tile_type::TileType};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Tile {
    pub r#type: TileType,
    pub state: TileState,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            r#type: TileType::Empty,
            state: TileState::Hidden,
        }
    }
}

impl Tile {
    pub fn reveal(&mut self) {
        match (self.r#type, self.state) {
            (TileType::Bomb, TileState::Hidden) => {
                self.state = TileState::Exploded;
            }
            (_, TileState::Hidden) => {
                self.state = TileState::Revealed;
            }
            _ => {}
        }
    }

    pub fn reveal_without_exploding(&mut self) {
        if self.state == TileState::Hidden {
            self.state = TileState::Revealed;
        }
    }

    pub fn toggle_flag(&mut self) {
        match self.state {
            TileState::Hidden => {
                self.state = TileState::Flagged;
            }
            TileState::Flagged => {
                self.state = TileState::Hidden;
            }
            _ => {}
        }
    }
}
