use std::f32::consts::PI;
use bevy::ecs::system::lifetimeless::*;
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::view::RenderLayers;
use bevy::utils::HashMap;

use crate::op::mesh::{CATEGORY, MeshOpBundle, MeshOpHandle};
use crate::op::{Op, OpImage, OpInputs, OpOutputs, OpPlugin, OpType};
use crate::param::ParamBundle;
use crate::render_layers::RenderLayerManager;

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
    const CATEGORY: &'static str = CATEGORY;
    type OpType = OpType<MeshOpCuboid>;
    type UpdateParam = (SCommands,);
    type BundleParam = (
        SCommands,
        SResMut<Assets<Mesh>>,
        SResMut<Assets<Image>>,
        SResMut<Assets<StandardMaterial>>,
        SResMut<RenderLayerManager>
    );
    type Bundle = (MeshOpBundle, RenderLayers);

    fn update<'w>(entity: Entity, param: &mut SystemParamItem<'w, '_, Self::UpdateParam>) {}

    fn create_bundle<'w>(
        entity: Entity,
        (commands, meshes, images, materials, layer_manager): &mut SystemParamItem<
            'w,
            '_,
            Self::BundleParam,
        >,
    ) -> Self::Bundle {
        let mesh = meshes.add(Mesh::from(Cuboid::default()));

        let size = Extent3d {
            width: 512,
            height: 512,
            ..default()
        };

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

        let image = images.add(image);

        let new_layer = layer_manager.next_open_layer();

        commands.spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(0.0, 0.0, 4.0)
                    .looking_at(Vec3::ZERO, Vec3::Y),
                camera: Camera {
                    target: RenderTarget::Image(image.clone()),
                    ..default()
                },
                ..default()
            },
            RenderLayers::layer(new_layer),
        ));

        commands.spawn((
            PointLightBundle {
                point_light: PointLight {
                    shadows_enabled: true,
                    intensity: 10_000_000.,
                    range: 100.0,
                    ..default()
                },
                transform: Transform::from_xyz(8.0, 16.0, 8.0),
                ..default()
            },
            RenderLayers::layer(new_layer),
        ));

        (
            MeshOpBundle {
                mesh: MeshOpHandle(mesh.clone()),
                pbr: PbrBundle {
                    mesh,
                    material: materials.add(Color::GRAY),
                    transform: Transform::from_xyz(0.0, 0.0, 0.0)
                        .with_rotation(Quat::from_rotation_x(-PI / 4.0)),
                    ..default()
                },
                image: OpImage(image),
                inputs: OpInputs {
                    count: Self::INPUTS,
                    connections: HashMap::new(),
                },
                outputs: OpOutputs {
                    count: Self::OUTPUTS,
                },
            },
            RenderLayers::layer(new_layer),
        )
    }

    fn params() -> Vec<ParamBundle> {
        vec![]
    }
}
