use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle};
use bevy::utils::{HashMap, HashSet};
use bevy_mod_picking::prelude::*;
use bevy_mod_picking::PickableBundle;
use egui_graph::node::SocketKind;
use petgraph::stable_graph::{DefaultIx, EdgeIndex, IndexType, NodeIndex};

use crate::texture::{TextureNode, TextureNodeImage};
use crate::ui::event::ClickNode;
use crate::ui::grid::InfiniteGridSettings;
use crate::ui::UiState;

pub struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GraphState::new())
            .add_plugins(Material2dPlugin::<NodeMaterial>::default())
            .add_systems(Startup, startup)
            .add_systems(
                Update,
                (
                    ui,
                    texture_ui,
                    update_graph,
                    click_node.run_if(on_event::<ClickNode>()),
                ),
            );
    }
}

#[derive(Component, Deref, DerefMut, Copy, Clone, PartialEq, Eq, Hash, Debug, Ord, PartialOrd)]
pub struct GraphId(NodeIndex<DefaultIx>);

#[derive(Component, Deref, DerefMut, Copy, Clone, PartialEq, Eq, Hash, Debug, Ord, PartialOrd)]
pub struct GraphRef(Entity);

#[derive(Component)]
pub struct GraphNode {}

impl GraphNode {}

type Graph = petgraph::stable_graph::StableGraph<GraphNode, (usize, usize)>;

#[derive(Component, Clone)]
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

fn click_node(
    mut commands: Commands,
    mut click_events: EventReader<ClickNode>,
    mut prev_selected: Query<(Entity), With<SelectedNode>>,
    mut graph_ref: Query<(&GraphRef, &Handle<NodeMaterial>)>,
    mut materials: ResMut<Assets<NodeMaterial>>,
) {
    for event in click_events.read() {
        for (entity) in prev_selected.iter_mut() {
            commands.entity(entity).remove::<SelectedNode>();
        }
        let q = graph_ref.get(**event).unwrap();
        let entity = **q.0;
        commands.entity(entity).insert(SelectedNode);
        let material = materials.get_mut(q.1).unwrap();
        material.selected = 1;
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<NodeMaterial>>,
    mut parent: Query<(Entity, &InheritedVisibility), With<InfiniteGridSettings>>,
    entities: Query<(Entity, &TextureNodeImage), Added<GraphId>>,
) {
    for (entity, image) in entities.iter() {
        let (grid, _) = parent.single_mut();
        commands.entity(grid).with_children(|parent| {
            parent
                .spawn((
                    GraphRef(entity),
                    MaterialMesh2dBundle {
                        mesh: meshes
                            .add(Mesh::from(shape::Quad::new(Vec2::new(100.0, 100.0))))
                            .into(),
                        material: materials.add(NodeMaterial {
                            selected: 0,
                            color_texture: (**image).clone(),
                        }),
                        transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                        ..Default::default()
                    },
                    PickableBundle::default(), // <- Makes the mesh pickable.
                    On::<Pointer<Down>>::send_event::<ClickNode>(), // <- Send SelectedNode event on pointer down
                    On::<Pointer<DragStart>>::target_insert(Pickable::IGNORE), // Disable picking
                    On::<Pointer<DragEnd>>::target_insert(Pickable::default()), // Re-enable picking
                    On::<Pointer<Drag>>::target_component_mut::<Transform>(|drag, transform| {
                        transform.translation.x += drag.delta.x; // Make the square follow the mouse
                        transform.translation.y -= drag.delta.y;
                    }),
                ));
                // .with_children(|parent| {
                //     parent.spawn((
                //         PickableBundle {
                //             pickable: Pickable::IGNORE,
                //             ..default()
                //         }, // <- Makes the mesh pickable.
                //         GraphRef(entity),
                //         SpriteBundle {
                //             sprite: Sprite {
                //                 custom_size: Some(Vec2::new(90.0, 90.0)),
                //                 ..Default::default()
                //             },
                //             texture: (**image).clone(),
                //             transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                //             ..default()
                //         },
                //     ));
                // });
        });
    }
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct NodeMaterial {
    #[uniform(0)]
    selected: u32,
    #[texture(1)]
    #[sampler(2)]
    color_texture: Handle<Image>,
}

// All functions on `Material2d` have default impls. You only need to implement the
// functions that are relevant for your material.
impl Material2d for NodeMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/ui/node_material.wgsl".into()
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
