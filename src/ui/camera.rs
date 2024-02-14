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
