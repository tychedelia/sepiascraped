use std::fmt::Debug;

use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::primitives::Aabb;
use bevy::render::render_resource::encase::internal::WriteInto;
use bevy::render::render_resource::ShaderType;
use bevy::render::view::RenderLayers;
use crate::op::{Op, OpImage, OpInputConfig, OpInputs, OpOutputConfig, OpOutputs};
use crate::op::mesh::types::cuboid::MeshOpCuboidPlugin;

use crate::op::texture::TextureOp;

pub mod types;

pub const CATEGORY : &str = "Mesh";

pub struct MeshPlugin;

impl Plugin for MeshPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(MeshOpCuboidPlugin);
    }
}

#[derive(Component, Deref, DerefMut, Clone, Debug)]
pub struct MeshOpHandle(pub Handle<Mesh>);

#[derive(Bundle)]
pub struct MeshOpBundle<T: Op>
    where T: Op + Component + ExtractComponent + Debug + Send + Sync + 'static,
{
    mesh: MeshOpHandle,
    pbr: PbrBundle,
    image: OpImage,
    inputs: OpInputs<T>,
    input_config: OpInputConfig,
    outputs: OpOutputs,
    output_config: OpOutputConfig,
}