use bevy::prelude::*;

use crate::op::texture::TextureOp;
use crate::ui::graph::{update_graph, GraphState};
use crate::Sets::{Graph, Params};

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app;
        // .add_systems(Update, order_cameras.in_set(Params));
    }
}

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
