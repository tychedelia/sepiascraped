use crate::index::CompositeIndex2Plugin;
use crate::ui::graph::OpRef;
use bevy::prelude::*;
use std::collections::BTreeMap;

pub struct ParamPlugin;

impl Plugin for ParamPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CompositeIndex2Plugin::<OpRef, ParamName>::new());
    }
}

trait FromParams {
    fn from_params(params: &Vec<Param>) -> Self;
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
#[derive(Component, Clone, Debug, Default)]
pub enum ParamValue {
    #[default]
    None,
    F32(f32),
    U32(u32),
    Vec2(Vec2),
    Color(Vec4),
    Bool(bool),
    TextureOp(Entity),
}

#[derive(Resource, Default, Debug)]
pub struct ParamHash(BTreeMap<Entity, u64>);

#[derive(Component, Default, Debug)]
pub struct ScriptedParam;
#[derive(Component, Default, Debug)]
pub struct ScriptedParamError(pub String);
