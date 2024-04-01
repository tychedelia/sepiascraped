use std::fmt::Debug;
use std::ops::Deref;

use bevy::ecs::query::QueryData;
use bevy::ecs::system::lifetimeless;
use bevy::ecs::system::lifetimeless::{Read, SQuery, Write};
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::render_resource::encase::internal::WriteInto;
use bevy::render::render_resource::{
    Extent3d, ShaderType, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::sprite::Material2d;
use lifetimeless::SResMut;

use types::composite::TextureOpCompositePlugin;
use types::ramp::TextureOpRampPlugin;

use crate::op::texture::render::TextureOpInputImages;
use crate::op::texture::types::noise::TextureOpNoisePlugin;
use crate::op::{Op, OpDefaultImage, OpImage, OpInputs, OpOutputs};
use crate::param::{ParamBundle, ParamName, ParamValue};

pub mod render;
pub mod types;

pub const CATEGORY: &str = "Texture";

pub struct TexturePlugin;

impl Plugin for TexturePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<TextureOpInputImages>::default(),
            ExtractComponentPlugin::<OpInputs>::default(),
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
pub struct TextureOpBundle {
    pub camera: Camera3dBundle,
    pub image: OpImage,
    inputs: OpInputs,
    outputs: OpOutputs,
}

type DefaultTextureUpdateParam<T: TextureOp> =(
    SQuery<(Read<Children>, Write<OpImage>, Write<T::Uniform>)>,
    SQuery<(Read<ParamName>, Read<ParamValue>)>,
    SResMut<Assets<Image>>,
);

fn update<'w, T: TextureOp>(
    entity: Entity,
    param: &mut bevy::ecs::system::SystemParamItem<
        'w,
        '_,
        DefaultTextureUpdateParam<T>,
    >,
) {
    let (self_q, params_q, ref mut images) = param;

    let Ok((children, mut image, mut uniform)) = self_q.get_mut(entity) else {
        return;
    };

    let params = children
        .iter()
        .filter_map(|entity| params_q.get(*entity).ok())
        .collect();

    T::update_uniform(&mut uniform, &params);

    let resolution = params
        .iter()
        .find(|(name, _)| *name == &crate::param::ParamName("Resolution".to_string()))
        .unwrap()
        .1;

    if let crate::param::ParamValue::Vec2(resolution) = resolution {
        let image_size = images.get(&image.0).unwrap().size();
        if image_size.x != resolution.x as u32 || image_size.y != resolution.y as u32 {
            let mut new_image =
                crate::op::OpImage::new_image(resolution.x as u32, resolution.y as u32);
            let new_image = images.add(new_image);
            *image = crate::op::OpImage(new_image);
        }
    }
}

type DefaultTextureSpawnParam = (SResMut<Assets<Image>>);
type DefaultTextureBundle<T: TextureOp> = (TextureOpBundle, TextureOpInputImages, T::Uniform);

fn create_bundle<'w, T: TextureOp>(
    entity: Entity,
    (mut images): &mut bevy::ecs::system::SystemParamItem<'w, '_, DefaultTextureSpawnParam<>>,
) -> DefaultTextureBundle<T> {
    let image = images.add(crate::op::OpImage::new_image(512, 512));
    (
        TextureOpBundle {
            camera: Camera3dBundle {
                camera_render_graph: bevy::render::camera::CameraRenderGraph::new(
                    render::TextureOpSubGraph,
                ),
                camera: Camera {
                    order: 3,
                    target: image.clone().into(),
                    ..default()
                },
                ..default()
            },
            image: OpImage(image.clone()),
            inputs: OpInputs {
                count: T::INPUTS,
                connections: Vec::new(),
            },
            outputs: crate::op::OpOutputs { count: T::OUTPUTS },
        },
        TextureOpInputImages::default(),
        T::Uniform::default(),
    )
}

fn params<T: TextureOp>(bundle: &DefaultTextureBundle<T>) -> Vec<ParamBundle> {
    let common_params = vec![ParamBundle {
        name: ParamName("Resolution".to_string()),
        value: ParamValue::UVec2(UVec2::new(512, 512)),
        order: crate::param::ParamOrder(0),
        page: crate::param::ParamPage("Common".to_string()),
        ..default()
    }];

    [common_params, <T as TextureOp>::params()].concat()
}

type DefaultTextureOnConnectParam =  (
    lifetimeless::SCommands,
    SResMut<Assets<crate::ui::graph::NodeMaterial>>,
    SQuery<(
        Read<OpImage>,
        Read<crate::ui::graph::GraphRef>,
        Write<TextureOpInputImages>,
    )>,
    SQuery<Read<Handle<crate::ui::graph::NodeMaterial>>>,
);
fn on_connect<'w>(
    entity: Entity,
    event: crate::ui::event::Connect,
    fully_connected: bool,
    param: &mut bevy::ecs::system::SystemParamItem<
        'w,
        '_,
        DefaultTextureOnConnectParam,
    >,
) {
    let (ref mut commands, ref mut materials, ref mut op_q, ref mut material_q) = param;
    let (new_image, _, _) = op_q.get(event.output).unwrap();
    let new_image = new_image.0.clone();
    let (my_image, graph_ref, mut my_images) = op_q.get_mut(entity).unwrap();
    my_images.push(new_image);

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
