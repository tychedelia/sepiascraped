use std::collections::BTreeMap;
use std::ops::DerefMut;
use bevy::ecs::system::SystemParam;

use bevy::prelude::*;

use crate::index::{CompositeIndex2, CompositeIndex2Plugin};
use crate::op::{OpCategory, OpRef};
use crate::script::update;
use crate::Sets;

pub struct ParamPlugin;

impl Plugin for ParamPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, validate.after(update).in_set(Sets::Params))
            .add_plugins(CompositeIndex2Plugin::<OpRef, ParamName>::new());
    }
}

#[derive(Bundle, Default, Clone)]
pub struct ParamBundle {
    pub name: ParamName,
    pub value: ParamValue,
    pub order: ParamOrder,
    pub page: ParamPage,
}

#[derive(Component, Clone, Default, Debug)]
pub struct ParamPage(pub String);
#[derive(Component, Clone, Default, Debug, Eq, Ord, PartialOrd, PartialEq)]
pub struct ParamOrder(pub u32);
#[derive(Component, Clone, Default, Debug)]
pub struct Param;
#[derive(
    Component, Deref, DerefMut, Default, Clone, PartialEq, Eq, Hash, Debug, Ord, PartialOrd,
)]
pub struct ParamName(pub String);

trait ParamType: Default {

}

#[derive(Component, Clone, Debug, Default)]
pub enum ParamValue {
    #[default]
    None,
    F32(f32),
    U32(u32),
    Vec2(Vec2),
    Vec3(Vec3),
    Quat(Quat),
    Color(Vec4),
    Bool(bool),
    TextureOp(Option<Entity>),
    MeshOp(Option<Entity>),
}

#[derive(Component, Default, Debug)]
pub struct ParamMin(ParamValue);

#[derive(Component, Default, Debug)]
pub struct ParamMax(ParamValue);

#[derive(Component, Default, Debug)]
pub struct ParamStep(pub ParamValue);

#[derive(Resource, Default, Debug)]
pub struct ParamHash(BTreeMap<Entity, u64>);

#[derive(Component, Default, Debug)]
pub struct ScriptedParam;
#[derive(Component, Default, Debug)]
pub struct ScriptedParamError(pub String);

fn validate(
    mut commands: Commands,
    mut params_q: Query<(Entity, &mut ParamValue), Changed<ParamValue>>,
    category_q: Query<&OpCategory>,
) {
    for (entity, mut param_value) in params_q.iter_mut() {
        match param_value.deref_mut() {
            ParamValue::TextureOp(Some(e)) => {
                let category = category_q.get(*e).unwrap();
                if !category.is_texture() {
                    commands
                        .entity(entity)
                        .insert(ScriptedParamError("Invalid texture".to_string()));
                    *param_value = ParamValue::TextureOp(None);
                }
            }
            ParamValue::MeshOp(Some(e)) => {
                let category = category_q.get(*e).unwrap();
                if !category.is_mesh() {
                    commands
                        .entity(entity)
                        .insert(ScriptedParamError("Invalid mesh".to_string()));
                    *param_value = ParamValue::MeshOp(None);
                }
            }
            _ => {}
        }
    }
}

#[derive(SystemParam)]
pub struct Params<'w, 's> {
    params_q: Query<'w, 's, &'static mut ParamValue>,
    param_idx: Res<'w, CompositeIndex2<OpRef, ParamName>>,
}

impl <'w, 's> Params<'w, 's> {
    pub fn get(&self, entity: Entity, name: impl Into<String>) -> Option<&ParamValue> {
        self.param_idx
            .get(&(OpRef(entity), ParamName(name.into())))
            .map(|e| self.params_q.get(*e).unwrap())
    }

    pub fn get_mut(&mut self, entity: Entity, name: impl Into<String>) -> Option<Mut<ParamValue>> {
        self.param_idx
            .get(&(OpRef(entity), ParamName(name.into())))
            .map(|e| self.params_q.get_mut(*e).unwrap())
    }
}

pub trait IntoParams {
    fn as_params(&self) -> Vec<ParamBundle>;
}

/// IntoParams for Transform
impl IntoParams for Transform {
    fn as_params(&self) -> Vec<ParamBundle> {
        vec![
            ParamBundle {
                name: ParamName("Translation".to_string()),
                value: ParamValue::Vec3(self.translation),
                order: ParamOrder(0),
                ..default()
            },
            ParamBundle {
                name: ParamName("Rotation".to_string()),
                value: ParamValue::Quat(self.rotation),
                order: ParamOrder(1),
                ..default()
            },
            ParamBundle {
                name: ParamName("Scale".to_string()),
                value: ParamValue::Vec3(self.scale),
                order: ParamOrder(2),
                ..default()
            },
        ]
    }
}
