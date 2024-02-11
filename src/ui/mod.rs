use bevy::ecs::query::{QueryItem, WorldQuery};
use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy_egui::{egui, EguiContexts};

use crate::texture::TextureNodeImage;
use crate::ui::graph::GraphPlugin;
use crate::ui::grid::{InfiniteGridBundle, InfiniteGridPlugin, InfiniteGridSettings};

pub mod graph;
pub mod grid;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            GraphPlugin,
            camera_controller::CameraControllerPlugin,
            InfiniteGridPlugin,
        ))
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
) {
    commands.spawn(InfiniteGridBundle {
        settings: InfiniteGridSettings {
            // shadow_color: None,
            x_axis_color: Color::rgb(1.0, 0.2, 0.2),
            z_axis_color: Color::rgb(0.2, 0.2, 1.0),
            ..default()
        },
        ..default()
    });

    // // Circle
    // commands.spawn(MaterialMesh2dBundle {
    //     mesh: meshes.add(<shape::Circle as Into<Mesh>>::into(shape::Circle::new(50.))).into(),
    //     material: materials.add(ColorMaterial::from(Color::PURPLE)),
    //     transform: Transform::from_translation(Vec3::new(-150., 0., 0.)),
    //     ..default()
    // });


    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_translation(Vec3::new(10., 10., 100.0)),
            ..default()
        },
        camera_controller::CameraController::default(),
    ));
}

mod camera_controller {
    use std::f32::consts::*;

    use bevy::{input::mouse::MouseMotion, prelude::*};
    use bevy::input::mouse::MouseWheel;
    use bevy::input::touchpad::TouchpadMagnify;
    use crate::ui::grid::InfiniteGridSettings;

    pub const RADIANS_PER_DOT: f32 = 1.0 / 180.0;

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
        time: Res<Time>,
        mut mouse_events: EventReader<MouseMotion>,
        mouse_button_input: Res<ButtonInput<MouseButton>>,
        mut evr_touchpad_magnify: EventReader<TouchpadMagnify>,
        mut scroll_evr: EventReader<MouseWheel>,
        mut grid_settings: Query<(&mut InfiniteGridSettings, &mut Transform)>,
        mut query: Query<(&mut CameraController), With<Camera>>,
    ) {
        let dt = time.delta_seconds();

        if let Ok((mut state)) = query.get_single_mut() {
            // Handle mouse input
            let mut mouse_delta = Vec2::ZERO;
            if mouse_button_input.pressed(MouseButton::Left) {
                for mouse_event in mouse_events.read() {
                    mouse_delta += mouse_event.delta;
                }
            }

            if mouse_delta != Vec2::ZERO {
                state.velocity = Vec3::new(-mouse_delta.x, mouse_delta.y, 0.0);
            } else {
                state.velocity *= 0.9; // friction
                if state.velocity.length_squared() < 1e-6 {
                    state.velocity = Vec3::ZERO;
                }
            }

            for (_settings, mut transform) in grid_settings.iter_mut() {
                transform.translation += state.velocity;
            }

            // Handle zoom input
            let min = 0.1;
            let max = 2.0;

            for ev_scroll in scroll_evr.read() {
                for (mut settings, _transform) in grid_settings.iter_mut() {
                    settings.scale = (settings.scale + ev_scroll.y * 0.001).clamp(min, max);
                }
            }

            for ev_magnify in evr_touchpad_magnify.read() {
                for (mut settings, _transform) in grid_settings.iter_mut() {
                    settings.scale = (settings.scale + ev_magnify.0 * 0.5) .clamp(min, max);
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