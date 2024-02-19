#![feature(associated_type_defaults)]
#![feature(lazy_cell)]

use crate::render::RenderPlugin;
use crate::script::ScriptPlugin;
use bevy::app::{AppExit, MainSchedulePlugin};
use bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell;
use bevy::prelude::*;
use bevy::render::camera::{CameraOutputMode, CameraRenderGraph};
use bevy::utils::hashbrown::HashMap;
use bevy::{
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
};
use bevy_egui::EguiPlugin;
use bevy_prototype_lyon::plugin::ShapePlugin;
use std::cell::OnceCell;
use std::ptr;

use crate::texture::operator::composite::{CompositePlugin, CompositeSettings};
use crate::texture::operator::ramp::{TextureRampPlugin, TextureRampSettings};
use crate::texture::render::TextureOpRenderNode;
use crate::texture::{
    TextureOp, TextureOpBundle, TextureOpImage, TextureOpInputs, TextureOpOutputs, TextureOpType,
    TexturePlugin,
};
use crate::ui::UiPlugin;
mod render;
mod script;
mod texture;
mod ui;

fn main() {
    App::new()
        .add_plugins((
        ScriptPlugin,
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
    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };

    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // fill image.data with zeroes
    image.resize(size);

    let image_handle_1 = images.add(image);

    commands.spawn((
        TextureOpBundle {
            camera: Camera3dBundle {
                camera_render_graph: CameraRenderGraph::new(TextureRampPlugin::render_sub_graph()),
                camera: Camera {
                    target: image_handle_1.clone().into(),
                    order: 1,
                    ..default()
                },
                ..default()
            },
            op: TextureOp,
            op_type: TextureOpType("texture_ramp".into()),
            image: TextureOpImage(image_handle_1.clone()),
            inputs: TextureOpInputs {
                count: 0,
                connections: HashMap::new(),
            },
            outputs: TextureOpOutputs { count: 1 },
        },
        TextureRampSettings {
            color_a: Vec4::new(1.0, 0.0, 0.0, 1.0),
            color_b: Vec4::new(0.0, 0.0, 1.0, 1.0),
            mode: 0,
        },
    ));

    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    image.resize(size);

    let image_handle_2 = images.add(image);

    commands.spawn((
        TextureOpBundle {
            camera: Camera3dBundle {
                camera_render_graph: CameraRenderGraph::new(TextureRampPlugin::render_sub_graph()),
                camera: Camera {
                    order: 2,
                    target: image_handle_2.clone().into(),
                    ..default()
                },
                ..default()
            },
            op: TextureOp,
            op_type: TextureOpType("texture_ramp".into()),
            image: TextureOpImage(image_handle_2.clone()),
            inputs: TextureOpInputs {
                count: 0,
                connections: HashMap::new(),
            },
            outputs: TextureOpOutputs { count: 1 },
        },
        TextureRampSettings {
            color_a: Vec4::new(1.0, 0.0, 0.0, 1.0),
            color_b: Vec4::new(0.0, 0.5, 1.0, 1.0),
            mode: 2,
        },
    ));

    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    image.resize(size);

    let image_handle_3 = images.add(image);

    commands.spawn((
        TextureOpBundle {
            camera: Camera3dBundle {
                camera_render_graph: CameraRenderGraph::new(CompositePlugin::render_sub_graph()),
                camera: Camera {
                    order: 3,
                    target: image_handle_3.clone().into(),
                    ..default()
                },
                ..default()
            },
            op: TextureOp,
            op_type: TextureOpType("composite".into()),
            image: TextureOpImage(image_handle_3.clone()),
            inputs: TextureOpInputs {
                count: 2,
                connections: HashMap::new(),
            },
            outputs: TextureOpOutputs { count: 0 },
        },
        CompositeSettings { mode: 0 },
    ));
}
