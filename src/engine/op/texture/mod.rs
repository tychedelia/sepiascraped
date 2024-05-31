use std::fmt::Debug;
use std::ops::Deref;

use bevy::ecs::query::QueryData;
use bevy::ecs::system::lifetimeless::{Read, SQuery, Write};
use bevy::ecs::system::{lifetimeless, SystemParamItem};
use bevy::prelude::*;
use bevy::render::camera::{CameraRenderGraph, RenderTarget};
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::render_resource::encase::internal::WriteInto;
use bevy::render::render_resource::{
    Extent3d, ShaderType, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::sprite::Material2d;
use lifetimeless::SResMut;

use types::composite::TextureOpCompositePlugin;
use types::ramp::TextureOpRampPlugin;

use crate::engine::op::texture::render::TextureOpInputImages;
use crate::engine::op::texture::types::noise::TextureOpNoisePlugin;
use crate::engine::op::{Op, OpDefaultImage, OpImage, OpInputs, OpOutputs};
use crate::engine::param::{ParamBundle, ParamName, ParamValue};

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

type DefaultTextureUpdateParam<T> = (
    SQuery<(
        Read<Children>,
        Write<OpImage>,
        Write<<T as TextureOp>::Uniform>,
    )>,
    SQuery<(Read<ParamName>, Read<ParamValue>)>,
    SResMut<Assets<Image>>,
);

fn update<'w, T: TextureOp>(
    entity: Entity,
    param: &mut SystemParamItem<'w, '_, DefaultTextureUpdateParam<T>>,
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
        .find(|(name, _)| *name == &crate::engine::param::ParamName("Resolution".to_string()))
        .unwrap()
        .1;

    if let crate::engine::param::ParamValue::Vec2(resolution) = resolution {
        let image_size = images.get(&image.0).unwrap().size();
        if image_size.x != resolution.x as u32 || image_size.y != resolution.y as u32 {
            let mut new_image =
                crate::engine::op::OpImage::new_image(resolution.x as u32, resolution.y as u32);
            let new_image = images.add(new_image);
            *image = crate::engine::op::OpImage(new_image);
        }
    }
}

type DefaultTextureSpawnParam = (SResMut<Assets<Image>>);
type DefaultTextureBundle<T> = (
    TextureOpBundle,
    TextureOpInputImages,
    <T as TextureOp>::Uniform,
);

fn create_bundle<'w, T: TextureOp>(
    entity: Entity,
    (mut images): &mut SystemParamItem<'w, '_, DefaultTextureSpawnParam>,
) -> DefaultTextureBundle<T> {
    let image = images.add(OpImage::new_image(512, 512));

    (
        TextureOpBundle {
            camera: Camera3dBundle {
                camera_render_graph: CameraRenderGraph::new(
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
            inputs: OpInputs::new(T::INPUTS),
            outputs: OpOutputs { count: T::OUTPUTS },
        },
        TextureOpInputImages::default(),
        T::Uniform::default(),
    )
}

fn params<T: TextureOp>(bundle: &DefaultTextureBundle<T>) -> Vec<ParamBundle> {
    let common_params = vec![ParamBundle {
        name: ParamName("Resolution".to_string()),
        value: ParamValue::UVec2(UVec2::new(512, 512)),
        order: crate::engine::param::ParamOrder(0),
        page: crate::engine::param::ParamPage("Common".to_string()),
        ..default()
    }];

    [common_params, <T as TextureOp>::params()].concat()
}

type DefaultTextureOnConnectParam = (
    lifetimeless::SCommands,
    SQuery<(Read<OpImage>, Write<TextureOpInputImages>)>,
);

type DefaultTextureOnDisconnectParam = (
    lifetimeless::SCommands,
    SResMut<Assets<Image>>,
    SQuery<(Write<OpImage>, Write<TextureOpInputImages>)>,
);

fn on_connect<'w>(
    entity: Entity,
    event: crate::engine::graph::event::Connect,
    fully_connected: bool,
    param: &mut SystemParamItem<'w, '_, DefaultTextureOnConnectParam>,
) {
    let (ref mut commands, ref mut op_q) = param;
    let (new_image, _) = op_q.get(event.output).unwrap();
    let new_image = new_image.0.clone();
    let (_, mut my_images) = op_q.get_mut(entity).unwrap();
    my_images.insert(event.output, new_image);
}

fn on_disconnect<'w>(
    entity: Entity,
    event: crate::engine::graph::event::Disconnect,
    fully_connected: bool,
    param: &mut SystemParamItem<'w, '_, DefaultTextureOnDisconnectParam>,
) {
    let (ref mut commands, ref mut images,  ref mut op_q) = param;
    let (my_image, mut my_images) = op_q.get_mut(entity).unwrap();
    my_images.remove(&event.output);
    if !fully_connected {
        let mut my_image = images.get_mut(&my_image.0).unwrap();
        *my_image = OpImage::new_image(my_image.width(), my_image.height());
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
