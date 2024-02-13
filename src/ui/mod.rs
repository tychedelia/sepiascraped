use std::ops::Deref;
use bevy::ecs::query::{QueryItem, WorldQuery};
use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use bevy::render::view::VisibleEntities;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy_egui::{egui, EguiContexts};
use bevy_mod_picking::prelude::*;
use bevy_mod_picking::{DefaultPickingPlugins, PickableBundle};

use camera_controller::CameraControllerPlugin;

use crate::texture::TextureNodeImage;
use crate::ui::event::ClickNode;
use crate::ui::graph::{GraphPlugin, GraphRef};
use crate::ui::grid::{InfiniteGrid, InfiniteGridBundle, InfiniteGridPlugin, InfiniteGridSettings};

mod event;
pub mod graph;
pub mod grid;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            GraphPlugin,
            CameraControllerPlugin,
            InfiniteGridPlugin,
            DefaultPickingPlugins,
        ))
        .add_event::<ClickNode>()
            .init_resource::<UiState>()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0,
        })
            .insert_resource(PreviousScale {
                scale: 1.0
            })
        .add_systems(Startup, ui_setup)
        .add_systems(Update, (ui, resize_grid_drag));
    }
}

#[derive(Resource, Default)]
pub struct UiState {
    pub side_panel: Option<egui::Response>,
}

pub fn ui_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    window: Query<&Window>,
) {
    let window = window.single();
    commands
        .spawn(
            (InfiniteGridBundle {
                settings: InfiniteGridSettings {
                    // shadow_color: None,
                    x_axis_color: Color::rgb(1.0, 0.2, 0.2),
                    y_axis_color: Color::rgb(0.2, 0.2, 1.0),
                    ..default()
                },
                ..default()
            }),
        )
        .with_children(|parent| {
            let parent_entity = parent.parent_entity();
            parent.spawn((
                GridDrag,
                MaterialMesh2dBundle {
                    mesh: meshes
                        .add(Mesh::from(shape::Quad {
                            size: Vec2::new(window.width() + 10.0, window.height() + 10.0),
                            ..Default::default()
                        }))
                        .into(),
                    material: materials.add(Color::rgba(0.0, 0.0, 0.0, 0.0)),
                    ..Default::default()
                },
                PickableBundle::default(), // <- Makes the mesh pickable.
                On::<Pointer<DragStart>>::target_insert(Pickable::IGNORE), // Disable picking
                On::<Pointer<DragEnd>>::target_insert(Pickable::default()), // Re-enable picking
                On::<Pointer<Drag>>::run(
                    move |drag: Listener<Pointer<Drag>>,
                          window: Query<&Window>,
                          projection: Query<(
                        &Camera,
                        &OrthographicProjection,
                        &GlobalTransform,
                    )>,
                          mut transform: Query<&mut Transform>| {
                        let window = window.single();
                        let (camera, projection, camera_transform) = projection.single();

                        let mut parent_transform = transform.get_mut(parent_entity).unwrap();
                        parent_transform.translation.x += drag.delta.x * projection.scale;
                        parent_transform.translation.y -= drag.delta.y * projection.scale;

                        let screen_center = Vec2::new(window.width() / 2.0, window.height() / 2.0);
                        let world_center = camera
                            .viewport_to_world_2d(camera_transform, screen_center)
                            .expect("Failed to convert screen center to world coordinates");
                        let relative_translation =
                            world_center.extend(0.0) - parent_transform.translation;
                        let mut transform = transform.get_mut(drag.target).unwrap();
                        transform.translation.x = relative_translation.x;
                        transform.translation.y = relative_translation.y;
                    },
                ),
            ));
        });

    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_translation(Vec3::new(0.1, 0.1, 0.0)),
            ..default()
        },
        camera_controller::CameraController::default(),
    ));
}

#[derive(Component)]
struct GridDrag;

#[derive(Resource)]
struct PreviousScale {
    scale: f32,
}

pub fn resize_grid_drag(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    grid_drag: Query<(Entity, &Mesh2dHandle), With<GridDrag>>,
    projection: Query<&OrthographicProjection>,
    mut previous_scale: ResMut<PreviousScale>,
    window: Query<&Window>,
) {
    let current_scale = projection.single().scale;
    let previous_scale_value = previous_scale.scale;
    let window = window.single();
    // let new_mesh = meshes
    //     .add(Mesh::from(shape::Quad {
    //         size: Vec2::new(window.width() * current_scale + 10.0, window.height() * current_scale + 10.0),
    //         ..Default::default()
    //     }));


    // Calculate the percentage change
    let scale_change = current_scale / previous_scale_value;

    let (entity, mesh) = grid_drag.single();

    // commands.entity(entity).insert(Mesh2dHandle(new_mesh));

    let mesh = meshes.get_mut(&mesh.0).expect("Failed to get grid drag mesh");
    //
    // println!("scale_change: {}", scale_change);
    // // Scale the mesh by the percentage change
    mesh.scale_by(Vec2::splat(scale_change).extend(1.0));
    commands.entity(entity).remove::<Aabb>();

    // Update the previous scale
    previous_scale.scale = current_scale;
}

mod camera_controller {
    use bevy::input::mouse::MouseWheel;
    use bevy::input::touchpad::TouchpadMagnify;
    use bevy::prelude::*;

    #[derive(Component)]
    pub struct CameraController {
        pub velocity: Vec3,
    }

    impl Default for CameraController {
        fn default() -> Self {
            Self {
                velocity: Vec3::ZERO,
            }
        }
    }

    pub struct CameraControllerPlugin;

    impl Plugin for CameraControllerPlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(Update, camera_controller);
        }
    }

    fn camera_controller(
        mut evr_touchpad_magnify: EventReader<TouchpadMagnify>,
        mut scroll_evr: EventReader<MouseWheel>,
        mut query: Query<(&mut OrthographicProjection)>,
    ) {
        if let Ok((mut projection)) = query.get_single_mut() {
            // Handle zoom input
            let min = 0.1;
            let max = 3.0;

            for ev_scroll in scroll_evr.read() {
                if ev_scroll.y != 0.0 {
                    let scale = (projection.scale + ev_scroll.y * 0.001).clamp(min, max);
                    projection.scale = scale;
                }
            }

            for ev_magnify in evr_touchpad_magnify.read() {
                let scale = (projection.scale + ev_magnify.0 * 0.001).clamp(min, max);
                projection.scale = scale;
            }
        }
    }
}

pub fn ui(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UiState>,
    query: Query<(&TextureNodeImage)>,
) {
}

trait Ui {
    fn side_panel<Q: WorldQuery>(&self, ui: &mut egui::Ui, item: QueryItem<Q>);
}
