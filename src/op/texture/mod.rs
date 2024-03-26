use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Deref;

use bevy::ecs::query::QueryData;
use bevy::ecs::system::{SystemId, SystemParamItem};
use bevy::ecs::system::lifetimeless::{Read, SQuery, SRes, SResMut, Write};
use bevy::prelude::*;
use bevy::render::camera::{CameraRenderGraph, RenderTarget};
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::render_resource::{
    Extent3d, ShaderType, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::render::render_resource::encase::internal::WriteInto;
use bevy::sprite::Material2d;
use bevy::utils::{HashMap, info};
use bevy_egui::{egui, EguiContexts};
use bevy_egui::egui::{Align, CollapsingHeader};

use types::composite::TextureOpCompositePlugin;
use types::ramp::TextureOpRampPlugin;

use crate::{OpName, Sets, ui};
use crate::event::SpawnOp;
use crate::index::{UniqueIndex, UniqueIndexPlugin};
use crate::op::{Op, OpInputs, OpOutputs, OpRef, OpType, OpDefaultImage, OpImage};
use crate::op::texture::render::TextureOpSubGraph;
use crate::op::texture::types::composite::{CompositeMode, CompositeSettings};
use crate::op::texture::types::noise::TextureOpNoisePlugin;
use crate::op::texture::types::ramp::{TextureRampMode, TextureRampSettings};
use crate::param::{
    ParamBundle, ParamName, ParamOrder, ParamPage, ParamValue, ScriptedParam, ScriptedParamError,
};
use crate::ui::event::{Connect, Disconnect};
use crate::ui::graph::{GraphRef, NodeMaterial, SelectedNode};
use crate::ui::UiState;

pub mod render;
pub mod types;

pub const CATEGORY: &str = "Texture";

pub struct TexturePlugin;

impl Plugin for TexturePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                ExtractComponentPlugin::<OpImage>::default(),
                ExtractComponentPlugin::<OpInputs>::default(),
                TextureOpRampPlugin,
                TextureOpCompositePlugin,
                TextureOpNoisePlugin,
            ))
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    update_materials,
                    connect_handler,
                    disconnect_handler,
                )
                    .in_set(Sets::Ui),
            )
            .add_systems(
                Last,
                update_op_cameras
            );
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
pub struct TextureOpBundle {
    pub camera: Camera3dBundle,
    pub image: OpImage,
    pub inputs: OpInputs,
    pub outputs: OpOutputs,
}

macro_rules! impl_op {
    ($name:ident, $inputs:expr, $outputs:expr) => {
        impl crate::op::Op for $name {
            const CATEGORY : &'static str = crate::op::texture::CATEGORY;
            const INPUTS: usize = $inputs;
            const OUTPUTS: usize = $outputs;

            type OpType = OpType<Self>;
            type UpdateParam = (
                bevy::ecs::system::lifetimeless::SQuery<(
                    bevy::ecs::system::lifetimeless::Read<bevy::prelude::Children>,
                    bevy::ecs::system::lifetimeless::Write<crate::op::OpImage>,
                    bevy::ecs::system::lifetimeless::Write<<$name as TextureOp>::Uniform>)>,
                bevy::ecs::system::lifetimeless::SQuery<(
                    bevy::ecs::system::lifetimeless::Read<ParamName>, bevy::ecs::system::lifetimeless::Read<ParamValue>
                )>,
                bevy::ecs::system::lifetimeless::SResMut<bevy::prelude::Assets<bevy::prelude::Image>>,
            );
            type BundleParam = (bevy::ecs::system::lifetimeless::SResMut<bevy::prelude::Assets<bevy::prelude::Image>>);
            type Bundle = (crate::op::texture::TextureOpBundle, <$name as TextureOp>::Uniform);

            fn update<'w>(entity: bevy::prelude::Entity, param: &mut bevy::ecs::system::SystemParamItem<'w, '_, Self::UpdateParam>) {
                let (self_q, params_q, ref mut images) = param;

                let (children, mut image, mut uniform) = self_q.get_mut(entity).expect("Expected update entity to exist in self_q");

                let params = children
                    .iter()
                    .filter_map(|entity| params_q.get(*entity).ok())
                    .collect();

                <$name as TextureOp>::update_uniform(&mut uniform, &params);

                let resolution = params.iter().find(|(name, _)| *name == &crate::param::ParamName("Resolution".to_string())).unwrap().1;
                if let crate::param::ParamValue::Vec2(resolution) = resolution {
                    let image_size = images.get(image.0.clone()).unwrap().size();
                    if image_size.x != resolution.x as u32 || image_size.y != resolution.y as u32 {
                        let mut new_image = crate::op::OpImage::new_image(resolution.x as u32, resolution.y as u32);
                        let new_image = images.add(new_image);
                        *image = crate::op::OpImage(new_image);
                    }
                }
            }

            fn create_bundle<'w>(entity: bevy::prelude::Entity, (mut images): &mut bevy::ecs::system::SystemParamItem<'w, '_, Self::BundleParam>) -> Self::Bundle {
                let image = images.add(crate::op::OpImage::new_image(512, 512));
                (
                    crate::op::texture::TextureOpBundle {
                        camera: bevy::prelude::Camera3dBundle {
                            camera_render_graph: bevy::render::camera::CameraRenderGraph::new(crate::op::texture::TextureOpSubGraph),
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
                        outputs: crate::op::OpOutputs { count: $outputs },
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

                [common_params, <$name as TextureOp>::params()]
                    .concat()
            }
        }
    }
}

pub(crate) use impl_op;

fn connect_handler(
    mut ev_connect: EventReader<Connect>,
    mut op_q: Query<(&mut OpInputs, &OpImage, &GraphRef)>,
    input_q: Query<&OpImage>,
) {
    for ev in ev_connect.read() {
        if let Ok((mut input, my_image, graph_ref)) = op_q.get_mut(ev.input) {
            if let Ok(image) = input_q.get(ev.output) {
                input.connections.insert(ev.output, image.0.clone());
            }
        }
    }
}

pub fn update_op_cameras(
    mut op_q: Query<(&mut Camera, &mut OpImage), Changed<OpImage>>,
) {
    for (mut camera, mut image) in op_q.iter_mut() {
        camera.target = RenderTarget::Image(image.0.clone());
    }
}

fn update_materials(
    mut materials: ResMut<Assets<NodeMaterial>>,
    mut op_q: Query<(&OpInputs, &OpImage, &GraphRef)>,
    mut material_q: Query<&Handle<NodeMaterial>>,
) {
    // TODO: add component to test for connected rather than constantly doing this
    for (input, my_image, graph_ref) in op_q.iter_mut() {
        if input.is_fully_connected() {
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

fn disconnect_handler(
    mut ev_disconnect: EventReader<Disconnect>,
    mut op_q: Query<&mut OpInputs>,
    input_q: Query<&OpImage>,
) {
    for ev in ev_disconnect.read() {
        if let Ok(mut input) = op_q.get_mut(ev.input) {
            if let Ok(image) = input_q.get(ev.output) {
                input.connections.remove(&ev.output);
            }
        }
    }
}

pub trait TextureOp: Op {
    const SHADER: &'static str;
    type Uniform: Component + ExtractComponent + ShaderType + WriteInto + Clone + Default;

    fn params() -> Vec<ParamBundle>;

    fn update_uniform(uniform: &mut Self::Uniform, params: &Vec<(&ParamName, &ParamValue)>);
}
