use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy::render::camera::CameraOutputMode;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle};
use bevy::utils::{info, HashMap};
use bevy_mod_picking::prelude::*;
use bevy_mod_picking::PickableBundle;
use bevy_prototype_lyon::draw::Stroke;
use bevy_prototype_lyon::path::PathBuilder;
use bevy_prototype_lyon::prelude::{GeometryBuilder, ShapeBundle};
use bevy_prototype_lyon::shapes::Line;
use layout::core::base::Orientation;
use layout::core::geometry::Point;
use layout::core::style::StyleAttr;
use layout::std_shapes::shapes::{Arrow, Element, ShapeKind};
use layout::topo::layout::VisualGraph;
use layout::topo::placer::Placer;
use petgraph::stable_graph::{DefaultIx, IndexType, NodeIndex};
use rand::{random, Rng};

use crate::op::texture::TextureOp;
use crate::op::{OpCategory, OpDefaultImage, OpImage, OpInputs, OpOutputs, OpRef};
use crate::param::ParamValue;
use crate::ui::event::{ClickNode, Connect, Disconnect};
use crate::ui::grid::InfiniteGridSettings;
use crate::ui::UiCamera;
use crate::{OpName, Sets};

pub struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GraphState>()
            .add_event::<Connect>()
            .add_event::<Disconnect>()
            .add_plugins(Material2dPlugin::<NodeMaterial>::default())
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    ui.in_set(Sets::Ui),
                    update_camera_enabled.in_set(Sets::Ui),
                    add_graph_ids.in_set(Sets::Ui),
                    draw_refs.in_set(Sets::Ui),
                    update_connections.in_set(Sets::Ui),
                    update_graph.in_set(Sets::Graph),
                    click_node.run_if(on_event::<ClickNode>()),
                    update_graph_refs.in_set(Sets::Graph),
                ),
            )
            .add_systems(First, update_op_images);
    }
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Components
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[derive(Component, Deref, DerefMut, Copy, Clone, PartialEq, Eq, Hash, Debug, Ord, PartialOrd)]
pub struct GraphId(NodeIndex<DefaultIx>);

#[derive(Component, Deref, DerefMut, Copy, Clone, PartialEq, Eq, Hash, Debug, Ord, PartialOrd)]
pub struct GraphRef(pub Entity);

#[derive(Component, Debug)]
pub struct GraphNode;

type Graph = petgraph::stable_graph::StableGraph<GraphNode, (usize, usize)>;

#[derive(Component, Clone)]
pub struct SelectedNode;

#[derive(Component)]
pub struct Port;

#[derive(Component)]
pub struct InPort(u8);

#[derive(Component)]
pub struct OutPort(u8);

#[derive(Component, Clone)]
pub struct Connecting;

#[derive(Component)]
pub struct ConnectedTo(Entity);

#[derive(Component, Debug)]
pub struct NodeRoot;

#[derive(Component, Debug)]
pub struct OpRefConnection;

#[derive(Component, Debug)]
pub struct DisabledNode;

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Resources
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[derive(Resource, Default)]
pub struct GraphState {
    pub graph: Graph,
    pub entity_map: HashMap<NodeIndex, Entity>,
    pub layout: Layout,
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Assets
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct NodeMaterial {
    #[uniform(0)]
    pub selected: u32,
    #[uniform(0)]
    pub category_color: Color,
    #[uniform(0)]
    pub disabled: u32,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Handle<Image>,
}

impl Material2d for NodeMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/ui/node_material.wgsl".into()
    }
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Systems
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

fn click_node(
    mut commands: Commands,
    mut click_events: EventReader<ClickNode>,
    mut prev_selected: Query<Entity, With<SelectedNode>>,
    mut all_mats: Query<&Handle<NodeMaterial>>,
    clicked_q: Query<(&OpRef, &Handle<NodeMaterial>)>,
    mut materials: ResMut<Assets<NodeMaterial>>,
) {
    for event in click_events.read() {
        for mat in all_mats.iter_mut() {
            let mat = materials.get_mut(mat).unwrap();
            mat.selected = 0;
        }

        for entity in prev_selected.iter_mut() {
            commands.entity(entity).remove::<SelectedNode>();
        }

        if let Ok(q) = clicked_q.get(**event) {
            let entity = **q.0;
            commands.entity(entity).insert(SelectedNode);
            let material = materials.get_mut(q.1).unwrap();
            material.selected = 1;
        }
    }
}

fn setup(mut state: ResMut<GraphState>) {}

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
        state.layout = layout(
            state.graph.node_indices().map(|index| (index, Vec2::ZERO)),
            state.graph.edge_indices().map(|index| {
                let (a, b) = state.graph.edge_endpoints(index).unwrap();
                (a, b)
            }),
        );

        for (graph_ref, mut transform) in all_nodes_q.iter_mut() {
            let graph_id = graph_id_q.get(**graph_ref).unwrap();
            if let Some(pos) = state.layout.get(&graph_id.0) {
                // transform.translation.x = pos.x;
                // transform.translation.y = pos.y;
            }
        }
    }
}

pub fn ui(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<NodeMaterial>>,
    images: Res<Assets<Image>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    mut asset_server: ResMut<AssetServer>,
    default_image: Res<OpDefaultImage>,
    mut parent: Query<(Entity, &InheritedVisibility), With<InfiniteGridSettings>>,
    op_q: Query<
        (
            Entity,
            &OpName,
            &OpCategory,
            &OpImage,
            &OpInputs,
            &OpOutputs,
            &GraphId,
        ),
        Added<GraphId>,
    >,
) {
    for (entity, name, category, image, input_config, output_config, graph_id) in op_q.iter() {
        let (grid, _) = parent.single_mut();
        let index = ((*graph_id).index() as f32 / 100.0) + 10.0;
        let mut rng = rand::thread_rng();
        let size = images.get(&image.0).unwrap().size().as_vec2();

        commands.entity(grid).with_children(|parent| {
            parent
                .spawn((
                    OpRef(entity.clone()),
                    NodeRoot,
                    MaterialMesh2dBundle {
                        mesh: meshes
                            .add(Mesh::from(Rectangle::new(size.x / 4.0, size.y / 4.0)))
                            .into(),
                        material: materials.add(NodeMaterial {
                            selected: 0,
                            category_color: category.to_color(),
                            disabled: 0,
                            texture: (**image).clone(),
                        }),
                        transform: Transform::from_translation(Vec3::new(rng.gen::<f32>() * 80.0, rng.gen::<f32>() * 80.0, index)),
                        ..Default::default()
                    },
                    PickableBundle::default(),
                    On::<Pointer<Down>>::send_event::<ClickNode>(),
                    On::<Pointer<DragStart>>::target_insert(Pickable::IGNORE),
                    On::<Pointer<DragEnd>>::target_insert(Pickable::default()),
                    On::<Pointer<Drag>>::run(
                        |drag: ListenerMut<Pointer<Drag>>,
                         projection: Query<&OrthographicProjection, With<UiCamera>>,
                         mut transform: Query<&mut Transform, With<OpRef>>| {
                            if let Ok(mut transform) = transform.get_mut(drag.target) {
                                let projection = projection.single();

                                transform.translation.x += drag.delta.x * projection.scale;
                                transform.translation.y -= drag.delta.y * projection.scale;
                            }
                        },
                    ),
                ))
                .with_children(|parent| {
                    parent.spawn(
                        OpRefConnection
                    );
                    parent.spawn(
                        Text2dBundle {
                            text: Text::from_section(&name.0, TextStyle {
                                font: asset_server.load("fonts/Compagnon-Light.otf"),
                                font_size: 10.0,
                                color: Color::WHITE,
                                ..default()
                            }),
                            transform: Transform::from_translation(Vec3::new(0.0, -40.0, 0.001)),
                            ..default()
                        }
                    );
                    parent.spawn(
                        MaterialMesh2dBundle {
                            mesh: meshes
                                .add(Mesh::from(Rectangle::new(size.x / 4.0 - (size.x / 4.0 * 0.1), size.y / 4.0 - (size.y / 4.0 * 0.1))))
                                .into(),
                            material: color_materials.add(default_image.0.clone()),
                            transform: Transform::from_translation(Vec3::new(0.0, 0.0, -0.001)),
                            ..Default::default()
                        }
                    );
                    for i in 0..input_config.count {
                        let offset = -(size.x / 8.0);
                        spawn_port(&mut meshes, &mut color_materials, parent, InPort(i as u8), Vec3::new(offset, 0.0, -0.002));
                    }
                    for i in 0..output_config.count {
                        let offset = (size.x/ 8.0);
                        spawn_port(&mut meshes, &mut color_materials, parent, OutPort(i as u8), Vec3::new(offset, 0.0, -0.002));
                    }
                });
        });
    }
}

fn update_camera_enabled(
    mut commands: Commands,
    op_q: Query<(&GraphRef, &Camera), With<OpName>>,
    material_q: Query<&Handle<NodeMaterial>>,
    mut materials: ResMut<Assets<NodeMaterial>>,
) {
    for (graph_ref, mut camera) in op_q.iter() {
        let is_disabled = matches!(camera.output_mode, CameraOutputMode::Skip);
        let graph_entity = graph_ref.0;
        let material = material_q.get(graph_entity).unwrap();
        let mut material = materials.get_mut(material).unwrap();
        if is_disabled {
            commands.entity(graph_entity).insert(DisabledNode);
            material.disabled = 1;
        } else {
            commands.entity(graph_entity).remove::<DisabledNode>();
            material.disabled = 0;
        }
    }
}

fn update_graph_refs(
    mut commands: Commands,
    mut op_ref_q: Query<(Entity, &OpRef), (With<NodeRoot>, Added<OpRef>)>,
) {
    for (entity, op_ref) in op_ref_q.iter_mut() {
        commands.entity(op_ref.0).insert(GraphRef(entity));
    }
}

fn update_op_images(
    mut op_q: Query<(&GraphRef, &mut OpImage), Changed<OpImage>>,
    mut material_q: Query<&Handle<NodeMaterial>>,
    mut materials: ResMut<Assets<NodeMaterial>>,
) {
    for (graph_ref, mut image) in op_q.iter_mut() {
        let material = material_q.get_mut(**graph_ref).unwrap();
        let mut material = materials.get_mut(material).unwrap();
        material.texture = image.0.clone();
    }
}

fn spawn_port<T: Component>(
    meshes: &mut ResMut<Assets<Mesh>>,
    color_materials: &mut ResMut<Assets<ColorMaterial>>,
    parent: &mut ChildBuilder,
    port: T,
    translation: Vec3,
) {
    parent.spawn((
        port,
        Port,
        MaterialMesh2dBundle {
            mesh: meshes.add(Mesh::from(shape::Circle::new(10.0))).into(),
            material: color_materials.add(Color::rgb(0.5, 0.5, 0.5)),
            transform: Transform::from_translation(translation),
            ..Default::default()
        },
        PickableBundle::default(),
        On::<Pointer<DragStart>>::target_insert((Pickable::IGNORE, Connecting)), // Disable picking
        On::<Pointer<Drag>>::run(connection_drag),
        On::<Pointer<DragEnd>>::run(connection_drag_end),
    ));
}

fn connection_drag(
    event: Listener<Pointer<Drag>>,
    mut commands: Commands,
    camera_q: Query<(&Camera, &GlobalTransform), With<UiCamera>>,
    mut me_q: Query<
        (
            &GlobalTransform,
            Option<&Children>,
            Has<InPort>,
            Has<OutPort>,
        ),
        With<Connecting>,
    >,
    port_q: Query<(Entity, &GlobalTransform, Has<InPort>, Has<OutPort>), With<Port>>,
) {
    // TODO: this event sholdn't fire
    if let Ok((transform, children, is_input, is_output)) = me_q.get_mut(event.target()) {
        assert_ne!(is_input, is_output);

        if let Some(children) = children {
            for child in children.iter() {
                commands.entity(*child).despawn_recursive();
            }
        }

        let (camera, camera_transform) = camera_q.single();
        let start = Vec2::ZERO;
        let pointer_world = camera
            .viewport_to_world_2d(camera_transform, event.pointer_location.position)
            .expect("Failed to convert screen center to world coordinates");

        let mut end = pointer_world;

        // Snap to
        let mut closest_port = None;
        for (entity, transform, target_is_input, target_is_output) in port_q.iter() {
            if is_input && target_is_input || is_output && target_is_output {
                continue;
            }

            if transform.translation().xy().distance(pointer_world) < 40.0 {
                closest_port = Some((entity, transform, is_input));
            }
        }

        if let Some((to_entity, transform, is_input)) = closest_port {
            end = transform.translation().xy();
        }

        end -= transform.translation().xy();

        let entity = event.target();
        draw_connection(&mut commands, &start, &end, entity);
    }
}

fn connection_drag_end(
    mut commands: Commands,
    event: Listener<Pointer<DragEnd>>,
    camera_q: Query<(&Camera, &GlobalTransform), With<UiCamera>>,
    mut me_q: Query<
        (
            Entity,
            Option<&Children>,
            &Parent,
            Has<InPort>,
            Has<OutPort>,
        ),
        With<Connecting>,
    >,
    port_q: Query<(Entity, &Parent, &GlobalTransform, Has<InPort>, Has<OutPort>), With<Port>>,
    graph_ref_q: Query<&OpRef>,
    graph_id_q: Query<&GraphId>,
    mut graph_state: ResMut<GraphState>,
    mut ev_connect: EventWriter<Connect>,
) {
    let (from_entity, children, from_parent, is_input, is_output) =
        me_q.get_mut(event.target()).unwrap();
    assert_ne!(is_input, is_output);

    commands.entity(event.target()).insert(Pickable::default());
    commands.entity(event.target()).remove::<Connecting>();

    let (camera, camera_transform) = camera_q.single();
    let pointer_world = camera
        .viewport_to_world_2d(camera_transform, event.pointer_location.position)
        .expect("Failed to convert screen center to world coordinates");

    let mut closest_port = None;
    for (entity, parent, transform, target_is_input, target_is_output) in port_q.iter() {
        if is_input && target_is_input || is_output && target_is_output {
            continue;
        }

        if transform.translation().xy().distance(pointer_world) < 40.0 {
            closest_port = Some((entity, parent, transform, is_input));
        }
    }

    if let Some((to_entity, to_parent, transform, is_input)) = closest_port {
        let start = Vec2::ZERO;
        let end = camera
            .viewport_to_world_2d(camera_transform, event.pointer_location.position)
            .expect("Failed to convert screen center to world coordinates")
            - transform.translation().xy();

        let from_graph_ref = graph_ref_q.get(**from_parent).unwrap();
        let from_graph_id = graph_id_q.get(from_graph_ref.0).unwrap();
        let to_graph_ref = graph_ref_q.get(**to_parent).unwrap();
        let to_graph_id = graph_id_q.get(to_graph_ref.0).unwrap();

        // Ensure the connection is created on the output side
        if is_output {
            draw_connection(&mut commands, &start, &end, from_entity);
            commands.entity(from_entity).insert(ConnectedTo(to_entity));
            graph_state
                .graph
                .add_edge(from_graph_id.0, to_graph_id.0, (0, 0));
            ev_connect.send(Connect {
                output: from_graph_ref.0,
                input: to_graph_ref.0,
            });
        } else {
            draw_connection(&mut commands, &start, &end, to_entity);
            commands.entity(to_entity).insert(ConnectedTo(from_entity));
            graph_state
                .graph
                .add_edge(to_graph_id.0, from_graph_id.0, (0, 0));
            ev_connect.send(Connect {
                output: to_graph_ref.0,
                input: from_graph_ref.0,
            });
        }
    } else if let Some(children) = children {
        for child in children.iter() {
            commands.entity(*child).despawn_recursive();
        }
    }
}

fn draw_connection(commands: &mut Commands, start: &Vec2, end: &Vec2, entity: Entity) {
    commands.entity(entity).with_children(|parent| {
        let control_scale = ((end.x - start.x) / 2.0).max(30.0);
        let src_control = *start + Vec2::X * control_scale;
        let dst_control = *end - Vec2::X * control_scale;

        let mut path_builder = PathBuilder::new();
        path_builder.move_to(*start);
        path_builder.cubic_bezier_to(src_control, dst_control, *end);
        let path = path_builder.build();
        parent.spawn((
            ShapeBundle {
                path,
                spatial: SpatialBundle {
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, -5.03)),
                    ..default()
                },
                ..default()
            },
            Stroke::new(Color::BLACK, 4.0),
            Pickable::IGNORE,
        ));
    });
}

fn draw_refs(
    mut commands: Commands,
    params_q: Query<(&ParamValue, &Parent)>,
    graph_q: Query<&GraphRef>,
    transform_q: Query<&GlobalTransform>,
    children_q: Query<&Children, With<NodeRoot>>,
    op_ref_connection_q: Query<Entity, With<OpRefConnection>>,
) {
    for (param_value, parent) in params_q.iter() {
        match param_value {
            ParamValue::TextureOp(Some(entity)) | ParamValue::MeshOp(Some(entity)) => {
                let parent = parent.get();
                let from_graph_ref = graph_q.get(parent).unwrap();
                let to_graph_ref = graph_q.get(*entity).unwrap();
                let from_transform = transform_q.get(from_graph_ref.0).unwrap();
                let to_transform = transform_q.get(to_graph_ref.0).unwrap();
                let from_children = children_q.get(from_graph_ref.0).unwrap();

                let start = Vec2::ZERO;
                let end = to_transform.translation().xy() - from_transform.translation().xy();

                for child in from_children.iter() {
                    let entity = *child;
                    if let Ok(entity) = op_ref_connection_q.get(entity) {
                        commands.entity(entity).insert((
                            ShapeBundle {
                                spatial: SpatialBundle {
                                    transform: Transform::from_translation(Vec3::new(
                                        0.0, 0.0, -0.4,
                                    )),
                                    ..default()
                                },
                                path: GeometryBuilder::build_as(&Line(start, end)),
                                ..default()
                            },
                            Stroke::new(Color::GRAY, 1.5),
                        ));
                    }
                }
            }
            _ => {}
        }
    }
}

fn update_connections(
    mut commands: Commands,
    port_children_q: Query<&Children, (With<Port>, Without<Connecting>)>,
    out_port_q: Query<(Entity, &GlobalTransform, &ConnectedTo), With<OutPort>>,
    in_port_q: Query<(Entity, &GlobalTransform, &Transform), With<InPort>>,
) {
    for children in port_children_q.iter() {
        for child in children.iter() {
            commands.entity(*child).despawn_recursive();
        }
    }

    // Connect inputs to outputs
    for (in_entity, transform, in_connected_to) in out_port_q.iter() {
        let (out_entity, output_global_transform, output_transform) =
            in_port_q.get(in_connected_to.0).unwrap();

        let start = Vec2::ZERO;
        let end = output_global_transform.translation().xy() - transform.translation().xy();

        draw_connection(&mut commands, &start, &end, in_entity);
    }
}

fn add_graph_ids(
    mut commands: Commands,
    mut graph: ResMut<GraphState>,
    mut textures: Query<(Entity), (With<OpName>, Without<GraphId>)>,
) {
    for entity in textures.iter_mut() {
        let node_id = graph.graph.add_node(GraphNode {});
        commands.entity(entity).insert(GraphId(node_id));
    }
}

pub type Layout = HashMap<NodeIndex, Vec2>;

pub fn layout(
    nodes: impl IntoIterator<Item = (NodeIndex, Vec2)>,
    edges: impl IntoIterator<Item = (NodeIndex, NodeIndex)>,
) -> Layout {
    let orientation = Orientation::LeftToRight;
    let mut vg = VisualGraph::new(orientation);

    let mut handles = HashMap::new();
    let mut ids = Vec::new();
    for (id, size) in nodes {
        let shape = ShapeKind::new_box("");
        let style = StyleAttr::simple();
        let size = Point::new(size.x.into(), size.y.into());
        let node = Element::create(shape, style, orientation, size);
        let handle = vg.add_node(node);
        handles.insert(id, handle);
        ids.push((handle, id));
    }

    for (a, b) in edges {
        let edge = Arrow::default();
        let na = handles[&a];
        let nb = handles[&b];
        vg.add_edge(edge, na, nb);
    }

    vg.to_valid_dag();
    vg.split_text_edges();
    let disable_opts = false;
    vg.split_long_edges(disable_opts);
    Placer::new(&mut vg).layout(true);

    let mut layout = Layout::new();
    for (handle, id) in ids {
        let pos = vg.pos(handle);
        let with_halo = false;
        let (tl, br) = pos.bbox(with_halo);
        let pos: Vec2 = [tl.x as f32, tl.y as f32].into();
        let pos = pos * 3.0;
        layout.insert(id, pos);
    }

    layout
}
