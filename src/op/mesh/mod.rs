use std::fmt::Debug;

use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_resource::encase::internal::WriteInto;
use bevy::render::render_resource::ShaderType;
use bevy::render::view::RenderLayers;
use crate::op::{OpImage, OpInputs, OpOutputs};
use crate::op::mesh::types::cuboid::MeshOpCuboidPlugin;

use crate::op::texture::TextureOp;

pub mod types;

pub struct MeshPlugin;

impl Plugin for MeshPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(MeshOpCuboidPlugin)
            .add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            intensity: 10_000_000.,
            range: 100.0,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    },
    RenderLayers::all()
    ));
}

#[derive(Component, Deref, DerefMut, Clone, Debug)]
pub struct MeshOpHandle(pub Handle<Mesh>);

#[derive(Bundle)]
pub struct MeshOpBundle {
    mesh: MeshOpHandle,
    pbr: PbrBundle,
    image: OpImage,
    inputs: OpInputs,
    outputs: OpOutputs,
}