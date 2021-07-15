use bevy::ecs::system::{Query, ResMut, IntoSystem};
use bevy::asset::{Assets, Handle};
use crate::{BoardMember, FaceMaterialIdx, OldFaceMaterialIdx};
use bevy::transform::components::Parent;
use bevy::render::mesh::{Mesh, VertexAttributeValues};
use bevy::app::{AppBuilder, Plugin};
use bevy::ecs::query::Changed;

pub fn update_material_idx_system(
    mut query: Query<(&BoardMember, &FaceMaterialIdx, &mut OldFaceMaterialIdx, &Parent), Changed<FaceMaterialIdx>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mesh_handles: Query<&Handle<Mesh>>,
) {
    query
        .iter_mut()
        .for_each(|(member_data, new_face, mut old_face, parent)| {
            let mesh_handle = mesh_handles.get(parent.0).unwrap().clone();

            meshes
                .get_mut(mesh_handle)
                .unwrap()
                .attribute_mut(shaders::ATTRIBUTE_PER_FACE_INDEX)
                .map(|attribs| {
                    match attribs {
                        VertexAttributeValues::Sint32(v) => {
                            let idx = member_data.face_idx;
                            old_face.0 = v[idx];
                            v[idx] = new_face.0;
                        }
                        _ => panic!(),
                    }
                })
                .unwrap();
        });
}
