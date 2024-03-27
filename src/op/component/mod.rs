use std::fmt::Debug;

use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_resource::encase::internal::WriteInto;
use bevy::render::render_resource::ShaderType;

use crate::op::component::types::window::ComponentOpWindowPlugin;
use crate::op::{Op, OpInputConfig, OpInputs, OpOutputConfig, OpOutputs};
use crate::op::texture::TextureOp;

pub mod types;

pub const CATEGORY : &str = "Component";

pub struct ComponentPlugin;

impl Plugin for ComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ComponentOpWindowPlugin);
    }
}

#[derive(Bundle)]
pub struct ComponentOpBundle<T>
where T: Op + Component + ExtractComponent + Debug + Send + Sync + 'static,
{
    inputs: OpInputs<T>,
    input_config: OpInputConfig,
    outputs: OpOutputs,
    output_config: OpOutputConfig,
}