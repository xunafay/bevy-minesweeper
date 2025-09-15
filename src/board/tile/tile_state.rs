#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TileState {
    #[default]
    Hidden,
    Revealed,
    Flagged,
    Exploded,
}
