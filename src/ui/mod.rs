use bevy::prelude::*;
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
            // GraphPlugin,
            InfiniteGridPlugin))
            .init_resource::<UiState>()
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
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(InfiniteGridBundle {
        settings: InfiniteGridSettings {
            // shadow_color: None,
            ..default()
        },
        ..default()
    });
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 4.37, 14.77),
            ..default()
        },
    ));

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::X * 15. + Vec3::Y * 20.)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    let mat = standard_materials.add(StandardMaterial::default());

    // cube
    commands.spawn(PbrBundle {
        material: mat.clone(),
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        transform: Transform {
            translation: Vec3::new(3., 4., 0.),
            rotation: Quat::from_rotation_arc(Vec3::Y, Vec3::ONE.normalize()),
            scale: Vec3::splat(1.5),
        },
        ..default()
    });

    commands.spawn(PbrBundle {
        material: mat.clone(),
        mesh: meshes.add(Mesh::from(shape::Cube { size: 2.0 })),
        transform: Transform::from_xyz(0.0, 2.0, 0.0),
        ..default()
    });
}

pub fn ui(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UiState>,
    query: Query<(&TextureNodeImage)>,
) {

}
