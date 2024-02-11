use bevy::prelude::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy_egui::{egui, EguiContexts};

use crate::texture::TextureNodeImage;
use crate::ui::grid::{InfiniteGridBundle, InfiniteGridPlugin, InfiniteGridSettings};

pub mod graph;
pub mod grid;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            // GraphPlugin,
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

    pub const RADIANS_PER_DOT: f32 = 1.0 / 180.0;

    #[derive(Component)]
    pub struct CameraController {
        pub pitch: f32,
        pub yaw: f32,
        pub velocity: Vec3,
    }

    impl Default for CameraController {
        fn default() -> Self {
            Self {
                pitch: 0.0,
                yaw: 0.0,
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
        key_input: Res<ButtonInput<KeyCode>>,
        mut query: Query<(&mut Transform, &mut CameraController), With<Camera>>,
    ) {
        let dt = time.delta_seconds();

        if let Ok((mut transform, mut state)) = query.get_single_mut() {
            // Handle key input
            let mut axis_input = Vec3::ZERO;
            if key_input.pressed(KeyCode::KeyW) {
                axis_input.z += 1.0;
            }
            if key_input.pressed(KeyCode::KeyS) {
                axis_input.z -= 1.0;
            }
            if key_input.pressed(KeyCode::KeyD) {
                axis_input.x += 1.0;
            }
            if key_input.pressed(KeyCode::KeyA) {
                axis_input.x -= 1.0;
            }
            if key_input.pressed(KeyCode::KeyE) {
                axis_input.y += 1.0;
            }
            if key_input.pressed(KeyCode::KeyQ) {
                axis_input.y -= 1.0;
            }

            // Apply movement update
            if axis_input != Vec3::ZERO {
                let max_speed = if key_input.pressed(KeyCode::ShiftLeft) {
                    15.0
                } else {
                    5.0
                };
                state.velocity = axis_input.normalize() * max_speed;
            } else {
                state.velocity *= 0.5; // friction
                if state.velocity.length_squared() < 1e-6 {
                    state.velocity = Vec3::ZERO;
                }
            }
            let forward = transform.forward().z;
            let right = transform.right().x;
            transform.translation += state.velocity.x * dt * right
                + state.velocity.y * dt * Vec3::Y
                + state.velocity.z * dt * forward;

            // Handle mouse input
            let mut mouse_delta = Vec2::ZERO;
            if mouse_button_input.pressed(MouseButton::Left) {
                for mouse_event in mouse_events.read() {
                    mouse_delta += mouse_event.delta;
                }
            }
            if mouse_delta != Vec2::ZERO {
                // Apply look update
                state.pitch =
                    (state.pitch - mouse_delta.y * RADIANS_PER_DOT).clamp(-PI / 2., PI / 2.);
                state.yaw -= mouse_delta.x * RADIANS_PER_DOT;
                transform.rotation = Quat::from_euler(EulerRot::ZYX, 0.0, state.yaw, state.pitch);
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
