use crate::event::SpawnOp;
use crate::param::{ParamBundle, ParamName, ParamValue};
use crate::ui::graph::OpRef;
use crate::OpName;
use bevy::ecs::system::{ReadOnlySystemParam, StaticSystemParam, SystemParam, SystemParamItem};
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use std::fmt::Debug;
use std::marker::PhantomData;

pub mod component;
pub mod mesh;
pub mod texture;

#[derive(Component, Clone, ExtractComponent, Default, Debug)]
pub struct OpType<T: Debug + Sync + Send + 'static>(PhantomData<T>);

impl<T> OpType<T>
    where
        T: Debug + Sync + Send + 'static,
{
    pub fn name() -> &'static str {
        std::any::type_name::<T>().split("::").nth(3).unwrap()
    }
}

pub trait Op {
    const INPUTS: usize = 0;
    const OUTPUTS: usize = 0;

    /// The type of the op.
    type OpType: Debug + Component + ExtractComponent + Send + Sync + 'static;
    /// The update parameter.
    type UpdateParam: SystemParam + 'static;
    /// The bundle parameter.
    type BundleParam: SystemParam + 'static;
    /// The bundle type of this op;
    type Bundle: Bundle;


    /// Update the op, i.e. to apply updates from the UI.
    fn update<'w>(entity: Entity, param: &mut SystemParamItem<'w, '_, Self::UpdateParam>);

    /// Create the op bundle.
    fn create_bundle<'w>(
        entity: Entity,
        param: &mut SystemParamItem<'w, '_, Self::BundleParam>,
    ) -> Self::Bundle;

    /// Get the parameters for the op.
    fn params() -> Vec<ParamBundle>;
}

#[derive(Default)]
pub struct OpPlugin<T: Op> {
    _marker: std::marker::PhantomData<T>,
}

impl<T> Plugin for OpPlugin<T>
where
    T: Op + Component + Send + Sync + Debug + 'static,
{
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spawn::<T>, update::<T>));
    }
}

fn update<'w, 's, T: Op>(
    changed_q: Query<&Parent, Changed<ParamValue>>,
    param: StaticSystemParam<T::UpdateParam>,
) {
    let mut param = param.into_inner();
    for parent in changed_q.iter() {
        T::update(parent.get(), &mut param);
    }
}

fn spawn<'w, 's, T>(
    mut commands: Commands,
    param: StaticSystemParam<T::BundleParam>,
    added_q: Query<Entity, Added<OpType<T>>>,
    mut spawn_op_evt: EventWriter<SpawnOp>,
) where
    T: Op + Component + Debug + Send + Sync + 'static,
{
    let mut param = param.into_inner();

    for entity in added_q.iter() {
        let bundle = T::create_bundle(entity, &mut param);
        commands
            .entity(entity)
            .insert(bundle)
            .with_children(|parent| {
                T::params().into_iter().for_each(|param| {
                    parent.spawn((OpRef(parent.parent_entity()), param));
                });
            });

        spawn_op_evt.send(SpawnOp(entity));
    }
}
