use bevy::core_pipeline::fullscreen_vertex_shader::fullscreen_shader_vertex_state;
use bevy::ecs::query::QueryItem;
use bevy::prelude::*;
use bevy::render::extract_component::{
    ComponentUniforms, DynamicUniformIndex, ExtractComponent, ExtractComponentPlugin,
    UniformComponentPlugin,
};
use bevy::render::render_graph::{
    NodeRunError, RenderGraphApp, RenderGraphContext, RenderLabel, RenderSubGraph, ViewNodeRunner,
};
use bevy::render::render_resource::encase::internal::WriteInto;
use bevy::render::render_resource::{
    BindGroupEntries, BindGroupLayout, BindGroupLayoutEntries, CachedRenderPipelineId,
    ColorTargetState, ColorWrites, FragmentState, IntoBindGroupLayoutEntryBuilderArray,
    IntoBindingArray, LoadOp, MultisampleState, Operations, PipelineCache, PrimitiveState,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor, SamplerDescriptor,
    ShaderStages, ShaderType, StoreOp, TextureFormat,
};
use bevy::render::renderer::{RenderContext, RenderDevice};
use bevy::render::texture::BevyDefault;
use bevy::render::view::ViewTarget;
use bevy::render::{render_graph, RenderApp};

use crate::texture::TextureOpInputs;

#[derive(Default)]
pub struct TextureOpRenderPlugin<P, const N: usize = 0> {
    _marker: std::marker::PhantomData<P>,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderSubGraph)]
pub struct TextureSubGraph;

impl<P, const N: usize> Plugin for TextureOpRenderPlugin<P, N>
where
    P: TextureOpRenderNode<N> + Sync + Send + 'static,
    ViewNodeRunner<TextureOperatorViewNode<P, N>>: FromWorld,
{
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<P::Uniform>::default(),
            UniformComponentPlugin::<P::Uniform>::default(),
        ));

        app.get_sub_app_mut(RenderApp)
            .unwrap()
            .add_render_sub_graph(P::render_sub_graph())
            .add_render_graph_node::<ViewNodeRunner<TextureOperatorViewNode<P, N>>>(
                P::render_sub_graph(),
                P::render_label(),
            );
    }

    fn finish(&self, app: &mut App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<TextureOpPipeline<P, N>>();
    }
}

pub trait TextureOpRenderNode<const N: usize = 1> {
    const SHADER: &'static str;
    type Uniform: Component + ExtractComponent + ShaderType + WriteInto + Clone;

    fn render_sub_graph() -> impl RenderSubGraph;

    fn render_label() -> impl RenderLabel;

    fn bind_group_layout_entries() -> impl IntoBindGroupLayoutEntryBuilderArray<N>;

    fn bind_group_entries<'a>(
        inputs: &'a TextureOpInputs,
        world: &'a World,
    ) -> impl IntoBindingArray<'a, N>;
}

#[derive(Resource)]
struct TextureOpPipeline<P, const N: usize> {
    layout: BindGroupLayout,
    pipeline_id: CachedRenderPipelineId,
    _plugin: std::marker::PhantomData<P>,
}

impl<P, const N: usize> FromWorld for TextureOpPipeline<P, N>
where
    P: TextureOpRenderNode<N> + Sync + Send + 'static,
{
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let layout = render_device.create_bind_group_layout(
            "composite_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                P::bind_group_layout_entries(),
            ),
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let shader = world.resource::<AssetServer>().load(P::SHADER);

        let pipeline_id =
            world
                .resource_mut::<PipelineCache>()
                .queue_render_pipeline(RenderPipelineDescriptor {
                    label: Some("composite_pipeline".into()),
                    layout: vec![layout.clone()],
                    vertex: fullscreen_shader_vertex_state(),
                    fragment: Some(FragmentState {
                        shader,
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
                });

        Self {
            layout,
            pipeline_id,
            _plugin: Default::default(),
        }
    }
}

#[derive(Default)]
struct TextureOperatorViewNode<P, const N: usize> {
    _plugin: std::marker::PhantomData<P>,
}

impl<P, const N: usize> render_graph::ViewNode for TextureOperatorViewNode<P, N>
where
    P: TextureOpRenderNode<N> + Sync + Send + 'static,
{
    type ViewQuery = (
        &'static ViewTarget,
        &'static DynamicUniformIndex<P::Uniform>,
        &'static TextureOpInputs,
    );

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, uniform_index, inputs): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        if inputs.connections.len() < inputs.count {
            return Ok(());
        }

        let composite_pipeline = world.resource::<TextureOpPipeline<P, N>>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(pipeline) = pipeline_cache.get_render_pipeline(composite_pipeline.pipeline_id)
        else {
            return Ok(());
        };

        let settings_uniforms = world.resource::<ComponentUniforms<P::Uniform>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        let bind_group = render_context.render_device().create_bind_group(
            "composite_bind_group",
            &composite_pipeline.layout,
            &BindGroupEntries::sequential(P::bind_group_entries(inputs, world)),
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("composite_pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view_target.out_texture(),
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
        render_pass.set_bind_group(0, &bind_group, &[uniform_index.index()]);
        render_pass.draw(0..3, 0..1);

        Ok(())
    }
}
