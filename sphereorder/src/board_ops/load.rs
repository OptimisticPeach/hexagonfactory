use bevy::ecs::entity::Entity;
use bevy::ecs::system::{ResMut, Commands, Query};
use crate::{RelationPair, NeighbourOf, PackedRelations};

pub struct LoadState(
    pub Option<Entity>,
    pub Option<std::collections::hash_map::IntoIter<RelationPair, NeighbourOf>>
);

pub fn loader<const PER_ITER: usize>(
    mut state: ResMut<LoadState>,
    mut commands: Commands,
    query: Query<&PackedRelations>,
) {
    let planet_shell = if let Some(entity) = state.0 {
        entity
    } else {
        return;
    };

    let mut iter = state
        .1
        .get_or_insert_with(|| {
            query
                .get(planet_shell)
                .unwrap()
                .relations
                .clone()
                .into_iter()
        });

    // &mut I is also an iterator, and does not consume the I.
    let executions = (&mut iter)
        .take(PER_ITER)
        .map(|(RelationPair(a, b), data)| {
            commands
                .entity(a)
                .insert_relation(data.clone(), b);

            commands
                .entity(b)
                .insert_relation(data, a);
        })
        .count();

    // If we didn't get all of the iterations we should've, we're done.
    if executions != PER_ITER {
        state.0 = None;
        state.1 = None;
    }
}

pub fn load_all(
    mut state: ResMut<LoadState>,
    mut commands: Commands,
    query: Query<&PackedRelations>,
) {
    let planet_shell = if let Some(entity) = state.0 {
        entity
    } else {
        return;
    };

    let mut iter = state
        .1
        .take()
        .unwrap_or_else(|| {
            query
                .get(planet_shell)
                .unwrap()
                .relations
                .clone()
                .into_iter()
        });

    // &mut I is also an iterator, and does not consume the I.
    iter
        .for_each(|(RelationPair(a, b), data)| {
            commands
                .entity(a)
                .insert_relation(data.clone(), b);

            commands
                .entity(b)
                .insert_relation(data, a);
        });
}
