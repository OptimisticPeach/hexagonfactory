use crate::{FaceMaterialIdx, OldFaceMaterialIdx, TileDataIdx};
use bevy::asset::{Assets, Handle};
use bevy::ecs::query::Changed;
use bevy::ecs::system::{Query, ResMut};
use bevy::render::mesh::{Mesh, VertexAttributeValues};
use bevy::transform::components::Parent;
use bevy::transform::prelude::Children;
use bevy::utils::HashSet;

pub(crate) fn update_material_idx_system(
    children_query: Query<(&Handle<Mesh>, &Children)>,
    first_query: Query<&Parent, Changed<FaceMaterialIdx>>,
    mut query: Query<
        (&TileDataIdx, &FaceMaterialIdx, &mut OldFaceMaterialIdx),
        Changed<FaceMaterialIdx>,
    >,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    first_query
        .iter()
        .map(|x| x.0)
        .collect::<HashSet<_>>()
        .into_iter()
        .for_each(move |planet_with_changed_tile| {
            let (mesh_handle, children) = children_query.get(planet_with_changed_tile).unwrap();

            meshes
                .get_mut(mesh_handle)
                .unwrap()
                .attribute_mut(shaders::ATTRIBUTE_PER_FACE_INDEX)
                .map(|attribs| match attribs {
                    VertexAttributeValues::Sint32(v) => {
                        children.iter().for_each(|child| {
                            let child = query.get_mut(*child);
                            let (member_data, new_face, mut old_face) = if let Ok(x) = child {
                                x
                            } else {
                                return;
                            };

                            let idx = member_data.0;
                            old_face.0 = v[idx];
                            v[idx] = new_face.0;
                        });
                    }
                    _ => panic!(),
                })
                .unwrap();
        });
}
