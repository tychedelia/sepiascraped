#![feature(associated_type_defaults)]
#![feature(lazy_cell)]

use crate::index::UniqueIndexPlugin;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_prototype_lyon::plugin::ShapePlugin;

use crate::param::ParamPlugin;
use crate::render::RenderPlugin;
use crate::script::ScriptPlugin;
use crate::texture::operator::composite::TextureOpComposite;
use crate::texture::operator::ramp::TextureOpRamp;
use crate::texture::{TextureOp, TextureOpType, TexturePlugin};
use crate::ui::UiPlugin;

mod index;
mod param;
mod render;
mod script;
mod texture;
mod ui;

fn main() {
    App::new()
        .add_plugins((
            ScriptPlugin,
            ParamPlugin,
            DefaultPlugins,
            EguiPlugin,
            RenderPlugin,
            TexturePlugin,
            UiPlugin,
            ShapePlugin,
            UniqueIndexPlugin::<OpName>::default(),
        ))
        .add_systems(Startup, setup)
        .run();
}

#[derive(Component, Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OpName(pub String);

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    commands.spawn((TextureOp, TextureOpType::<TextureOpRamp>::default()));
    commands.spawn((TextureOp, TextureOpType::<TextureOpRamp>::default()));
    commands.spawn((TextureOp, TextureOpType::<TextureOpRamp>::default()));
    commands.spawn((TextureOp, TextureOpType::<TextureOpComposite>::default()));
    commands.spawn((TextureOp, TextureOpType::<TextureOpComposite>::default()));
}
