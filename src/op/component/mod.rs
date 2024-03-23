use std::fmt::Debug;
use std::marker::PhantomData;

use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_resource::encase::internal::WriteInto;
use bevy::render::render_resource::ShaderType;

use crate::op::component::types::window::ComponentOpWindowPlugin;
use crate::op::texture::TextureOp;

pub mod types;

pub struct ComponentPlugin;

impl Plugin for ComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ComponentOpWindowPlugin);
    }
}

#[derive(Component, Clone, ExtractComponent, Default, Debug)]
pub struct ComponentOpType<T: Debug + Sync + Send + 'static>(PhantomData<T>);

impl<T> ComponentOpType<T>
where
    T: Debug + Sync + Send + 'static,
{
    pub fn name() -> &'static str {
        std::any::type_name::<T>().split("::").nth(3).unwrap()
    }
}

#[derive(Bundle)]
pub struct ComponentOpBundle {
}