use std::fmt::Debug;
use std::marker::PhantomData;

use bevy::asset::LoadState;
use bevy::core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state;
use bevy::ecs::query::QueryItem;
use bevy::prelude::*;
use bevy::render::extract_component::{
    ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
    UniformComponentPlugin,
};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph::{
    NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, RenderSubGraph, ViewNodeRunner,
};
use bevy::render::render_resource::binding_types::{sampler, texture_2d, uniform_buffer};
use bevy::render::render_resource::encase::internal::WriteInto;
use bevy::render::render_resource::{
    BindGroup, BindGroupEntry, BindGroupLayout, CachedRenderPipelineId, ColorTargetState,
    ColorWrites, FragmentState, IntoBinding, LoadOp, MultisampleState, Operations, PipelineCache,
    PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor,
    SamplerBindingType, ShaderStages, ShaderType, SpecializedRenderPipeline,
    SpecializedRenderPipelines, StoreOp, TextureFormat, TextureSampleType,
};
use bevy::render::renderer::{RenderContext, RenderDevice};
use bevy::render::texture::BevyDefault;
use bevy::render::view::{ExtractedView, ViewTarget};
use bevy::render::{render_graph, Render, RenderApp, RenderSet};
use bevy::utils::{info, HashMap};
use crate::op::{Op, OpType};

use crate::op::texture::types::composite::TextureOpComposite;
use crate::op::texture::types::ramp::TextureOpRamp;
use crate::op::texture::{TextureOpInputs, TextureOp};

#[derive(Default)]
pub struct TextureOpRenderPlugin<T> {
    _marker: std::marker::PhantomData<T>,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderSubGraph)]
pub struct TextureOpSubGraph;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct TextureOpRenderLabel;

impl<T> Plugin for TextureOpRenderPlugin<T>
where
    T: TextureOp + Component + Clone + Debug + Send + Sync + 'static
{
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<T::Uniform>::default(),
            UniformComponentPlugin::<T::Uniform>::default(),
        ));

        app.get_sub_app_mut(RenderApp)
            .unwrap()
            .init_resource::<SpecializedRenderPipelines<TextureOpPipeline>>()
            .add_render_sub_graph(TextureOpSubGraph)
            .add_render_graph_node::<ViewNodeRunner<TextureOpViewNode>>(
                TextureOpSubGraph,
                TextureOpRenderLabel,
            )
            .add_systems(
                Render,
                (
                    prepare_texture_op_pipelines::<T>.in_set(RenderSet::Prepare),
                    prepare_texture_op_bind_group::<T>.in_set(RenderSet::PrepareBindGroups),
                ),
            );
    }

    fn finish(&self, app: &mut App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        let asset_server = render_app.world.resource_mut::<AssetServer>();
        let shader_handle = asset_server.load(T::SHADER);
        let shader_handle = TextureOpShaderHandle::<T>(shader_handle, PhantomData);
        render_app
            .insert_resource(shader_handle)
            .init_resource::<TextureOpPipeline>();
    }
}

#[derive(Resource, Debug)]
pub struct TextureOpShaderHandle<T>(pub Handle<Shader>, PhantomData<T>);

pub fn prepare_texture_op_pipelines<T>(
    mut commands: Commands,
    mut pipeline: ResMut<TextureOpPipeline>,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<TextureOpPipeline>>,
    views: Query<(Entity, &ExtractedView, &TextureOpInputs), With<<T as Op>::OpType>>,
    shader_handle: Res<TextureOpShaderHandle<T>>,
    render_device: Res<RenderDevice>,
) where
    T:  TextureOp + Component + Clone + Debug + Send + Sync + 'static
{
    for (entity, view, inputs) in views.iter() {
        if !inputs.is_fully_connected() {
            continue;
        }

        let mut entries = vec![uniform_buffer::<T::Uniform>(true).build(0, ShaderStages::FRAGMENT)];

        for i in 0..inputs.count {
            let idx = i as u32 * 2 + 1;
            entries.push(
                texture_2d(TextureSampleType::Float { filterable: true })
                    .build(idx, ShaderStages::FRAGMENT),
            );
            entries.push(
                sampler(SamplerBindingType::Filtering).build(idx + 1, ShaderStages::FRAGMENT),
            );
        }

        let key = TextureOpPipelineKey {
            input_count: inputs.count,
            shader: shader_handle.0.clone(),
        };

        let layout =
            render_device.create_bind_group_layout("texture_op_bind_group_layout", &entries);
        pipeline.layouts.insert(key.clone(), layout);

        let pipeline_id = pipelines.specialize(&pipeline_cache, &pipeline, key.clone());
        commands
            .entity(entity)
            .insert(TextureOpPipelineId(pipeline_id));
    }
}

pub fn prepare_texture_op_bind_group<T>(
    mut commands: Commands,
    pipeline: ResMut<TextureOpPipeline>,
    uniforms: Res<ComponentUniforms<T::Uniform>>,
    views: Query<
        (
            Entity,
            &ExtractedView,
            &TextureOpInputs,
            &DynamicUniformIndex<T::Uniform>,
        ),
        With<<T as Op>::OpType>
    >,
    shader_handle: Res<TextureOpShaderHandle<T>>,
    images: Res<RenderAssets<Image>>,
    render_device: Res<RenderDevice>,
) where
    T: TextureOp + Component + Clone + Debug + Send + Sync + 'static
{
    for (entity, view, inputs, uniform_index) in views.iter() {
        if !inputs.is_fully_connected() {
            continue;
        }

        let mut gpu_images = vec![];
        for connection in inputs.connections.iter() {
            if let Some(image) = images.get(connection.1) {
                gpu_images.push(image);
            }
        }

        // Not all our images are loaded yet
        if gpu_images.len() < inputs.count {
            continue;
        }

        let Some(uniforms_binding) = uniforms.uniforms().binding() else {
            warn!("TextureOp has no uniforms {}", T::SHADER);
            continue;
        };

        let mut entries = vec![BindGroupEntry {
            binding: 0,
            resource: uniforms_binding,
        }];

        for (idx, image) in gpu_images.iter().enumerate() {
            let idx = (idx * 2 + 1) as u32;
            entries.push(BindGroupEntry {
                binding: idx,
                resource: image.texture_view.into_binding(),
            });
            entries.push(BindGroupEntry {
                binding: idx + 1,
                resource: image.sampler.into_binding(),
            });
        }

        let bind_group = render_device.create_bind_group(
            "texture_op_bind_group",
            &pipeline.layouts[&TextureOpPipelineKey {
                input_count: inputs.count,
                shader: shader_handle.0.clone(),
            }],
            &entries[..],
        );

        commands
            .entity(entity)
            .insert(TextureOpBindGroup((bind_group, uniform_index.index())));
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct TextureOpPipelineKey {
    pub input_count: usize,
    pub shader: Handle<Shader>,
}

#[derive(Resource, Default)]
struct TextureOpPipeline {
    layouts: HashMap<TextureOpPipelineKey, BindGroupLayout>,
}

impl SpecializedRenderPipeline for TextureOpPipeline {
    type Key = TextureOpPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let layout = self.layouts[&key].clone();
        RenderPipelineDescriptor {
            label: Some("texture_op_pipeline".into()),
            layout: vec![layout],
            vertex: fullscreen_shader_vertex_state(),
            fragment: Some(FragmentState {
                shader: key.shader,
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            push_constant_ranges: vec![],
        }
    }
}

#[derive(Component, Debug)]
pub struct TextureOpPipelineId(pub CachedRenderPipelineId);

#[derive(Component, Debug)]
pub struct TextureOpBindGroup(pub (BindGroup, u32));

#[derive(Default)]
struct TextureOpViewNode;

impl render_graph::ViewNode for TextureOpViewNode {
    type ViewQuery = (
        Entity,
        &'static ViewTarget,
        &'static TextureOpBindGroup,
        &'static TextureOpPipelineId,
    );

    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (entity, view_target, bind_group, pipeline_id): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let Some(pipeline) = pipeline_cache.get_render_pipeline(pipeline_id.0) else {
            warn!("TextureOpViewNode missing pipeline {:?}", pipeline_id);
            return Ok(());
        };

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("texture_op_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: view_target.out_texture(),
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK.into()),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_render_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group.0 .0, &[bind_group.0 .1]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}
