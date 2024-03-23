use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Deref;

use bevy::ecs::query::QueryData;
use bevy::ecs::system::{SystemId, SystemParamItem};
use bevy::ecs::system::lifetimeless::{Read, SQuery, SRes, SResMut, Write};
use bevy::prelude::*;
use bevy::render::camera::CameraRenderGraph;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::render_resource::encase::internal::WriteInto;
use bevy::render::render_resource::{
    Extent3d, ShaderType, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::sprite::Material2d;
use bevy::utils::{HashMap, info};
use bevy_egui::egui::{Align, CollapsingHeader};
use bevy_egui::{egui, EguiContexts};

use types::composite::TextureOpCompositePlugin;
use types::ramp::TextureOpRampPlugin;

use crate::index::{UniqueIndex, UniqueIndexPlugin};
use crate::event::SpawnOp;
use crate::op::texture::render::TextureOpSubGraph;
use crate::op::texture::types::composite::{CompositeMode, CompositeSettings};
use crate::op::texture::types::noise::TextureOpNoisePlugin;
use crate::op::texture::types::ramp::{TextureRampMode, TextureRampSettings};
use crate::param::{
    ParamBundle, ParamName, ParamOrder, ParamPage, ParamValue, ScriptedParam, ScriptedParamError,
};
use crate::ui::event::{Connect, Disconnect};
use crate::ui::graph::{GraphRef, NodeMaterial, OpRef, SelectedNode};
use crate::ui::UiState;
use crate::{OpName, Sets, ui};
use crate::op::{Op, OpType};

pub mod render;
pub mod types;

pub struct TexturePlugin;

impl Plugin for TexturePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                ExtractComponentPlugin::<TextureOpImage>::default(),
                ExtractComponentPlugin::<TextureOpInputs>::default(),
                TextureOpRampPlugin,
                TextureOpCompositePlugin,
                TextureOpNoisePlugin,
            ))
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    ui::selected_node_ui,
                    update_materials,
                    connect_handler,
                    disconnect_handler,
                )
                    .in_set(Sets::Ui),
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
    commands.insert_resource(TextureOpDefaultImage(image));
}

#[derive(Resource, Clone, Default)]
pub struct TextureOpDefaultImage(pub Handle<Image>);

#[derive(Component, Clone, Debug, Deref, DerefMut, ExtractComponent, Default)]
pub struct TextureOpImage(pub Handle<Image>);

#[derive(Component, ExtractComponent, Clone, Default, Debug)]
pub struct TextureOpInputs {
    pub(crate) count: usize,
    pub(crate) connections: HashMap<Entity, Handle<Image>>,
}

impl TextureOpInputs {
    pub fn is_fully_connected(&self) -> bool {
        self.count == 0 || self.connections.len() == self.count
    }
}

#[derive(Component, Default)]
pub struct TextureOpOutputs {
    pub(crate) count: usize,
}

#[derive(Bundle, Default)]
pub struct TextureOpBundle {
    pub camera: Camera3dBundle,
    pub image: TextureOpImage,
    pub inputs: TextureOpInputs,
    pub outputs: TextureOpOutputs,
}

macro_rules! impl_op {
    ($name:ident, $inputs:expr, $outputs:expr) => {
        impl crate::op::Op for $name {
            const INPUTS: usize = $inputs;
            const OUTPUTS: usize = $outputs;

            type OpType = OpType<Self>;
            type UpdateParam = (
                bevy::ecs::system::lifetimeless::SQuery<(
                    bevy::ecs::system::lifetimeless::Read<Children>, bevy::ecs::system::lifetimeless::Write<<$name as TextureOp>::Uniform>)>,
                bevy::ecs::system::lifetimeless::SQuery<(
                    bevy::ecs::system::lifetimeless::Read<ParamName>, bevy::ecs::system::lifetimeless::Read<ParamValue>
                )>,
            );
            type BundleParam = (bevy::ecs::system::lifetimeless::SResMut<bevy::prelude::Assets<bevy::prelude::Image>>);
            type Bundle = (crate::op::texture::TextureOpBundle, <$name as TextureOp>::Uniform);

            fn update<'w>(entity: bevy::prelude::Entity, param: &mut bevy::ecs::system::SystemParamItem<'w, '_, Self::UpdateParam>) {
                let (self_q, params_q) = param;

                let (children, mut uniform) = self_q.get_mut(entity).unwrap();

                let params = children
                    .iter()
                    .filter_map(|entity| params_q.get(*entity).ok())
                    .collect();

                <$name as TextureOp>::update_uniform(&mut uniform, &params)
            }

            fn create_bundle<'w>(entity: bevy::prelude::Entity, (mut images): &mut bevy::ecs::system::SystemParamItem<'w, '_, Self::BundleParam>) -> Self::Bundle
            {
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
                        image: crate::op::texture::TextureOpImage(image.clone()),
                        inputs: crate::op::texture::TextureOpInputs {
                            count: $inputs,
                            connections: bevy::utils::HashMap::new(),
                        },
                        outputs: crate::op::texture::TextureOpOutputs { count: $outputs },
                    },
                    <$name as TextureOp>::Uniform::default(),
                )
            }

            fn params() -> Vec<crate::param::ParamBundle> {
                let common_params = vec![
                    crate::param::ParamBundle {
                        name: crate::param::ParamName("Resolution".to_string()),
                        value: crate::param::ParamValue::Vec2(Vec2::new(512.0, 512.0)),
                        order: crate::param::ParamOrder(0),
                        page: crate::param::ParamPage("Common".to_string()),
                        ..default()
                    },
                    crate::param::ParamBundle {
                        name: crate::param::ParamName("View".to_string()),
                        value: crate::param::ParamValue::Bool(false),
                        order: crate::param::ParamOrder(1),
                        page: crate::param::ParamPage("Common".to_string()),
                        ..default()
                    },
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
    mut op_q: Query<(&mut TextureOpInputs, &TextureOpImage, &GraphRef)>,
    input_q: Query<&TextureOpImage>,
) {
    for ev in ev_connect.read() {
        if let Ok((mut input, my_image, graph_ref)) = op_q.get_mut(ev.input) {
            if let Ok(image) = input_q.get(ev.output) {
                input.connections.insert(ev.output, image.0.clone());
            }
        }
    }
}

fn update_materials(
    mut materials: ResMut<Assets<NodeMaterial>>,
    mut op_q: Query<(&TextureOpInputs, &TextureOpImage, &GraphRef)>,
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
    mut op_q: Query<&mut TextureOpInputs>,
    input_q: Query<&TextureOpImage>,
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
