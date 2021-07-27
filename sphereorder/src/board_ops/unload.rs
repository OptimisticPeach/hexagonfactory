use crate::board_ops::OpState;
use smallvec::SmallVec;
use bevy::ecs::entity::Entity;

pub struct UnloadState(SmallVec<[Entity; 2]>);
