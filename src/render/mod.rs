use std::hash::{BuildHasherDefault, Hash, Hasher};
use bevy::prelude::*;
use bevy::render::camera::CameraOutputMode;
use bevy::utils::AHasher;

use crate::op::texture::TextureOp;
use crate::{OpName, Sets};
use crate::param::{Params};
use crate::ui::graph::{update_graph, GraphState};

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (order_cameras).in_set(Sets::Params));
    }
}

// fn params_hash(mut params_hash: ResMut<ParamsHash>, mut query: Query<(Entity, &mut Camera), With<OpName>>, params: Params) {
//     for (entity, mut camera) in query.iter_mut() {
//         let mut params = params.get_all(entity.clone());
//         let prev_hash = params_hash.hashes.get(&entity);
//         let mut hasher = AHasher::default();
//         params.hash(&mut hasher);
//         let new_hash = hasher.finish();
//         if let None = prev_hash {
//             params_hash.hashes.insert(entity, new_hash);
//             camera.output_mode = CameraOutputMode::default();
//         } else if let Some(prev_hash) = prev_hash {
//             if *prev_hash != new_hash {
//                 params_hash.hashes.insert(entity, new_hash);
//                 camera.output_mode = CameraOutputMode::default();
//             } else {
//                 camera.output_mode = CameraOutputMode::Skip;
//             }
//         }
//     }
// }

fn order_cameras(graph: Res<GraphState>, mut cameras: Query<&mut Camera>) {
    let sorted = petgraph::algo::toposort(&graph.graph, None);
    match sorted {
        Ok(sorted) => {
            for (i, index) in sorted.iter().enumerate() {
                let entity = graph
                    .entity_map
                    .get(index)
                    .expect(format!("Failed to get entity for index {:?}", index).as_str());
                if let Ok(mut camera) = cameras.get_mut(*entity) {
                    camera.order = i as isize;
                }
            }
        }
        Err(e) => {
            error!("Failed to sort cameras: {:?}", e)
        }
    }
}
