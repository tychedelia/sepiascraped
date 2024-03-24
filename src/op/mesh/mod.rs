use std::fmt::Debug;

use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_resource::encase::internal::WriteInto;
use bevy::render::render_resource::ShaderType;

use crate::op::texture::TextureOp;

pub mod types;

pub struct MeshPlugin;

impl Plugin for MeshPlugin {
    fn build(&self, app: &mut App) {
        app;
    }
}

#[derive(Component, Deref, DerefMut, Copy, Clone, Debug)]
pub struct MeshOpHandle(pub Handle<Mesh>);

#[derive(Component, Clone, Default, Debug)]
pub struct MeshOpImage(pub Handle<Image>);

#[derive(Bundle)]
pub struct MeshOpBundle {
    mesh: MeshOpHandle,
    camera: Camera3dBundle,
    image: MeshOpImage,
}