use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use bevy_mod_picking::DefaultPickingPlugins;

use camera::CameraControllerPlugin;

use crate::ui::event::ClickNode;
use crate::ui::graph::GraphPlugin;
use crate::ui::grid::InfiniteGridPlugin;

mod camera;
pub mod event;
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
        .add_systems(Startup, ui_setup)
        .add_systems(Update, ui)
        .init_resource::<UiState>()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0,
        });
    }
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Resources
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[derive(Resource, Default)]
pub struct UiState {
    pub top_panel: Option<egui::Response>,
    pub side_panel: Option<egui::Response>,
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Systems
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

pub fn ui_setup(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_translation(Vec3::new(0.1, 0.1, 0.0)),
            ..default()
        },
        camera::CameraController::default(),
    ));
}

pub fn ui(mut ui_state: ResMut<UiState>, mut egui_contexts: EguiContexts) {
    let ctx = egui_contexts.ctx_mut();
    ui_state.top_panel = Some(
        egui::TopBottomPanel::top("top_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Top Panel");
            })
            .response,
    );
}
