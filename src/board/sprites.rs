use bevy::prelude::*;

#[derive(Resource)]
pub struct Sprites {
    pub explosion: Handle<Image>,
    pub flag: Handle<Image>,
    pub cover: Handle<Image>,
    pub uncovered: Handle<Image>,
    pub bomb: Handle<Image>,
    pub font: Handle<Font>,
}
