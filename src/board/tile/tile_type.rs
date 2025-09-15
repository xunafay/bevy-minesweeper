use bevy::ecs::component::Component;
use colored::Colorize;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Component)]
pub enum TileType {
    Empty,
    Bomb,
    Neighbour(u8),
}

impl TileType {
    pub fn is_bomb(&self) -> bool {
        matches!(self, TileType::Bomb)
    }

    pub fn console_draw(&self) -> String {
        match self {
            TileType::Empty => " ".to_string(),
            TileType::Bomb => "*".to_string(),
            TileType::Neighbour(n) => format!(
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
        matches!(self, TileType::Empty)
    }

    pub(crate) fn is_neighbour(&self) -> bool {
        matches!(self, TileType::Neighbour(_))
    }
}
