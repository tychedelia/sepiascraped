use crate::ui::graph::SelectedNode;
use crate::ui::UiState;
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
use bevy::render::render_resource::{
    BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, BindingType, BufferBindingType,
    CachedRenderPipelineId, ColorTargetState, ColorWrites, FragmentState, LoadOp, MultisampleState,
    Operations, PipelineCache, PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipelineDescriptor, Sampler, SamplerDescriptor, ShaderStages, ShaderType, StoreOp,
    TextureFormat,
};
use bevy::render::renderer::{RenderContext, RenderDevice};
use bevy::render::texture::BevyDefault;
use bevy::render::view::ViewTarget;
use bevy::render::{render_graph, RenderApp};
use bevy_egui::{egui, EguiContexts};

pub struct TextureRampPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderSubGraph)]
pub struct TextureRampSubGraph;

impl Plugin for TextureRampPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<TextureRampSettings>::default(),
            UniformComponentPlugin::<TextureRampSettings>::default(),
        ))
        .add_systems(Update, side_panel_ui);

        app.get_sub_app_mut(RenderApp)
            .unwrap()
            .add_render_sub_graph(TextureRampSubGraph)
            .add_render_graph_node::<ViewNodeRunner<TextureRampNode>>(
                TextureRampSubGraph,
                TextureRampLabel,
            );
    }

    fn finish(&self, app: &mut App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        render_app.init_resource::<TextureRampPipeline>();
    }
}

fn side_panel_ui(
    mut ui_state: ResMut<UiState>,
    mut egui_contexts: EguiContexts,
    mut selected_node: Query<(Entity, &mut TextureRampSettings, &SelectedNode)>,
) {
    let ctx = egui_contexts.ctx_mut();
    if let Ok((entity, mut settings, _selected_node)) = selected_node.get_single_mut() {
        ui_state.side_panel = Some(
            egui::Window::new("texture_ramp_side_panel")
                .resizable(false)
                .show(ctx, |ui| {
                    ui.heading("Ramp");
                    ui.separator();
                    ui.label("Color A");
                    ui.color_edit_button_rgba_premultiplied(settings.color_a.as_mut());
                    ui.label("Color B");
                    ui.color_edit_button_rgba_premultiplied(settings.color_b.as_mut());
                    let mut mode = TextureRampMode::from_u32(settings.mode).expect("Invalid mode");
                    egui::ComboBox::from_label("Mode")
                        .selected_text(format!("{mode:?}"))
                        .show_ui(ui, |ui| {
                            ui.set_min_width(60.0);
                            ui.selectable_value(
                                &mut mode,
                                TextureRampMode::Horizontal,
                                "Horizontal",
                            );
                            ui.selectable_value(&mut mode, TextureRampMode::Vertical, "Vertical");
                            ui.selectable_value(&mut mode, TextureRampMode::Circular, "Circular");
                        });
                    settings.mode = mode.as_u32();
                })
                .unwrap()
                .response,
        );
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum TextureRampMode {
    #[default]
    Horizontal = 0,
    Vertical = 1,
    Circular = 2,
}

impl TextureRampMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            TextureRampMode::Horizontal => "Horizontal",
            TextureRampMode::Vertical => "Vertical",
            TextureRampMode::Circular => "Circular",
        }
    }

    pub fn as_u32(&self) -> u32 {
        *self as u32
    }

    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(TextureRampMode::Horizontal),
            1 => Some(TextureRampMode::Vertical),
            2 => Some(TextureRampMode::Circular),
            _ => None,
        }
    }
}

// This is the component that will get passed to the shader
#[derive(Component, Default, Debug, Clone, Copy, ExtractComponent, ShaderType)]
pub struct TextureRampSettings {
    pub color_a: Vec4,
    pub color_b: Vec4,
    pub mode: u32,
    // WebGL2 structs must be 16 byte aligned.
    #[cfg(feature = "webgl2")]
    _webgl2_padding: Vec3,
}

#[derive(Resource)]
struct TextureRampPipeline {
    layout: BindGroupLayout,
    sampler: Sampler,
    pipeline_id: CachedRenderPipelineId,
}

impl FromWorld for TextureRampPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let layout = render_device.create_bind_group_layout(
            "texture_ramp_bind_group_layout",
            &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: Some(TextureRampSettings::min_size()),
                },
                count: None,
            }],
        );

        let sampler = render_device.create_sampler(&SamplerDescriptor::default());

        let shader = world
            .resource::<AssetServer>()
            .load("shaders/texture/ramp.wgsl");

        let pipeline_id =
            world
                .resource_mut::<PipelineCache>()
                .queue_render_pipeline(RenderPipelineDescriptor {
                    label: Some("texture_ramp_pipeline".into()),
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
struct TextureRampLabel;

#[derive(Default)]
struct TextureRampNode;

impl render_graph::ViewNode for TextureRampNode {
    type ViewQuery = (
        &'static ViewTarget,
        &'static DynamicUniformIndex<TextureRampSettings>,
    );
    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        (view_target, uniform_index): QueryItem<Self::ViewQuery>,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let texture_ramp_pipeline = world.resource::<TextureRampPipeline>();
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(pipeline) = pipeline_cache.get_render_pipeline(texture_ramp_pipeline.pipeline_id)
        else {
            return Ok(());
        };

        let settings_uniforms = world.resource::<ComponentUniforms<TextureRampSettings>>();
        let Some(settings_binding) = settings_uniforms.uniforms().binding() else {
            return Ok(());
        };

        let bind_group = render_context.render_device().create_bind_group(
            "texture_ramp_bind_group",
            &texture_ramp_pipeline.layout,
            &[BindGroupEntry {
                binding: 0,
                resource: settings_binding.clone(),
            }],
        );

        let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
            label: Some("texture_ramp_pass"),
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