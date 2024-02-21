#![feature(associated_type_defaults)]
#![feature(lazy_cell)]

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_prototype_lyon::plugin::ShapePlugin;

use crate::param::ParamPlugin;
use crate::render::RenderPlugin;
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
            // ScriptPlugin,
            ParamPlugin,
            DefaultPlugins,
            EguiPlugin,
            RenderPlugin,
            TexturePlugin,
            UiPlugin,
            ShapePlugin,
        ))
        .add_systems(Startup, setup)
        .run();
}

// Marks the first pass cube (rendered to a texture.)
#[derive(Component)]
struct FirstPassCube;

// Marks the main pass cube, to which the texture is applied.
#[derive(Component)]
struct MainPassCube;

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {

}
