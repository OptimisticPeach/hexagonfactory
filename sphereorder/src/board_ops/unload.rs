use smallvec::SmallVec;
use bevy::ecs::entity::Entity;

pub struct UnloadState(pub SmallVec<[Entity; 2]>);
