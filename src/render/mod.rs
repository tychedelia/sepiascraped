use bevy::prelude::*;

use crate::texture::TextureOp;
use crate::ui::graph::GraphState;

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, order_cameras.after(crate::ui::graph::update_graph));
    }
}

fn order_cameras(mut graph: Res<GraphState>, mut cameras: Query<(&mut Camera), With<TextureOp>>) {
    let sorted = petgraph::algo::toposort(&graph.graph, None);
    match sorted {
        Ok(sorted) => {
            for (i, index) in sorted.iter().enumerate() {
                let entity = graph.entity_map.get(index).expect("entity not found");
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
