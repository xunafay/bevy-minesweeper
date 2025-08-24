use bevy::ecs::component::Component;
use colored::Colorize;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Component)]
pub enum Tile {
    Empty,
    Bomb,
    Neighbour(u8),
}

impl Tile {
    pub fn is_bomb(&self) -> bool {
        matches!(self, Tile::Bomb)
    }

    pub fn console_draw(&self) -> String {
        match self {
            Tile::Empty => " ".to_string(),
            Tile::Bomb => "*".to_string(),
            Tile::Neighbour(n) => format!(
                "{}",
                match *n {
                    1 => "1".cyan(),
                    2 => "2".green(),
                    3 => "3".yellow(),
                    _ => n.to_string().red(),
                }
            ),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        matches!(self, Tile::Empty)
    }
}
