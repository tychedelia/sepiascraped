use bevy::{
    ecs::query::QueryData,
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    }
};
use bevy::prelude::*;
use bevy::render::camera::CameraRenderGraph;
use bevy_egui::{EguiPlugin, EguiUserTextures};

use crate::texture::{TextureNode, TextureNodeBundle, TextureNodeImage, TextureNodeInputs, TextureNodeOutputs, TextureNodeType, TexturePlugin};
use crate::texture::ramp::{TextureRampSettings, TextureRampSubGraph};
use crate::ui::UiPlugin;

mod texture;
mod ui;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, EguiPlugin, TexturePlugin, UiPlugin))
        .add_systems(Startup, setup)
        .run();
}

// Marks the first pass cube (rendered to a texture.)
#[derive(Component)]
struct FirstPassCube;

// Marks the main pass cube, to which the texture is applied.
#[derive(Component)]
struct MainPassCube;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut egui_user_textures: ResMut<EguiUserTextures>,
) {
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

    let image_handle = images.add(image);
    egui_user_textures.add_image(image_handle.clone());

    commands.spawn((
        TextureNodeBundle {
            camera: Camera3dBundle {
                camera_render_graph: CameraRenderGraph::new(TextureRampSubGraph),
                camera: Camera {
                    order: -1,
                    target: image_handle.clone().into(),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 10.0)),
                ..default()
            },
            node: TextureNode,
            node_type: TextureNodeType("texture_ramp".into()),
            image: TextureNodeImage(image_handle.clone()),
            inputs: TextureNodeInputs { count: 0, connections: vec![] },
            outputs: TextureNodeOutputs { count: 0, connections: vec![] },
        },
        TextureRampSettings {
            color_a: Vec4::new(1.0, 0.0, 0.0, 1.0), // Color::RED.into(
            color_b: Vec4::new(0.0, 0.0, 1.0, 1.0), // Color::GREEN.into(),
        },
    ));
}
