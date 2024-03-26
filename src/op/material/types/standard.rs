use bevy::ecs::system::lifetimeless::*;
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::render::view::RenderLayers;
use bevy::utils::HashMap;
use std::ops::Deref;

use crate::op::material::{CATEGORY, MaterialDefaultMesh, MaterialOpBundle, MaterialOpHandle};
use crate::op::{Op, OpImage, OpInputs, OpOutputs, OpPlugin, OpType};
use crate::param::{ParamBundle, ParamName, ParamOrder, ParamValue};
use crate::render_layers::RenderLayerManager;

#[derive(Default)]
pub struct MaterialOpStandardPlugin;

impl Plugin for MaterialOpStandardPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(OpPlugin::<MaterialOpStandard>::default());
    }
}

#[derive(Component, Clone, Default, Debug)]
pub struct MaterialOpStandard;

impl Op for MaterialOpStandard {
    const CATEGORY: &'static str = CATEGORY;
    type OpType = OpType<MaterialOpStandard>;
    type UpdateParam = (
        SResMut<Assets<StandardMaterial>>,
        SQuery<Read<OpImage>>,
        SQuery<(Read<Children>, Read<MaterialOpHandle<StandardMaterial>>)>,
        SQuery<(Read<ParamName>, Read<ParamValue>)>,
    );
    type BundleParam = (
        SCommands,
        SRes<MaterialDefaultMesh>,
        SResMut<Assets<StandardMaterial>>,
        SResMut<Assets<Image>>,
        SResMut<RenderLayerManager>,
    );
    type Bundle = (MaterialOpBundle<StandardMaterial>, RenderLayers);

    fn update<'w>(entity: Entity, param: &mut SystemParamItem<'w, '_, Self::UpdateParam>) {
        let (materials, image_q, self_q, params_q) = param;

        let (children, handle) = self_q
            .get_mut(entity)
            .expect("Expected update entity to exist in self_q");

        for (param_name, param_value) in params_q.iter_many(children) {
            match param_name.0.as_str() {
                "Texture" => {
                    if let ParamValue::TextureOp(Some(texture_entity)) = param_value {
                        let material = materials.get_mut(&**handle).unwrap();
                        let texture = image_q.get(*texture_entity).unwrap();
                        material.base_color_texture = Some(texture.deref().clone());
                    }
                }
                _ => {}
            }
        }
    }

    fn create_bundle<'w>(
        entity: Entity,
        (commands, default_mesh, materials, images, layer_manager): &mut SystemParamItem<
            'w,
            '_,
            Self::BundleParam,
        >,
    ) -> Self::Bundle {
        let image = OpImage::new_image(512, 512);
        let image = images.add(image);

        let new_layer = layer_manager.next_open_layer();

        commands.spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(0.0, 1.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
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

        let material = materials.add(StandardMaterial::default());

        commands.spawn((
            PbrBundle {
                mesh: default_mesh.0.clone(),
                material: material.clone(),
                ..default()
            },
            RenderLayers::layer(new_layer),
        ));

        (
            MaterialOpBundle {
                material: MaterialOpHandle(material),
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

    fn params(bundle: &Self::Bundle) -> Vec<ParamBundle> {
        vec![ParamBundle {
            name: ParamName("Texture".to_string()),
            value: ParamValue::TextureOp(None),
            order: ParamOrder(0),
            ..default()
        }]
    }
}
