use bevy::prelude::*;
use bevy::render::camera::{CameraOutputMode, CameraRenderGraph};
use bevy::{
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
};
use bevy_egui::EguiPlugin;
use bevy_prototype_lyon::plugin::ShapePlugin;

use crate::texture::ramp::{TextureRampSettings, TextureRampSubGraph};
use crate::texture::{
    TextureNode, TextureNodeBundle, TextureNodeImage, TextureNodeInputs, TextureNodeOutputs,
    TextureNodeType, TexturePlugin,
};
use crate::ui::UiPlugin;

mod texture;
mod ui;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EguiPlugin,
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
        TextureNodeBundle {
            camera: Camera3dBundle {
                camera_render_graph: CameraRenderGraph::new(TextureRampSubGraph),
                camera: Camera {
                    output_mode: CameraOutputMode::Skip,
                    target: image_handle_1.clone().into(),
                    order: 1,
                    ..default()
                },
                ..default()
            },
            node: TextureNode,
            node_type: TextureNodeType("texture_ramp".into()),
            image: TextureNodeImage(image_handle_1.clone()),
            inputs: TextureNodeInputs {
                count: 0,
                connections: vec![],
            },
            outputs: TextureNodeOutputs {
                count: 0,
                connections: vec![],
            },
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

    // fill image.data with zeroes
    image.resize(size);

    let image_handle_2 = images.add(image);

    commands.spawn((
        TextureNodeBundle {
            camera: Camera3dBundle {
                camera_render_graph: CameraRenderGraph::new(TextureRampSubGraph),
                camera: Camera {
                    output_mode: CameraOutputMode::Skip,
                    order: 2,
                    target: image_handle_2.clone().into(),
                    ..default()
                },
                ..default()
            },
            node: TextureNode,
            node_type: TextureNodeType("texture_ramp".into()),
            image: TextureNodeImage(image_handle_2.clone()),
            inputs: TextureNodeInputs {
                count: 0,
                connections: vec![],
            },
            outputs: TextureNodeOutputs {
                count: 0,
                connections: vec![],
            },
        },
        TextureRampSettings {
            color_a: Vec4::new(1.0, 0.0, 0.0, 1.0),
            color_b: Vec4::new(0.0, 0.5, 1.0, 1.0),
            mode: 2,
        },
    ));
}
