use std::fmt::Debug;

use crate::op::{Op, OpImage, OpInputConfig, OpInputs, OpOutputConfig, OpOutputs};
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_resource::encase::internal::WriteInto;
use bevy::render::render_resource::ShaderType;
use bevy::render::view::RenderLayers;
use types::standard::MaterialOpStandardPlugin;

use crate::op::texture::TextureOp;

pub mod types;

pub const CATEGORY: &str = "Material";

pub struct MaterialPlugin;

impl Plugin for MaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialOpStandardPlugin)
            .init_resource::<MaterialDefaultMesh>()
            .add_systems(Startup, setup);
    }
}

#[derive(Resource, Deref, DerefMut, Default, Clone, Debug)]
pub struct MaterialDefaultMesh(Handle<Mesh>);

fn setup(mut default_mesh: ResMut<MaterialDefaultMesh>, mut meshes: ResMut<Assets<Mesh>>) {
    *default_mesh = MaterialDefaultMesh(meshes.add(Mesh::from(Torus::default())));
}

#[derive(Component, Deref, DerefMut, Clone, Debug)]
pub struct MaterialOpHandle<M: Asset>(pub Handle<M>);

#[derive(Bundle)]
pub struct MaterialOpBundle<M: Asset, T: Op>
where
    T: Op + Component + ExtractComponent + Debug + Send + Sync + 'static,
{
    material: MaterialOpHandle<M>,
    image: OpImage,
    inputs: OpInputs<T>,
    input_config: OpInputConfig,
    outputs: OpOutputs,
    output_config: OpOutputConfig,
}
