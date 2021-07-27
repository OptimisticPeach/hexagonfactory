use bevy::ecs::entity::Entity;
use bevy::ecs::system::{ResMut, Commands};

pub struct LoadState(pub Option<Entity>);

// pub fn loader<const PER_ITER: usize>(
//     mut current_target: ResMut<LoadState>,
//     mut commands: Commands,
//     mut current_state:
// )
