pub use bevy::prelude::*;
use bevy::render::view::RenderLayers;

pub struct RenderLayerPlugin;

impl Plugin for RenderLayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<RenderLayerManager>()
            .add_systems(First, sync);

    }
}

#[derive(Resource)]
pub struct RenderLayerManager {
    layers: Vec<bool>,
}

impl Default for RenderLayerManager {
    fn default() -> Self {
        Self { layers: vec![false; RenderLayers::TOTAL_LAYERS] }
    }
}

impl RenderLayerManager {
    fn add(&mut self, layer: u8) {
        self.layers[layer as usize] = true;
    }

    fn clear(&mut self) {
        self.layers.iter_mut().for_each(|layer| *layer = false);
    }

    pub(crate) fn next_open_layer(&mut self) -> u8 {
        let layer = self.layers.iter().position(|layer| !*layer).map(|layer| layer as u8);
        let Some(layer) = layer else {
            panic!("No more layers available")
        };

        self.add(layer);

        layer
    }
}

fn sync(
    render_layers_q: Query<&RenderLayers, With<Camera>>,
    mut render_layer_manager: ResMut<RenderLayerManager>,
) {
    render_layer_manager.clear();
    for layer in render_layers_q.iter() {
        for layer in layer.iter() {
            render_layer_manager.add(layer);
        }
    }
}