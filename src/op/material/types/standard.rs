use std::ops::Deref;
use bevy::ecs::system::lifetimeless::*;
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::render::view::RenderLayers;
use bevy::utils::HashMap;

use crate::op::material::{MaterialOpBundle, MaterialOpHandle};
use crate::op::{Op, OpImage, OpInputs, OpOutputs, OpPlugin, OpType};
use crate::param::{ParamBundle, ParamName, ParamOrder, ParamValue};

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
    type OpType = OpType<MaterialOpStandard>;
    type UpdateParam = (
        SResMut<Assets<StandardMaterial>>,
        SQuery<Read<OpImage>>,
        SQuery<(Read<Children>, Read<MaterialOpHandle<StandardMaterial>>)>,
        SQuery<(Read<ParamName>, Read<ParamValue>)>,
    );
    type BundleParam = (
        SCommands,
        SResMut<Assets<StandardMaterial>>,
        SResMut<Assets<Image>>,
        SQuery<Read<RenderLayers>, With<Camera>>,
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
        (commands, materials, images, render_layer_q): &mut SystemParamItem<
            'w,
            '_,
            Self::BundleParam,
        >,
    ) -> Self::Bundle {
        let max = render_layer_q
            .iter()
            .map(|layer| layer.clone())
            .max()
            .unwrap_or(RenderLayers::layer(1));
        let new_layer = max.bits() + 1;
        if new_layer > 32 {
            panic!("Too many layers");
        }

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

        commands.spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(0.0, 0.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
                camera: Camera {
                    target: RenderTarget::Image(image.clone()),
                    ..default()
                },
                ..default()
            },
            RenderLayers::layer(new_layer as u8),
        ));

        (
            MaterialOpBundle {
                material: MaterialOpHandle(materials.add(StandardMaterial::default())),
                image: OpImage(image),
                inputs: OpInputs {
                    count: Self::INPUTS,
                    connections: HashMap::new(),
                },
                outputs: OpOutputs {
                    count: Self::OUTPUTS,
                },
            },
            RenderLayers::layer(new_layer as u8),
        )
    }

    fn params() -> Vec<ParamBundle> {
        vec![ParamBundle {
            name: ParamName("Texture".to_string()),
            value: ParamValue::TextureOp(None),
            order: ParamOrder(0),
            ..default()
        }]
    }
}
