use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::mesh::{Indices, PrimitiveTopology},
};

use crate::{
    game::player::{HoverHighlight, TargetBlock},
    model::Model,
    screens::Screen,
};

pub struct BlockHighlightPlugin;

impl Plugin for BlockHighlightPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_highlight_mesh.run_if(in_state(Screen::Gameplay)),
        );
    }
}

fn update_highlight_mesh(
    models: Res<Assets<Model>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query_highlight: Query<
        (&mut Mesh3d, &TargetBlock),
        (With<HoverHighlight>, Changed<TargetBlock>),
    >,
) {
    let Ok((mut hover_mesh, hover_target)) = query_highlight.get_single_mut() else {
        return;
    };

    if let Some(handle) = &hover_target.model_handle {
        let model = models.get(handle.id()).unwrap();

        let mut vs = vec![];
        let mut is = vec![];

        for i in 0..model.faces.len() {
            let face = &model.faces[i];
            let offset = vs.len() as u32;
            for j in 0..face.vertices.len() {
                let v = &face.vertices[j];
                vs.push(v.position);
                is.push(offset + j as u32);
                is.push(offset + ((j + 1) % face.vertices.len()) as u32);
            }
        }

        let new_mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::default())
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vs)
            .with_inserted_indices(Indices::U32(is));
        let new_mesh_handle = meshes.add(new_mesh);
        hover_mesh.0 = new_mesh_handle;
    }
}
