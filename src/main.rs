#![feature(associated_type_defaults)]
#![feature(lazy_cell)]

use bevy::prelude::*;
use bevy::render::camera::CameraRenderGraph;
use bevy::render::render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};
use bevy::utils::hashbrown::HashMap;
use bevy_egui::EguiPlugin;
use bevy_prototype_lyon::plugin::ShapePlugin;

use crate::param::ParamPlugin;
use crate::render::RenderPlugin;
use crate::texture::operator::composite::TextureOpComposite;
use crate::texture::operator::ramp::{TextureOpRamp, TextureRampSettings};
use crate::texture::{TextureOp, TextureOpBundle, TextureOpImage, TextureOpInputs, TextureOpOutputs, TextureOpType, TexturePlugin};
use crate::texture::render::TextureOpSubGraph;
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
    commands.spawn((TextureOp, TextureOpType::<TextureOpRamp>::default()));
    commands.spawn((TextureOp, TextureOpType::<TextureOpRamp>::default()));
    commands.spawn((TextureOp, TextureOpType::<TextureOpComposite>::default()));
}
