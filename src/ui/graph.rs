use crate::texture::ramp::TextureRampSettings;
use crate::texture::TextureNodeImage;
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
            .add_systems(Update, (ui, texture_ui, update_graph))
            .add_systems(PostUpdate, (post_update_ui));
    }
}

#[derive(Component, Deref, DerefMut, Copy, Clone, PartialEq, Eq, Hash, Debug, Ord, PartialOrd)]
pub struct GraphId(NodeIndex<DefaultIx>);

#[derive(Component)]
pub struct GraphNode {
    pub ui: Box<dyn FnMut(&mut egui::Ui) + Sync + Send>,
}

impl GraphNode {
    pub fn new<F: 'static + FnMut(&mut egui::Ui) + Sync + Send>(ui: F) -> Self {
        Self { ui: Box::new(ui) }
    }
}

type Graph = petgraph::stable_graph::StableGraph<GraphNode, (usize, usize)>;

#[derive(Component)]
pub struct SelectedNode;

#[derive(Resource)]
pub struct GraphState {
    graph: Graph,
    entity_map: HashMap<GraphId, Entity>,
    view: egui_graph::View,
    interaction: Interaction,
    flow: egui::Direction,
    socket_radius: f32,
    socket_color: egui::Color32,
    wire_width: f32,
    wire_color: egui::Color32,
    auto_layout: bool,
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
            view: Default::default(),
            interaction: Default::default(),
            flow: egui::Direction::LeftToRight,
            socket_radius: 5.0,
            socket_color: egui::Color32::from_rgba_premultiplied(100, 100, 100, 255),
            wire_width: 2.0,
            wire_color: egui::Color32::from_rgba_premultiplied(100, 100, 100, 255),
            auto_layout: true,
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

fn post_update_ui(
    mut commands: Commands,
    selected_nodes: Query<(Entity, &SelectedNode)>
) {
    for (entity, _) in selected_nodes.iter() {
        // TODO: ordering prboblems with the ui_state
        // commands.entity(entity).remove::<SelectedNode>();
    }
}

pub fn ui(
    mut commands: Commands,
    mut egui_contexts: EguiContexts,
    mut state: ResMut<GraphState>,
    mut ui_state: ResMut<UiState>,
) {
    let ctx = if let Some(ref mut response) = &mut ui_state.side_panel {
        &mut response.ctx
    } else {
        egui_contexts.ctx_mut()
    };

    if state.graph.node_count() > 0 {
        state.view.layout = layout(&state.graph, state.flow, ctx);
    }

    egui::containers::CentralPanel::default()
        .frame(egui::Frame::default())
        .show(ctx, |ui| {
            egui_graph::Graph::new("Node Graph")
                .show(&mut Default::default(), ui)
                .nodes(|nctx, ui| nodes(nctx, ui, &mut state))
                .edges(|ectx, ui| edges(ectx, ui, &mut state));

            for node in &state.interaction.selection.nodes {
                let entity = state.entity_map[&GraphId(*node)];
                commands.entity(entity).insert(SelectedNode);
            }
        });
}

fn texture_ui(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut graph: ResMut<GraphState>,
    mut textures: Query<(Entity, &TextureNodeImage, &mut TextureRampSettings), Without<GraphId>>,
) {
    for (entity, texture, mut settings) in textures.iter_mut() {
        let preview_texture_id = contexts.image_id(&texture).unwrap();
        let node_id = graph.graph.add_node(GraphNode::new(move |ui| {
            ui.image(egui::load::SizedTexture::new(
                preview_texture_id,
                egui::vec2(200.0, 200.0),
            ));
        }));

        commands.entity(entity).insert(GraphId(node_id));
    }
}

fn layout(graph: &Graph, flow: egui::Direction, ctx: &egui::Context) -> egui_graph::Layout {
    ctx.memory(|m| {
        let nodes = graph.node_indices().map(|n| {
            let id = egui::Id::new(n);
            let size = m
                .area_rect(id)
                .map(|a| a.size())
                .unwrap_or([200.0, 50.0].into());
            (id, size)
        });
        let edges = graph
            .edge_indices()
            .filter_map(|e| graph.edge_endpoints(e))
            .map(|(a, b)| (egui::Id::new(a), egui::Id::new(b)));
        egui_graph::layout(nodes, edges, flow)
    })
}

fn nodes(nctx: &mut egui_graph::NodesCtx, ui: &mut egui::Ui, state: &mut GraphState) {
    let indices: Vec<_> = state.graph.node_indices().collect();
    for n in indices {
        let inputs = 1;
        let outputs = 1;
        let node = &mut state.graph[n];
        let graph_view = &mut state.view;
        let response = egui_graph::node::Node::new(n)
            .inputs(inputs)
            .outputs(outputs)
            .flow(state.flow)
            .socket_radius(state.socket_radius)
            .socket_color(state.socket_color)
            .show(graph_view, nctx, ui, |ui| {
                (node.ui)(ui);
            });

        if response.changed() {
            // Keep track of the selected nodes.
            if let Some(selected) = response.selection() {
                if selected {
                    assert!(state.interaction.selection.nodes.insert(n));
                } else {
                    assert!(state.interaction.selection.nodes.remove(&n));
                }
            }

            // Check for an edge event.
            if let Some(ev) = response.edge_event() {
                dbg!(&ev);
                match ev {
                    EdgeEvent::Started { kind, index } => {
                        state.interaction.edge_in_progress = Some((n, kind, index));
                    }
                    EdgeEvent::Ended { kind, index } => {
                        // Create the edge.
                        if let Some((src, _, ix)) = state.interaction.edge_in_progress.take() {
                            let (a, b, w) = match kind {
                                SocketKind::Input => (src, n, (ix, index)),
                                SocketKind::Output => (n, src, (index, ix)),
                            };
                            // Check that this edge doesn't already exist.
                            if !state
                                .graph
                                .edges(a)
                                .any(|e| e.target() == b && *e.weight() == w)
                            {
                                state.graph.add_edge(a, b, w);
                            }
                        }
                    }
                    EdgeEvent::Cancelled => {
                        state.interaction.edge_in_progress = None;
                    }
                }
            }

            // If the delete key was pressed while selected, remove it.
            if response.removed() {
                state.graph.remove_node(n);
            }
        }
    }
}

fn edges(ectx: &mut egui_graph::EdgesCtx, ui: &mut egui::Ui, state: &mut GraphState) {
    // Draw the attached edges.
    let indices: Vec<_> = state.graph.edge_indices().collect();
    let stroke = egui::Stroke {
        width: state.wire_width,
        color: state.wire_color,
    };

    let mouse_pos = ui.input(|i| i.pointer.interact_pos().unwrap_or_default());
    let click = ui.input(|i| i.pointer.any_released());
    let shift_held = ui.input(|i| i.modifiers.shift);
    let mut clicked_on_edge = false;
    let selection_threshold = state.wire_width * 8.0; // Threshold for selecting the edge

    for e in indices {
        let (na, nb) = state.graph.edge_endpoints(e).unwrap();
        let (output, input) = *state.graph.edge_weight(e).unwrap();
        let a = egui::Id::new(na);
        let b = egui::Id::new(nb);
        let a_out = ectx.output(ui, a, output).unwrap();
        let b_in = ectx.input(ui, b, input).unwrap();
        let bezier = egui_graph::bezier::Cubic::from_edge_points(a_out, b_in);
        let dist_per_pt = 5.0;
        let pts: Vec<_> = bezier.flatten(dist_per_pt).collect();

        // Check if mouse is over the bezier curve
        let closest_point = bezier.closest_point(dist_per_pt, egui::Pos2::from(mouse_pos));
        let distance_to_mouse = closest_point.distance(egui::Pos2::from(mouse_pos));
        if distance_to_mouse < selection_threshold && click {
            clicked_on_edge = true;
            // If Shift is not held, clear previous selection
            if !shift_held {
                state.interaction.selection.edges.clear();
            }
            // Add the clicked edge to the selection
            state.interaction.selection.edges.insert(e);
        }

        let wire_stroke = if state.interaction.selection.edges.contains(&e) {
            egui::Stroke {
                width: state.wire_width * 4.0,
                color: state.wire_color.linear_multiply(1.5),
            }
        } else {
            stroke
        };

        // Draw the bezier curve
        ui.painter()
            .add(egui::Shape::line(pts.clone(), wire_stroke));
    }

    if click && !clicked_on_edge {
        // Click occurred on the canvas, clear the selection
        state.interaction.selection.edges.clear();
    }

    // Draw the in-progress edge if there is one.
    if let Some(edge) = ectx.in_progress(ui) {
        let bezier = edge.bezier_cubic();
        let dist_per_pt = 5.0;
        let pts = bezier.flatten(dist_per_pt).collect();
        ui.painter().add(egui::Shape::line(pts, stroke));
    }

    // Remove selected edges if delete/backspace is pressed
    if ui.input(|i| i.key_pressed(egui::Key::Delete) | i.key_pressed(egui::Key::Backspace)) {
        state.interaction.selection.edges.iter().for_each(|e| {
            state.graph.remove_edge(*e);
        });
        state.interaction.selection.nodes.clear();
    }
}
