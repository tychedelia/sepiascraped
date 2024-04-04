use crate::engine::graph::event::{Connect, Disconnect};
use crate::engine::op::OpRef;
use crate::render_layers::{
    Added, Component, Deref, DerefMut, Entity, Parent, Query, ResMut, Resource, Transform, Vec2,
};
use crate::ui::graph;
use crate::ui::graph::{ConnectedTo, Layout};
use crate::{OpName, Sets};
use bevy::prelude::*;
use bevy::utils::HashMap;
use petgraph::adj::DefaultIx;
use petgraph::graph::NodeIndex;

pub mod event;

pub struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GraphState>()
            .add_event::<Connect>()
            .add_event::<Disconnect>()
            .add_systems(
                Update,
                (
                    add_graph_ids,
                    update_graph,
                    handle_connect.run_if(on_event::<Connect>()),
                    handle_disconnect.run_if(on_event::<Disconnect>()),
                )
                    .chain()
                    .in_set(Sets::Ui), //TODO: set ordering, should be graph
            );
    }
}

#[derive(Component, Deref, DerefMut, Copy, Clone, PartialEq, Eq, Hash, Debug, Ord, PartialOrd)]
pub struct GraphId(NodeIndex<DefaultIx>);

#[derive(Component, Debug)]
pub struct GraphNode;

type Graph = petgraph::stable_graph::StableGraph<GraphNode, (usize, usize)>;

#[derive(Resource, Default)]
pub struct GraphState {
    pub graph: Graph,
    pub entity_map: HashMap<NodeIndex, Entity>,
    pub layout: Layout,
}

pub fn update_graph(
    mut state: ResMut<GraphState>,
    mut connected_q: Query<&Parent, Added<ConnectedTo>>,
    mut added_q: Query<(Entity, &GraphId), Added<GraphId>>,
    mut all_nodes_q: Query<(&OpRef, &mut Transform)>,
    graph_id_q: Query<&GraphId>,
) {
    for (entity, graph_id) in added_q.iter_mut() {
        state.entity_map.insert(graph_id.0, entity);
    }

    for parent in connected_q.iter_mut() {
        state.layout = graph::layout(
            state.graph.node_indices().map(|index| (index, Vec2::ZERO)),
            state.graph.edge_indices().map(|index| {
                let (a, b) = state.graph.edge_endpoints(index).unwrap();
                (a, b)
            }),
        );

        for (op_ref, mut transform) in all_nodes_q.iter_mut() {
            let graph_id = graph_id_q.get(**op_ref).unwrap();
            if let Some(pos) = state.layout.get(&graph_id.0) {
                // transform.translation.x = pos.x;
                // transform.translation.y = pos.y;
            }
        }
    }
}

pub fn add_graph_ids(
    mut commands: Commands,
    mut graph: ResMut<GraphState>,
    mut textures: Query<(Entity), (With<OpName>, Without<GraphId>)>,
) {
    for entity in textures.iter_mut() {
        let node_id = graph.graph.add_node(GraphNode {});
        commands.entity(entity).insert(GraphId(node_id));
    }
}

pub fn handle_connect(
    mut graph_state: ResMut<GraphState>,
    graph_id_q: Query<&GraphId>,
    mut ev_connect: EventReader<Connect>,
) {
    for connect in ev_connect.read() {
        let output = graph_id_q.get(connect.output).unwrap();
        let input = graph_id_q.get(connect.input).unwrap();
        graph_state.graph.add_edge(**output, **input, (0, 0));
    }
}

pub fn handle_disconnect(
    mut graph_state: ResMut<GraphState>,
    graph_id_q: Query<&GraphId>,
    mut ev_disconnect: EventReader<Disconnect>,
) {
    for disconnect in ev_disconnect.read() {
        let output = graph_id_q.get(disconnect.output).unwrap();
        let input = graph_id_q.get(disconnect.input).unwrap();
        let edge = graph_state.graph.find_edge(**output, **input).unwrap();
        graph_state.graph.remove_edge(edge);
    }
}
