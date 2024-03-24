use bevy::ecs::system::lifetimeless::*;
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::view::RenderLayers;
use bevy::window::WindowRef;

use crate::op::{Op, OpPlugin, OpType};
use crate::op::mesh::{MeshOpBundle, MeshOpHandle, MeshOpImage};
use crate::param::ParamBundle;

#[derive(Default)]
pub struct MeshOpCuboidPlugin;

impl Plugin for MeshOpCuboidPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(OpPlugin::<MeshOpCuboid>::default());
    }
}

#[derive(Component, Clone, Default, Debug)]
pub struct MeshOpCuboid;

impl Op for MeshOpCuboid {
    type OpType = OpType<MeshOpCuboid>;
    type UpdateParam = (
        SCommands,
    );
    type BundleParam = (SResMut<Assets<Mesh>>, SResMut<Assets<Image>>, SQuery<Read<RenderLayers>>);
    type Bundle = (MeshOpBundle, RenderLayers);

    fn update<'w>(entity: Entity, param: &mut SystemParamItem<'w, '_, Self::UpdateParam>) {

    }

    fn create_bundle<'w>(
        entity: Entity,
        (meshes, images, render_layer_q): &mut SystemParamItem<'w, '_, Self::BundleParam>,
    ) -> Self::Bundle {
        let max = render_layer_q.iter().map(|layer| layer.clone()).max().unwrap_or(RenderLayers::layer(1));
        let new_layer = max.bits() + 1;
        if new_layer > 32 {
            panic!("Too many layers");
        }

        let mesh = meshes.add(Mesh::from(Cuboid::default()));

        let size = bevy::render::render_resource::Extent3d {
            width: 512,
            height: 512,
            ..default()
        };

        let mut image = bevy::prelude::Image {
            texture_descriptor: bevy::render::render_resource::TextureDescriptor {
                label: None,
                size,
                dimension: bevy::render::render_resource::TextureDimension::D2,
                format: bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: bevy::render::render_resource::TextureUsages::TEXTURE_BINDING
                    | bevy::render::render_resource::TextureUsages::COPY_DST
                    | bevy::render::render_resource::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            ..default()
        };

        image.resize(size);

        let image = images.add(image);

        (
            MeshOpBundle {
                mesh: MeshOpHandle(mesh),
                camera: Camera3dBundle {
                    camera: Camera {
                        target: RenderTarget::Image(image.clone()),
                        ..default()
                    },
                    ..default()
                },
                image: MeshOpImage(image),
            },
            RenderLayers::layer(new_layer as u8),
        )
    }

    fn params() -> Vec<ParamBundle> {
        vec![
        ] 
    }
}
