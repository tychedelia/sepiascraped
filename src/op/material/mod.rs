use std::fmt::Debug;

use crate::op::{OpImage, OpInputs, OpOutputs};
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_resource::encase::internal::WriteInto;
use bevy::render::render_resource::ShaderType;
use bevy::render::view::RenderLayers;
use types::standard::MaterialOpStandardPlugin;

use crate::op::texture::TextureOp;

pub mod types;

pub struct MaterialPlugin;

impl Plugin for MaterialPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(MaterialOpStandardPlugin)
            .add_systems(Startup, setup);
    }
}

#[derive(Component, Deref, DerefMut, Clone, Debug)]
pub struct MaterialDefaultMesh(Handle<Mesh>);

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.spawn((MaterialDefaultMesh(meshes.add(Mesh::from(Torus::default())))));
}

#[derive(Component, Deref, DerefMut, Clone, Debug)]
pub struct MaterialOpHandle<M: Asset>(pub Handle<M>);

#[derive(Bundle)]
pub struct MaterialOpBundle<M: Asset> {
    material: MaterialOpHandle<M>,
    image: OpImage,
    inputs: OpInputs,
    outputs: OpOutputs,
}
