use std::f32::consts::PI;
use std::ops::DerefMut;
use bevy::ecs::system::lifetimeless::*;
use bevy::ecs::system::{StaticSystemParam, SystemParamItem};
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::view::RenderLayers;
use bevy::utils::HashMap;

use crate::op::mesh::{CATEGORY, MeshOpBundle, MeshOpHandle};
use crate::op::{Op, OpImage, OpInputConfig, OpInputs, OpOutputConfig, OpOutputs, OpPlugin, OpType};
use crate::param::{IntoParams, ParamBundle, Params, ParamValue};
use crate::render_layers::RenderLayerManager;

#[derive(Default)]
pub struct MeshOpCuboidPlugin;

impl Plugin for MeshOpCuboidPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(OpPlugin::<MeshOpCuboid>::default());
    }
}

#[derive(Component, ExtractComponent, Clone, Default, Debug)]
pub struct MeshOpCuboid;

impl Op for MeshOpCuboid {
    const CATEGORY: &'static str = CATEGORY;
    type OpType = OpType<MeshOpCuboid>;
    type UpdateParam = (SQuery<Write<Transform>>, Params<'static, 'static>);
    type BundleParam = (
        SCommands,
        SResMut<Assets<Mesh>>,
        SResMut<Assets<Image>>,
        SResMut<Assets<StandardMaterial>>,
        SResMut<RenderLayerManager>
    );
    type OnConnectParam = ();
    type ConnectionDataParam = ();
    type OnDisconnectParam = ();
    type Bundle = (MeshOpBundle<Self>, RenderLayers);
    type ConnectionData = ();

    fn update<'w>(entity: Entity, param: &mut SystemParamItem<'w, '_, Self::UpdateParam>) {
        let (transform, params) = param;

        params.get_mut(entity, "Translation").map(|mut param| {
            if let ParamValue::Vec3(translation) = param.deref_mut() {
                transform.get_mut(entity).unwrap().translation = *translation;
            }
        });
        params.get_mut(entity, "Rotation").map(|mut param| {
            if let ParamValue::Quat(rotation) = param.deref_mut() {
                transform.get_mut(entity).unwrap().rotation = *rotation;
            }
        });
        params.get_mut(entity, "Scale").map(|mut param| {
            if let ParamValue::Vec3(scale) = param.deref_mut() {
                transform.get_mut(entity).unwrap().scale = *scale;
            }
        });
    }

    fn create_bundle<'w>(
        entity: Entity,
        (commands, meshes, images, materials, layer_manager): &mut SystemParamItem<
            'w,
            '_,
            Self::BundleParam,
        >,
    ) -> Self::Bundle {
        let mesh = meshes.add(Mesh::from(Cuboid::default()));
        let image = OpImage::new_image(512, 512);
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
                input_config: OpInputConfig {
                    count: Self::INPUTS,
                },
                outputs: OpOutputs {
                    count: Self::OUTPUTS,
                },
                output_config: OpOutputConfig {
                    count: Self::OUTPUTS,
                },
            },
            RenderLayers::layer(new_layer),
        )
    }

    fn params(bundle: &Self::Bundle) -> Vec<ParamBundle> {
        [vec![], bundle.0.pbr.transform.as_params()]
            .concat()
    }

    fn connection_data<'w>(entity: Entity, param: &mut SystemParamItem<'w, '_, Self::ConnectionDataParam>) -> Self::ConnectionData {
        todo!()
    }
}
