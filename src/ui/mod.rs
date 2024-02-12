use bevy::ecs::query::{QueryItem, WorldQuery};
use bevy::prelude::*;
use bevy::render::view::VisibleEntities;
use bevy::sprite::MaterialMesh2dBundle;
use bevy_egui::{egui, EguiContexts};
use bevy_mod_picking::prelude::*;
use bevy_mod_picking::{DefaultPickingPlugins, PickableBundle};
use camera_controller::CameraControllerPlugin;
use crate::texture::TextureNodeImage;
use crate::ui::event::ClickNode;
use crate::ui::graph::{GraphPlugin, SelectedNode};
use crate::ui::grid::{InfiniteGrid, InfiniteGridPlugin, InfiniteGridSettings};

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
        .add_systems(Startup, ui_setup)
        .add_systems(Update, ui);
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
    commands.spawn((
        InfiniteGridSettings {
            // shadow_color: None,
            x_axis_color: Color::rgb(1.0, 0.2, 0.2),
            z_axis_color: Color::rgb(0.2, 0.2, 1.0),
            ..default()
        },
        InfiniteGrid,
        VisibleEntities::default(),
        MaterialMesh2dBundle {
            mesh: meshes
                .add(Mesh::from(shape::Quad {
                    size: Vec2::new(window.width(), window.height()),
                    ..Default::default()
                }))
                .into(),
            material: materials.add(Color::rgba(0.0, 0.0, 0.0, 0.0)),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        },
        PickableBundle::default(), // <- Makes the mesh pickable.
        On::<Pointer<DragStart>>::target_insert(Pickable::IGNORE), // Disable picking
        On::<Pointer<DragEnd>>::target_insert(Pickable::default()), // Re-enable picking
        On::<Pointer<Drag>>::target_component_mut::<Transform>(|drag, transform| {
            transform.translation.x += drag.delta.x; // Make the square follow the mouse
            transform.translation.y -= drag.delta.y;
        }),
    ));

    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_translation(Vec3::new(10., 10., 100.0)),
            ..default()
        },
        camera_controller::CameraController::default(),
    ));
}

mod camera_controller {
    use bevy::input::mouse::MouseWheel;
    use bevy::input::touchpad::TouchpadMagnify;
    use bevy::prelude::*;
    use bevy::sprite::Mesh2dHandle;

    use crate::ui::grid::InfiniteGridSettings;

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
        mut grid_settings: Query<(&mut InfiniteGridSettings, &mut Transform, &Mesh2dHandle)>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut query: Query<(&mut CameraController), With<Camera>>,
    ) {
        if let Ok((mut state)) = query.get_single_mut() {
            // Handle zoom input
            let min = 0.1;
            let max = 3.0;

            for ev_scroll in scroll_evr.read() {
                if ev_scroll.y != 0.0 {
                    for (mut settings, mut transform, mesh) in grid_settings.iter_mut() {
                        let scale = (settings.scale + ev_scroll.y * 0.001).clamp(min, max);
                        transform.scale = Vec3::new(1.0 / scale, 1.0 / scale, transform.scale.z);
                        settings.scale = scale;
                    }
                }
            }

            for ev_magnify in evr_touchpad_magnify.read() {
                for (mut settings, mut transform, mesh) in grid_settings.iter_mut() {
                    let scale = (settings.scale + ev_magnify.0 * 0.001).clamp(min, max);
                    transform.scale = Vec3::new(1.0 / scale, 1.0 / scale, transform.scale.z);
                    settings.scale = scale;
                }
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
