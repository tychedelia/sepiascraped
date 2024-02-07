use crate::ui::graph::{GraphNode, GraphPlugin, SelectedNode};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::texture::TextureNodeImage;

pub mod graph;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_plugins((GraphPlugin))
            .init_resource::<UiState>()
            // .add_systems(Update, ui)
        ;
    }
}

#[derive(Resource, Default)]
pub struct UiState {
    pub side_panel: Option<egui::Response>,
}

pub fn ui(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UiState>,
    query: Query<(&TextureNodeImage)>
) {
    // egui::CentralPanel::default().show(contexts.ctx(), |ui| {
    //     ui.heading("Texture Nodes");
    //     ui.separator();
    //     for (image) in query.iter() {
    //         let image = contexts.image_id(&image.0).unwrap();
    //         ui.image(egui::load::SizedTexture::new(
    //             image,
    //             egui::vec2(200.0, 200.0),
    //         ));
    //     }
    // });

}
