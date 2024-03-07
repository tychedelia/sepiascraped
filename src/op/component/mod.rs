use crate::event::SpawnOp;
use crate::op::component::types::window::ComponentOpWindowPlugin;
use crate::op::texture::{TextureOp, TextureOpImage, TextureOpMeta, TextureOpType};
use crate::param::{ParamBundle, ParamName, ParamOrder, ParamPage, ParamValue};
use crate::ui::graph::OpRef;
use crate::OpName;
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_resource::encase::internal::WriteInto;
use bevy::render::render_resource::ShaderType;
use std::fmt::Debug;
use std::marker::PhantomData;

pub mod types;

pub struct ComponentPlugin;

impl Plugin for ComponentPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ComponentOpWindowPlugin));
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct ComponentOp;

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
    pub op: ComponentOp,
}

pub fn spawn_component_op<T>(
    mut commands: Commands,
    added_q: Query<(Entity, &OpName), (With<ComponentOp>, Added<ComponentOpType<T>>)>,
    mut spawn_op_evt: EventWriter<SpawnOp>,
) where
    T: ComponentOpMeta + Debug + Send + Sync + 'static,
{
    for (entity, op_name) in added_q.iter() {
        info!("Spawning component op: {:?}", op_name);

        let bundle = T::bundle(entity, op_name, &mut commands);
        commands
            .entity(entity.clone())
            .insert((bundle, ComponentOpBundle { op: ComponentOp }))
            .with_children(|parent| {
                let common_params = vec![];

                [common_params, T::params()]
                    .concat()
                    .into_iter()
                    .for_each(|param| {
                        parent.spawn((OpRef(parent.parent_entity()), param));
                    });
            });

        spawn_op_evt.send(SpawnOp(entity));
    }
}

pub trait ComponentOpMeta: Debug + Clone + Send + Sync + 'static {
    type OpType: Debug + Component + ExtractComponent + Send + Sync + 'static;

    fn bundle(entity: Entity, op_name: &OpName, commands: &mut Commands) -> impl Bundle;

    fn params() -> Vec<ParamBundle>;
}
