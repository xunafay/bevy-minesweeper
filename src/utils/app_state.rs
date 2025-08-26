use bevy::state::state::States;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, States)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
    Victory,
    Defeat,
}
