use crate::ui::graph::SelectedNode;
use crate::ui::UiState;
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
use bevy::render::render_resource::{
    BindGroupEntries, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntries,
    BindGroupLayoutEntry, BindingType, BufferBindingType, CachedRenderPipelineId, ColorTargetState,
    ColorWrites, FragmentState, LoadOp, MultisampleState, Operations, PipelineCache,
    PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor, RenderPipelineDescriptor,
    Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages, ShaderType, StoreOp,
    TextureFormat, TextureSampleType,
};
use bevy::render::renderer::{RenderContext, RenderDevice};
use bevy::render::texture::BevyDefault;
use bevy::render::view::ViewTarget;
use bevy::render::{render_graph, RenderApp};
use bevy_egui::{egui, EguiContexts};

pub struct CompositePlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderSubGraph)]
pub struct CompositeSubGraph;

impl Plugin for CompositePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<CompositeSettings>::default(),
            UniformComponentPlugin::<CompositeSettings>::default(),
            ExtractComponentPlugin::<CompositeInput>::default(),
        ))
        .add_systems(Update, side_panel_ui);

        app.get_sub_app_mut(RenderApp)
            .unwrap()
            .add_render_sub_graph(CompositeSubGraph)
            .add_render_graph_node::<ViewNodeRunner<CompositeNode>>(
                CompositeSubGraph,
                CompositeLabel,
            );
    }

    fn finish(&self, app: &mut App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<CompositePipeline>();
    }
}

fn side_panel_ui(
    mut ui_state: ResMut<UiState>,
    mut egui_contexts: EguiContexts,
    mut selected_node: Query<(Entity, &mut CompositeSettings, &SelectedNode)>,
) {
    let ctx = egui_contexts.ctx_mut();
    if let Ok((entity, mut settings, _selected_node)) = selected_node.get_single_mut() {
        ui_state.side_panel = Some(
            egui::SidePanel::left("composite_side_panel")
                .resizable(false)
                .show(ctx, |ui| {
                    ui.heading("Composite");
                    let mut mode = CompositeMode::from_u32(settings.mode).expect("Invalid mode");
                    egui::ComboBox::from_label("Mode")
                        .selected_text(format!("{mode:?}"))
                        .show_ui(ui, |ui| {
                            ui.set_min_width(60.0);
                            ui.selectable_value(&mut mode, CompositeMode::Add, "Add");
                            ui.selectable_value(&mut mode, CompositeMode::Multiply, "Multiply");
                            ui.selectable_value(&mut mode, CompositeMode::Subtract, "Subtract");
                            ui.selectable_value(&mut mode, CompositeMode::Divide, "Divide");
                        });
                    settings.mode = mode.as_u32();
                })
                .response,
        );
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum CompositeMode {
    #[default]
    Add = 0,
    Multiply = 1,
    Subtract = 2,
    Divide = 3,
}

impl CompositeMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            CompositeMode::Add => "Add",
            CompositeMode::Multiply => "Multiply",
            CompositeMode::Subtract => "Subtract",
            CompositeMode::Divide => "Divide",
        }
    }

    pub fn as_u32(&self) -> u32 {
        *self as u32
    }

    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(CompositeMode::Add),
            1 => Some(CompositeMode::Multiply),
            2 => Some(CompositeMode::Subtract),
            3 => Some(CompositeMode::Divide),
            _ => None,
        }
    }
}

// This is the component that will get passed to the shader
#[derive(Component, Default, Debug, Clone, Copy, ExtractComponent, ShaderType)]
pub struct CompositeSettings {
    pub mode: u32,
    // WebGL2 structs must be 16 byte aligned.
    #[cfg(feature = "webgl2")]
    _webgl2_padding: Vec3,
}

#[derive(Component, Default, Debug, Clone, ExtractComponent)]
pub struct CompositeInput(pub Vec<Handle<Image>>);

#[derive(Resource)]
struct CompositePipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for CompositePipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let layout = render_device.create_bind_group_layout(
            "composite_bind_group_layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::FRAGMENT,
                (
                    uniform_buffer::<CompositeSettings>(true),
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                ),
            ),
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let shader = world
            .resource::<AssetServer>()
            .load("shaders/texture/composite.wgsl");

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
            sampler,
            pipeline_id,
        }
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct CompositeLabel;

#[derive(Default)]
struct CompositeNode;

impl render_graph::ViewNode for CompositeNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static DynamicUniformIndex<CompositeSettings>,
        &'static CompositeInput,
    );
    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, uniform_index, composite_input): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let composite_pipeline = world.resource::<CompositePipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let images = world.resource::<RenderAssets<Image>>();

        let Some(pipeline) = pipeline_cache.get_render_pipeline(composite_pipeline.pipeline_id)
        else {
            return Ok(());
        };

        let settings_uniforms = world.resource::<ComponentUniforms<CompositeSettings>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        let bind_group = render_context.render_device().create_bind_group(
            "composite_bind_group",
            &composite_pipeline.layout,
            &BindGroupEntries::sequential((
                settings_binding.clone(),
                &images.get(&composite_input.0[0]).unwrap().texture_view,
                &images.get(&composite_input.0[0]).unwrap().sampler,
                &images.get(&composite_input.0[1]).unwrap().texture_view,
                &images.get(&composite_input.0[1]).unwrap().sampler,
            )),
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
