use crate::ui::graph::{GraphNode, GraphPlugin, SelectedNode};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

pub mod graph;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((GraphPlugin))
            .init_resource::<UiState>()
            .add_systems(Update, ui);
    }
}

#[derive(Resource, Default)]
pub struct UiState {
    pub side_panel: Option<egui::Response>,
}

pub fn ui(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UiState>,
) {

}
