use crate::texture::ramp::TextureRampSettings;
use crate::texture::{TextureNode, TextureNodeImage};
use crate::ui::grid::InfiniteGridSettings;
use crate::ui::UiState;
use bevy::prelude::*;
use bevy::utils::{HashMap, HashSet};
use bevy_egui::{egui, EguiContexts};
use egui_graph::node::{EdgeEvent, SocketKind};
use petgraph::prelude::EdgeRef;
use petgraph::stable_graph::{DefaultIx, EdgeIndex, IndexType, NodeIndex};
use petgraph::Directed;

pub struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GraphState::new())
            .add_systems(Startup, startup)
            .add_systems(Update, (ui, texture_ui, update_graph));
    }
}

#[derive(Component, Deref, DerefMut, Copy, Clone, PartialEq, Eq, Hash, Debug, Ord, PartialOrd)]
pub struct GraphId(NodeIndex<DefaultIx>);

#[derive(Component)]
pub struct GraphNode {}

impl GraphNode {}

type Graph = petgraph::stable_graph::StableGraph<GraphNode, (usize, usize)>;

#[derive(Component)]
pub struct SelectedNode;

#[derive(Resource)]
pub struct GraphState {
    graph: Graph,
    entity_map: HashMap<GraphId, Entity>,
}

#[derive(Default)]
struct Interaction {
    selection: Selection,
    edge_in_progress: Option<(NodeIndex, SocketKind, usize)>,
}

#[derive(Default)]
struct Selection {
    nodes: HashSet<NodeIndex>,
    edges: HashSet<EdgeIndex>,
}

impl GraphState {
    fn new() -> Self {
        Self {
            graph: Graph::with_capacity(0, 0),
            entity_map: Default::default(),
        }
    }
}

fn startup(mut state: ResMut<GraphState>) {}

fn update_graph(
    mut state: ResMut<GraphState>,
    mut added: Query<(Entity, &GraphId), Added<GraphId>>,
) {
    for (entity, graph_id) in added.iter_mut() {
        state.entity_map.insert(*graph_id, entity);
    }
}

pub fn ui(
    mut commands: Commands,
    mut state: ResMut<GraphState>,
    mut ui_state: ResMut<UiState>,
    mut parent: Query<(Entity, &InheritedVisibility), With<InfiniteGridSettings>>,
    entities: Query<(&TextureNodeImage), Added<GraphId>>,
) {
    for (image) in entities.iter() {
        let (grid, _) = parent.single_mut();
        commands.entity(grid).with_children(|parent| {
            parent.spawn(SpriteBundle {
                texture: (**image).clone(),
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                ..default()
            });
        });
    }
}

fn texture_ui(
    mut commands: Commands,
    mut graph: ResMut<GraphState>,
    mut textures: Query<(Entity, &TextureNode), Without<GraphId>>,
) {
    for (entity, _node) in textures.iter_mut() {
        let node_id = graph.graph.add_node(GraphNode {});
        commands.entity(entity).insert(GraphId(node_id));
    }
}
