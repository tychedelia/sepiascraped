mod ui;

use std::any::Any;
use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;

pub struct ParamPlugin;

impl Plugin for ParamPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, |mut commands: Commands| {

        });
    }
}

pub trait IntoParams {
    fn into_params(self) -> Vec<ParamBundle>;
}

#[derive(Bundle, Default)]
pub struct ParamBundle {
    pub name: ParamName,
    pub param_type: ParamType,
    pub value: ParamValue,
    pub order: ParamOrder,
    pub page: ParamPage,
}

#[derive(Component, Default, Debug)]
pub struct ParamPage(pub String);
#[derive(Component, Default, Debug, PartialEq, Eq)]
pub struct ParamOrder(pub u32);
#[derive(Component, Default, Debug)]
pub struct Param;
#[derive(Component, Default, Debug)]
pub struct ParamName(pub(crate) String);
#[derive(Component, Default, Debug)]
pub struct ParamType(String);
#[derive(Component, Clone, Debug, Default)]
pub enum ParamValue {
    #[default]
    None,
    F32(f32),
    U32(u32),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
}
#[derive(Component, Default, Debug)]
pub struct ScriptedParam;
#[derive(Component, Default, Debug)]
pub struct ScriptedParamValue(pub String);