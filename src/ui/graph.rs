use bevy::asset::LoadState;
use bevy::color::palettes::css::{BLACK, GRAY};
use bevy::ecs::entity::Entities;
use bevy::ecs::system::lifetimeless::{Read, SQuery, SResMut, Write};
use bevy::prelude::*;
use bevy::render::camera::CameraOutputMode;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle};
use bevy::transform::TransformSystem::TransformPropagate;
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

use crate::engine::graph::event::{ClickNode, Connect, Disconnect};
use crate::engine::graph::{GraphId, GraphNode, GraphState, Layout};
use crate::engine::op::texture::render::TextureOpInputImages;
use crate::engine::op::texture::TextureOp;
use crate::engine::op::{OpCategory, OpDefaultImage, OpImage, OpInputs, OpName, OpOutputs, OpRef};
use crate::engine::param::ParamValue;
use crate::ui::grid::InfiniteGridSettings;
use crate::ui::UiCamera;
use crate::{engine::graph, Sets};

pub struct GraphPlugin;

impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<NodeMaterial>::default())
            .add_systems(Startup, setup)
            .add_systems(First, (ensure_despawn_nodes, update_op_images))
            .add_systems(
                Update,
                (
                    (update_graph).in_set(Sets::Graph),
                    (
                        ui,
                        update_camera_enabled,
                        update_ui_refs,
                        draw_refs,
                        do_layout,
                        click_node.run_if(on_event::<ClickNode>()),
                        handle_connect.run_if(on_event::<Connect>()),
                        handle_disconnect.run_if(on_event::<Disconnect>()),
                    )
                        .chain()
                        .in_set(Sets::Ui),
                ),
            )
            .add_systems(
                PostUpdate,
                (
                    update_connections,
                )
                    .in_set(TransformPropagate),
            );
    }
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Components
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[derive(Component, Deref, DerefMut, Copy, Clone, PartialEq, Eq, Hash, Debug, Ord, PartialOrd)]
pub struct UiRef(pub Entity);

#[derive(Component, Clone)]
pub struct SelectedNode;

#[derive(Component)]
pub struct ConnectionWire;

#[derive(Component)]
pub struct Port;

#[derive(Component, Eq, PartialEq)]
pub struct PortCategory(pub &'static str);

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

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Assets
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct NodeMaterial {
    #[uniform(0)]
    pub selected: u32,
    #[uniform(0)]
    pub category_color: LinearRgba,
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
                            category_color: category.to_color().linear(),
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
                    let spacing = 40.0;
                    for i in 0..input_config.count {
                        let offset_x = -(size.x / 8.0);
                        let total_height = spacing * ((input_config.count - 1) as f32);
                        let offset_y = i as f32 * spacing - total_height / 2.0;
                        spawn_port(&mut meshes, &mut color_materials, parent, InPort(i as u8), PortCategory(category.0), Vec3::new(offset_x, offset_y as f32, -0.002));
                    }
                    for i in 0..output_config.count {
                        let offset_x = (size.x / 8.0);
                        let total_height = spacing * ((output_config.count - 1) as f32);
                        let offset_y = i as f32 * spacing - total_height / 2.0;
                        spawn_port(&mut meshes, &mut color_materials, parent, OutPort(i as u8), PortCategory(category.0), Vec3::new(offset_x, offset_y, -0.002));
                    }
                });
        });
    }
}

fn update_camera_enabled(
    mut commands: Commands,
    op_q: Query<(&UiRef, &Camera), With<OpName>>,
    material_q: Query<&Handle<NodeMaterial>>,
    mut materials: ResMut<Assets<NodeMaterial>>,
) {
    for (ui_ref, mut camera) in op_q.iter() {
        let is_disabled = matches!(camera.output_mode, CameraOutputMode::Skip);
        let graph_entity = ui_ref.0;
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

pub fn update_ui_refs(
    mut commands: Commands,
    mut op_ref_q: Query<(Entity, &OpRef), (With<NodeRoot>, Added<OpRef>)>,
) {
    for (entity, op_ref) in op_ref_q.iter_mut() {
        commands.entity(op_ref.0).insert(UiRef(entity));
    }
}

fn update_op_images(
    mut op_q: Query<(&UiRef, &mut OpImage), Changed<OpImage>>,
    mut material_q: Query<&Handle<NodeMaterial>>,
    mut materials: ResMut<Assets<NodeMaterial>>,
) {
    for (ui_ref, mut image) in op_q.iter_mut() {
        let material = material_q.get_mut(**ui_ref).unwrap();
        let mut material = materials.get_mut(material).unwrap();
        material.texture = image.0.clone();
    }
}

fn spawn_port<T: Component>(
    meshes: &mut ResMut<Assets<Mesh>>,
    color_materials: &mut ResMut<Assets<ColorMaterial>>,
    parent: &mut ChildBuilder,
    port: T,
    category: PortCategory,
    translation: Vec3,
) {
    parent.spawn((
        port,
        Port,
        category,
        MaterialMesh2dBundle {
            mesh: meshes.add(Mesh::from(Circle::new(10.0))).into(),
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
            &PortCategory,
            Has<InPort>,
            Has<OutPort>,
        ),
        With<Connecting>,
    >,
    port_q: Query<
        (
            Entity,
            &GlobalTransform,
            &PortCategory,
            Has<InPort>,
            Has<OutPort>,
        ),
        With<Port>,
    >,
) {
    // TODO: this event sholdn't fire
    if let Ok((transform, children, category, is_input, is_output)) = me_q.get_mut(event.target()) {
        assert_ne!(is_input, is_output);

        let connection_wire = match children {
            None => commands.entity(event.target).with_children(|parent| {
                parent.spawn((
                    ConnectionWire,
                    ShapeBundle {
                        spatial: SpatialBundle {
                            transform: Transform::from_translation(Vec3::new(0.0, 0.0, -5.03)),
                            ..default()
                        },
                        ..default()
                    },
                    Stroke::new(Color::from(BLACK), 4.0),
                    Pickable::IGNORE,
                ));
            }).id(),
            Some(children) => *children.first().unwrap()
        };

        let (camera, camera_transform) = camera_q.single();
        let start = Vec2::ZERO;
        let pointer_world = camera
            .viewport_to_world_2d(camera_transform, event.pointer_location.position)
            .expect("Failed to convert screen center to world coordinates");

        let mut end = pointer_world;

        // Snap to
        let mut closest_port = None;
        for (entity, transform, target_category, target_is_input, target_is_output) in port_q.iter()
        {
            if is_input && target_is_input || is_output && target_is_output {
                continue;
            }
            if target_category != category {
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
        draw_connection(&mut commands, &start, &end, connection_wire);
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
            &PortCategory,
            Has<InPort>,
            Has<OutPort>,
            Option<&InPort>,
            Option<&OutPort>,
        ),
        With<Connecting>,
    >,
    port_q: Query<
        (
            Entity,
            &Parent,
            &GlobalTransform,
            &PortCategory,
            Has<InPort>,
            Has<OutPort>,
            Option<&InPort>,
            Option<&OutPort>,
        ),
        With<Port>,
    >,
    ui_ref_q: Query<&OpRef>,
    mut ev_connect: EventWriter<Connect>,
) {
    let (
        from_entity,
        children,
        from_parent,
        category,
        is_input,
        is_output,
        from_maybe_in_port,
        from_maybe_out_port,
    ) = me_q.get_mut(event.target()).unwrap();
    assert_ne!(is_input, is_output);

    commands.entity(event.target()).insert(Pickable::default());
    commands.entity(event.target()).remove::<Connecting>();

    let (camera, camera_transform) = camera_q.single();
    let pointer_world = camera
        .viewport_to_world_2d(camera_transform, event.pointer_location.position)
        .expect("Failed to convert screen center to world coordinates");

    let mut closest_port = None;
    for (
        entity,
        parent,
        transform,
        target_category,
        target_is_input,
        target_is_output,
        target_maybe_in_port,
        target_maybe_out_port,
    ) in port_q.iter()
    {
        if is_input && target_is_input || is_output && target_is_output {
            continue;
        }
        if target_category != category {
            continue;
        }

        if transform.translation().xy().distance(pointer_world) < 40.0 {
            closest_port = Some((
                entity,
                parent,
                transform,
                target_maybe_in_port,
                target_maybe_out_port,
            ));
        }
    }

    if let Some((to_entity, to_parent, transform, to_maybe_in_port, to_maybe_out_port)) =
        closest_port
    {
        let start = Vec2::ZERO;
        let end = camera
            .viewport_to_world_2d(camera_transform, event.pointer_location.position)
            .expect("Failed to convert screen center to world coordinates")
            - transform.translation().xy();

        let from_op_ref = ui_ref_q.get(**from_parent).unwrap();
        let to_op_ref = ui_ref_q.get(**to_parent).unwrap();

        // Ensure the connection is created on the output side
        if is_output {
            draw_connection(&mut commands, &start, &end, from_entity);
            ev_connect.send(Connect {
                output: from_op_ref.0,
                input: to_op_ref.0,
                output_port: from_maybe_out_port.unwrap().0,
                input_port: to_maybe_in_port.unwrap().0,
            });
        } else {
            draw_connection(&mut commands, &end, &start, to_entity);
            ev_connect.send(Connect {
                output: to_op_ref.0,
                input: from_op_ref.0,
                output_port: to_maybe_out_port.unwrap().0,
                input_port: from_maybe_in_port.unwrap().0,
            });
        }
    }
}

fn handle_connect(
    mut commands: Commands,
    ui_ref_q: Query<&UiRef>,
    children_q: Query<&Children>,
    out_port_q: Query<(Entity, &OutPort)>,
    in_port_q: Query<(Entity, &InPort)>,
    mut ev_connect: EventReader<Connect>,
    mut materials: ResMut<Assets<NodeMaterial>>,
    op_q: Query<(&OpImage, &OpInputs, &UiRef)>,
    material_q: Query<&Handle<NodeMaterial>>,
) {
    for connect in ev_connect.read() {
        let ui_ref = ui_ref_q.get(connect.output).unwrap();
        let children = children_q.get(ui_ref.0).unwrap();
        for child in children {
            if let Ok((out_entity, out_port)) = out_port_q.get(*child) {
                if out_port.0 == connect.output_port {
                    let ui_ref = ui_ref_q.get(connect.input).unwrap();
                    let children = children_q.get(ui_ref.0).unwrap();
                    for child in children {
                        if let Ok((in_entity, in_port)) = in_port_q.get(*child) {
                            if in_port.0 == connect.input_port {
                                commands.entity(out_entity).insert(ConnectedTo(in_entity));
                            }
                        }
                    }
                }
            }
        }
    }
}

fn handle_disconnect(
    mut commands: Commands,
    ui_ref_q: Query<&UiRef>,
    children_q: Query<&Children>,
    port_q: Query<(Entity, &OutPort)>,
    mut ev_disconnect: EventReader<Disconnect>,
) {
    for disconnect in ev_disconnect.read() {
        let ui_ref = ui_ref_q.get(disconnect.output).unwrap();
        let children = children_q.get(ui_ref.0).unwrap();
        for child in children {
            if let Ok((entity, out_port)) = port_q.get(*child) {
                if out_port.0 == disconnect.output_port {
                    commands.entity(entity).remove::<ConnectedTo>();
                }
            }
        }
    }
}

fn draw_connection(commands: &mut Commands, start: &Vec2, end: &Vec2, entity: Entity) {
    let control_scale = ((end.x - start.x) / 2.0).max(30.0);
    let src_control = *start + Vec2::X * control_scale;
    let dst_control = *end - Vec2::X * control_scale;

    let mut path_builder = PathBuilder::new();
    path_builder.move_to(*start);
    path_builder.cubic_bezier_to(src_control, dst_control, *end);
    let path = path_builder.build();
    commands.entity(entity).insert(path);
}

fn draw_refs(
    mut commands: Commands,
    params_q: Query<(&ParamValue, &Parent)>,
    graph_q: Query<&UiRef>,
    transform_q: Query<&GlobalTransform>,
    children_q: Query<&Children, With<NodeRoot>>,
    op_ref_connection_q: Query<Entity, With<OpRefConnection>>,
) {
    for (param_value, parent) in params_q.iter() {
        match param_value {
            ParamValue::TextureOp(Some(entity)) | ParamValue::MeshOp(Some(entity)) => {
                let parent = parent.get();
                let from_ui_ref = graph_q.get(parent).unwrap();
                let to_ui_ref = graph_q.get(*entity).unwrap();
                let from_transform = transform_q.get(from_ui_ref.0).unwrap();
                let to_transform = transform_q.get(to_ui_ref.0).unwrap();
                let from_children = children_q.get(from_ui_ref.0).unwrap();

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
                            Stroke::new(Color::from(GRAY), 1.5),
                        ));
                    }
                }
            }
            _ => {}
        }
    }
}

fn despawn_connections(
    mut commands: Commands,
    port_children_q: Query<&Children, (With<Port>, Without<Connecting>)>,
) {
    for children in port_children_q.iter() {
        for child in children.iter() {
            commands.entity(*child).despawn_recursive();
        }
    }
}

fn update_connections(
    mut commands: Commands,
    out_port_q: Query<(Entity, &GlobalTransform, &ConnectedTo), With<OutPort>>,
    in_port_q: Query<(Entity, &GlobalTransform, &Transform), With<InPort>>,
) {
    // Connect inputs to outputs
    for (in_entity, transform, in_connected_to) in out_port_q.iter() {
        let (out_entity, output_global_transform, output_transform) =
            in_port_q.get(in_connected_to.0).unwrap();

        let start = Vec2::ZERO;
        let end = output_global_transform.translation().xy() - transform.translation().xy();

        draw_connection(&mut commands, &start, &end, in_entity);
    }
}

pub fn update_graph(
    mut state: ResMut<GraphState>,
    mut connected_q: Query<&Parent, Added<ConnectedTo>>,
) {
    if !connected_q.is_empty() {
        state.layout = layout(
            state.graph.node_indices().map(|index| (index, Vec2::ZERO)),
            state.graph.edge_indices().map(|index| {
                let (a, b) = state.graph.edge_endpoints(index).unwrap();
                (a, b)
            }),
        );
    }
}

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

fn ensure_despawn_nodes(
    mut commands: Commands,
    mut entity_q: Query<Entity, With<UiRef>>,
    mut all_nodes_q: Query<(Entity, &OpRef), With<NodeRoot>>,
) {
    for (entity, op_ref) in all_nodes_q.iter_mut() {
        if !entity_q.contains(op_ref.0) {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn do_layout(
    mut state: Res<GraphState>,
    mut all_nodes_q: Query<(&OpRef, &mut Transform)>,
    graph_id_q: Query<&GraphId>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if keys.just_pressed(KeyCode::KeyL) {
        for (op_ref, mut transform) in all_nodes_q.iter_mut() {
            let graph_id = graph_id_q.get(**op_ref).unwrap();
            if let Some(pos) = state.layout.get(&graph_id.0) {
                transform.translation.x = pos.x;
                transform.translation.y = pos.y;
            }
        }
    }
}
