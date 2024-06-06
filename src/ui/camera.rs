use crate::ui::UiCamera;
use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

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
    mut scroll_evr: EventReader<MouseWheel>,
    mut query: Query<&mut OrthographicProjection, With<UiCamera>>,
) {
    if let Ok(mut projection) = query.get_single_mut() {
        // Handle zoom input
        let min = 0.1;
        let max = 3.0;

        for ev_scroll in scroll_evr.read() {
            if ev_scroll.y != 0.0 {
                let scale = (projection.scale + ev_scroll.y * 0.01).clamp(min, max);
                projection.scale = scale;
            }
        }
    }
}
