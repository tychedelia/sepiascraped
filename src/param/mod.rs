use bevy::prelude::*;
use bevy::render::render_resource::ShaderType;

pub struct ParamPlugin;

impl Plugin for ParamPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, |mut commands: Commands| {
            commands.spawn(ParamValue(Vec4::ZERO));
        });
    }
}

pub trait ParamUi {
    fn ui(&mut self);
}

#[derive(Component)]
pub struct Param;
#[derive(Component)]
pub struct ParamType(String);
#[derive(Component)]
pub struct ParamValue<T: ShaderType>(pub T);
#[derive(Component)]
pub struct ScriptedParam;
#[derive(Component)]
pub struct ScriptedParamValue(pub String);
