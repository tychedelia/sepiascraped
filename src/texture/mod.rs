use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};

pub mod ramp;

pub struct TexturePlugin;

impl Plugin for TexturePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<TextureNodeImage>::default(),
            ExtractComponentPlugin::<TextureNodeType>::default(),
            ramp::TextureRampPlugin,
        ));
    }
}

#[derive(Component)]
pub struct TextureNode;

#[derive(Component, Clone, ExtractComponent)]
pub struct TextureNodeType(pub String);

#[derive(Component, Clone, Deref, DerefMut, ExtractComponent)]
pub struct TextureNodeImage(pub Handle<Image>);

#[derive(Component)]
pub struct TextureNodeInputs {
    pub(crate) count: usize,
    pub(crate) connections: Vec<Entity>,
}

#[derive(Component)]
pub struct TextureNodeOutputs {
    pub(crate) count: usize,
    pub(crate) connections: Vec<Entity>,
}

#[derive(Bundle)]
pub struct TextureNodeBundle {
    pub camera: Camera2dBundle,
    pub node: TextureNode,
    pub node_type: TextureNodeType,
    pub image: TextureNodeImage,
    pub inputs: TextureNodeInputs,
    pub outputs: TextureNodeOutputs,
}
