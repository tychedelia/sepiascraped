use std::fmt::Debug;
use std::ops::Deref;

use bevy::ecs::query::QueryData;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::render_resource::encase::internal::WriteInto;
use bevy::render::render_resource::{
    Extent3d, ShaderType, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::sprite::Material2d;

use types::composite::TextureOpCompositePlugin;
use types::ramp::TextureOpRampPlugin;

use crate::op::texture::types::noise::TextureOpNoisePlugin;
use crate::op::{Op, OpDefaultImage, OpImage, OpInputConfig, OpInputs, OpOutputConfig, OpOutputs};
use crate::param::{ParamBundle, ParamName, ParamValue};

pub mod render;
pub mod types;

pub const CATEGORY: &str = "Texture";

pub struct TexturePlugin;

impl Plugin for TexturePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<OpImage>::default(),
            TextureOpRampPlugin,
            TextureOpCompositePlugin,
            TextureOpNoisePlugin,
        ))
        .add_systems(Startup, setup)
        .add_systems(Last, update_op_cameras);
    }
}

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
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
    /// All black
    image
        .data
        .chunks_mut(4)
        .enumerate()
        .for_each(|(i, mut chunk)| {
            let width = (size.width / 4) as f32;
            let x = i % 512;
            let y = i / 512;
            let x = (x as f32 % width) / width;
            let y = (y as f32 % width) / width;
            let x = x < 0.5;
            let y = y < 0.5;

            if x == y {
                chunk.copy_from_slice(&[150, 150, 150, 255]);
            } else {
                chunk.copy_from_slice(&[50, 50, 50, 255]);
            }
        });

    let image = images.add(image);
    commands.insert_resource(OpDefaultImage(image));
}

#[derive(Bundle, Default)]
pub struct TextureOpBundle<T: Op>
where
    T: Op + Component + ExtractComponent + Debug + Send + Sync + 'static,
{
    pub camera: Camera3dBundle,
    pub image: OpImage,
    inputs: OpInputs<T>,
    input_config: OpInputConfig,
    outputs: OpOutputs,
    output_config: OpOutputConfig,
}

macro_rules! impl_op {
    ($name:ident, $inputs:expr, $outputs:expr) => {
        impl crate::op::Op for $name {
            const CATEGORY: &'static str = crate::op::texture::CATEGORY;
            const INPUTS: usize = $inputs;
            const OUTPUTS: usize = $outputs;

            type OpType = OpType<Self>;
            type UpdateParam = (
                bevy::ecs::system::lifetimeless::SQuery<(
                    bevy::ecs::system::lifetimeless::Read<bevy::prelude::Children>,
                    bevy::ecs::system::lifetimeless::Write<crate::op::OpImage>,
                    bevy::ecs::system::lifetimeless::Write<<$name as TextureOp>::Uniform>,
                )>,
                bevy::ecs::system::lifetimeless::SQuery<(
                    bevy::ecs::system::lifetimeless::Read<ParamName>,
                    bevy::ecs::system::lifetimeless::Read<ParamValue>,
                )>,
                bevy::ecs::system::lifetimeless::SResMut<
                    bevy::prelude::Assets<bevy::prelude::Image>,
                >,
            );
            type BundleParam = (bevy::ecs::system::lifetimeless::SResMut<
                bevy::prelude::Assets<bevy::prelude::Image>,
            >);
            type OnConnectParam = (
                bevy::ecs::system::lifetimeless::SResMut<
                    bevy::prelude::Assets<crate::ui::graph::NodeMaterial>,
                >,
                bevy::ecs::system::lifetimeless::SQuery<
                    (
                        bevy::ecs::system::lifetimeless::Read<crate::op::OpImage>,
                        bevy::ecs::system::lifetimeless::Read<crate::ui::graph::GraphRef>,
                    ),
                    bevy::prelude::With<crate::op::OpType<Self>>,
                >,
                bevy::ecs::system::lifetimeless::SQuery<
                    bevy::ecs::system::lifetimeless::Read<
                        bevy::prelude::Handle<crate::ui::graph::NodeMaterial>,
                    >,
                >,
            );
            type ConnectionDataParam = (bevy::ecs::system::lifetimeless::SQuery<
                bevy::ecs::system::lifetimeless::Read<crate::op::OpImage>,
            >);
            type OnDisconnectParam = ();
            type Bundle = (
                crate::op::texture::TextureOpBundle<Self>,
                <$name as TextureOp>::Uniform,
            );
            type ConnectionData = Handle<Image>;

            fn update<'w>(
                entity: bevy::prelude::Entity,
                param: &mut bevy::ecs::system::SystemParamItem<'w, '_, Self::UpdateParam>,
            ) {
                let (self_q, params_q, ref mut images) = param;

                let Ok((children, mut image, mut uniform)) = self_q.get_mut(entity) else {
                    return;
                };

                let params = children
                    .iter()
                    .filter_map(|entity| params_q.get(*entity).ok())
                    .collect();

                <$name as TextureOp>::update_uniform(&mut uniform, &params);

                let resolution = params
                    .iter()
                    .find(|(name, _)| *name == &crate::param::ParamName("Resolution".to_string()))
                    .unwrap()
                    .1;
                if let crate::param::ParamValue::Vec2(resolution) = resolution {
                    let image_size = images.get(image.0.clone()).unwrap().size();
                    if image_size.x != resolution.x as u32 || image_size.y != resolution.y as u32 {
                        let mut new_image =
                            crate::op::OpImage::new_image(resolution.x as u32, resolution.y as u32);
                        let new_image = images.add(new_image);
                        *image = crate::op::OpImage(new_image);
                    }
                }
            }

            fn create_bundle<'w>(
                entity: bevy::prelude::Entity,
                (mut images): &mut bevy::ecs::system::SystemParamItem<'w, '_, Self::BundleParam>,
            ) -> Self::Bundle {
                let image = images.add(crate::op::OpImage::new_image(512, 512));
                (
                    crate::op::texture::TextureOpBundle {
                        camera: bevy::prelude::Camera3dBundle {
                            camera_render_graph: bevy::render::camera::CameraRenderGraph::new(
                                crate::op::texture::render::TextureOpSubGraph,
                            ),
                            camera: bevy::prelude::Camera {
                                order: 3,
                                target: image.clone().into(),
                                ..default()
                            },
                            ..default()
                        },
                        image: crate::op::texture::OpImage(image.clone()),
                        inputs: crate::op::texture::OpInputs {
                            count: $inputs,
                            connections: bevy::utils::HashMap::new(),
                        },
                        input_config: crate::op::OpInputConfig { count: $inputs },
                        outputs: crate::op::OpOutputs { count: $outputs },
                        output_config: crate::op::OpOutputConfig { count: $outputs },
                    },
                    <$name as TextureOp>::Uniform::default(),
                )
            }

            fn params(bundle: &Self::Bundle) -> Vec<crate::param::ParamBundle> {
                let common_params = vec![
                    crate::param::ParamBundle {
                        name: crate::param::ParamName("Resolution".to_string()),
                        value: crate::param::ParamValue::Vec2(Vec2::new(512.0, 512.0)),
                        order: crate::param::ParamOrder(0),
                        page: crate::param::ParamPage("Common".to_string()),
                        ..default()
                    },
                    // crate::param::ParamBundle {
                    //     name: crate::param::ParamName("View".to_string()),
                    //     value: crate::param::ParamValue::Bool(false),
                    //     order: crate::param::ParamOrder(1),
                    //     page: crate::param::ParamPage("Common".to_string()),
                    //     ..default()
                    // },
                ];

                [common_params, <$name as TextureOp>::params()].concat()
            }

            fn connection_data<'w>(
                entity: bevy::prelude::Entity,
                param: &mut bevy::ecs::system::SystemParamItem<'w, '_, Self::ConnectionDataParam>,
            ) -> Self::ConnectionData {
                let (image_q) = param;
                let image = image_q.get(entity).unwrap();
                image.0.clone()
            }

            fn on_connect<'w>(
                entity: bevy::prelude::Entity,
                event: crate::ui::event::Connect,
                fully_connected: bool,
                param: &mut bevy::ecs::system::SystemParamItem<'w, '_, Self::OnConnectParam>,
            ) {
                let (ref mut materials, ref mut op_q, ref mut material_q) = param;
                for (my_image, graph_ref) in op_q.iter_mut() {
                    if fully_connected {
                        if let Ok(material) = material_q.get(graph_ref.0) {
                            let mut material = materials.get_mut(material).unwrap();
                            if material.texture != my_image.0 {
                                material.texture = my_image.0.clone();
                            }
                        } else {
                            warn!("No material found for {:?}", graph_ref);
                        }
                    }
                }
            }

            fn on_disconnect<'w>(
                entity: bevy::prelude::Entity,
                event: crate::ui::event::Disconnect,
                fully_connected: bool,
                param: &mut bevy::ecs::system::SystemParamItem<'w, '_, Self::OnDisconnectParam>,
            ) {
                ()
            }
        }
    };
}

pub(crate) use impl_op;

pub fn update_op_cameras(mut op_q: Query<(&mut Camera, &mut OpImage), Changed<OpImage>>) {
    for (mut camera, mut image) in op_q.iter_mut() {
        camera.target = RenderTarget::Image(image.0.clone());
    }
}

pub trait TextureOp: Op {
    const SHADER: &'static str;
    type Uniform: Component + ExtractComponent + ShaderType + WriteInto + Clone + Default;

    fn params() -> Vec<ParamBundle>;

    fn update_uniform(uniform: &mut Self::Uniform, params: &Vec<(&ParamName, &ParamValue)>);
}
