use bevy::ecs::{entity::Entity, resource::Resource};

#[derive(Resource)]
pub struct MenuData {
    pub button_entity: Entity,
}
