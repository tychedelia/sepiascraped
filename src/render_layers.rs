pub use bevy::prelude::*;
use bevy::render::view::RenderLayers;

pub struct RenderLayerPlugin;

impl Plugin for RenderLayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RenderLayerManager>()
            .add_systems(First, sync);
    }
}

#[derive(Resource)]
pub struct RenderLayerManager {
    layers: Vec<bool>,
}

impl Default for RenderLayerManager {
    fn default() -> Self {
        Self {
            layers: vec![],
        }
    }
}

impl RenderLayerManager {
    fn add(&mut self, layer: usize) {
        if layer >= self.layers.len() {
            self.layers.resize_with(layer + 1, || false);
        }

        self.layers[layer] = true;
    }

    fn clear(&mut self) {
        self.layers.iter_mut().for_each(|layer| *layer = false);
    }

    pub fn next_open_layer(&mut self) -> usize {
        let layer = self
            .layers
            .iter()
            .position(|layer| !*layer)
            .map(|layer| layer)
            .unwrap_or(self.layers.len());

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
            render_layer_manager.add(layer.0);
        }
    }
}
