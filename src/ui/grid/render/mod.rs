use std::borrow::Cow;

use bevy::core_pipeline::core_2d::Transparent2d;
use bevy::math::FloatOrd;
use bevy::render::render_phase::{PhaseItemExtraIndex, SortedRenderPhase, ViewSortedRenderPhases};
use bevy::render::view::{ViewUniform, ViewUniformOffset, ViewUniforms};
use bevy::{
    ecs::{
        query::ROQueryItem,
        system::{
            lifetimeless::{Read, SRes},
            SystemParamItem,
        },
    },
    pbr::MeshPipelineKey,
    prelude::*,
    render::{
        mesh::PrimitiveTopology,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult,
            SetItemPipeline,
        },
        render_resource::{
            BindGroup, BindGroupEntries, BindGroupLayout, BindGroupLayoutEntry, BindingType,
            BlendState, BufferBindingType, BufferSize, ColorTargetState, ColorWrites,
            DynamicUniformBuffer, FragmentState, MultisampleState, PipelineCache, PolygonMode,
            PrimitiveState, RenderPipelineDescriptor, ShaderStages, ShaderType,
            SpecializedRenderPipeline, SpecializedRenderPipelines, TextureFormat, VertexState,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::BevyDefault,
        view::{ExtractedView, ViewTarget, VisibleEntities},
        Extract, ExtractSchedule, Render, RenderApp, RenderSet,
    },
};

use crate::ui::grid::InfiniteGridSettings;

static PLANE_RENDER: &str = include_str!("grid.wgsl");

const SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(15204473893972682982);

#[derive(Component)]
struct ExtractedInfiniteGrid {
    transform: GlobalTransform,
    grid: InfiniteGridSettings,
}

#[derive(Debug, ShaderType)]
pub struct InfiniteGridUniform {
    translation: Vec3,
}

#[derive(Debug, ShaderType)]
pub struct GridDisplaySettingsUniform {
    x_axis_color: Vec3,
    y_axis_color: Vec3,
    minor_line_color: Vec4,
    major_line_color: Vec4,
}

#[derive(Resource, Default)]
struct InfiniteGridUniforms {
    uniforms: DynamicUniformBuffer<InfiniteGridUniform>,
}

#[derive(Resource, Default)]
struct GridDisplaySettingsUniforms {
    uniforms: DynamicUniformBuffer<GridDisplaySettingsUniform>,
}

#[derive(Component)]
struct InfiniteGridUniformOffsets {
    position_offset: u32,
    settings_offset: u32,
}

#[derive(Component)]
pub struct PerCameraSettingsUniformOffset {
    offset: u32,
}

#[derive(Resource)]
struct InfiniteGridBindGroup {
    value: BindGroup,
}

#[derive(Component)]
struct GridViewBindGroup {
    value: BindGroup,
}

struct SetGridViewBindGroup<const I: usize>;

impl<const I: usize, P: PhaseItem> RenderCommand<P> for SetGridViewBindGroup<I> {
    type Param = ();
    type ViewQuery = (Read<ViewUniformOffset>, Read<GridViewBindGroup>);
    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        _item: &P,
        (view_uniform, bind_group): ROQueryItem<'w, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, Self::ItemQuery>>,
        _param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(I, &bind_group.value, &[view_uniform.offset]);

        RenderCommandResult::Success
    }
}

struct SetInfiniteGridBindGroup<const I: usize>;

impl<const I: usize, P: PhaseItem> RenderCommand<P> for SetInfiniteGridBindGroup<I> {
    type Param = SRes<InfiniteGridBindGroup>;
    type ViewQuery = Option<Read<PerCameraSettingsUniformOffset>>;
    type ItemQuery = Read<InfiniteGridUniformOffsets>;

    #[inline]
    fn render<'w>(
        _item: &P,
        camera_settings_offset: ROQueryItem<'w, Self::ViewQuery>,
        base_offsets: Option<ROQueryItem<'w, Self::ItemQuery>>,
        bind_group: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Some(base_offsets) = base_offsets {
            pass.set_bind_group(
                I,
                &bind_group.into_inner().value,
                &[
                    base_offsets.position_offset,
                    camera_settings_offset
                        .map(|cs| cs.offset)
                        .unwrap_or(base_offsets.settings_offset),
                ],
            );
        }
        RenderCommandResult::Success
    }
}

struct FinishDrawInfiniteGrid;

impl<P: PhaseItem> RenderCommand<P> for FinishDrawInfiniteGrid {
    type Param = ();
    type ViewQuery = ();
    type ItemQuery = ();

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: ROQueryItem<'w, Self::ViewQuery>,
        _entity: Option<ROQueryItem<'w, Self::ItemQuery>>,
        _param: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut bevy::render::render_phase::TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.draw(0..4, 0..1);
        RenderCommandResult::Success
    }
}

fn prepare_grid_view_bind_groups(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    uniforms: Res<ViewUniforms>,
    pipeline: Res<InfiniteGridPipeline>,
    views: Query<Entity, With<ViewUniformOffset>>,
) {
    if let Some(binding) = uniforms.uniforms.binding() {
        for entity in views.iter() {
            let bind_group = render_device.create_bind_group(
                "grid-view-bind-group",
                &pipeline.view_layout,
                &BindGroupEntries::single(binding.clone()),
            );
            commands
                .entity(entity)
                .insert(GridViewBindGroup { value: bind_group });
        }
    }
}

fn extract_infinite_grids(
    mut commands: Commands,
    grids: Extract<
        Query<(
            Entity,
            &InfiniteGridSettings,
            &GlobalTransform,
            &VisibleEntities,
        )>,
    >,
) {
    let extracted: Vec<_> = grids
        .iter()
        .map(|(entity, grid, transform, visible_entities)| {
            (
                entity,
                (
                    ExtractedInfiniteGrid {
                        transform: *transform,
                        grid: *grid,
                    },
                    visible_entities.clone(),
                ),
            )
        })
        .collect();
    commands.insert_or_spawn_batch(extracted);
}

fn extract_per_camera_settings(
    mut commands: Commands,
    cameras: Extract<Query<(Entity, &InfiniteGridSettings), With<Camera>>>,
) {
    let extracted: Vec<_> = cameras
        .iter()
        .map(|(entity, settings)| (entity, *settings))
        .collect();
    commands.insert_or_spawn_batch(extracted);
}

fn prepare_infinite_grids(
    mut commands: Commands,
    grids: Query<(Entity, &ExtractedInfiniteGrid)>,
    cameras: Query<(Entity, &InfiniteGridSettings), With<ExtractedView>>,
    mut position_uniforms: ResMut<InfiniteGridUniforms>,
    mut settings_uniforms: ResMut<GridDisplaySettingsUniforms>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    position_uniforms.uniforms.clear();
    for (entity, extracted) in grids.iter() {
        let transform = extracted.transform;
        let offset = transform.translation();
        commands.entity(entity).insert(InfiniteGridUniformOffsets {
            position_offset: position_uniforms.uniforms.push(&InfiniteGridUniform {
                translation: offset,
            }),
            settings_offset: settings_uniforms
                .uniforms
                .push(&GridDisplaySettingsUniform {
                    x_axis_color: Vec3::from_slice(
                        &extracted.grid.x_axis_color.linear().to_f32_array(),
                    ),
                    y_axis_color: Vec3::from_slice(
                        &extracted.grid.y_axis_color.linear().to_f32_array(),
                    ),
                    minor_line_color: Vec4::from_slice(
                        &extracted.grid.minor_line_color.linear().to_f32_array(),
                    ),
                    major_line_color: Vec4::from_slice(
                        &extracted.grid.major_line_color.linear().to_f32_array(),
                    ),
                }),
        });
    }

    for (entity, settings) in cameras.iter() {
        commands
            .entity(entity)
            .insert(PerCameraSettingsUniformOffset {
                offset: settings_uniforms
                    .uniforms
                    .push(&GridDisplaySettingsUniform {
                        x_axis_color: Vec3::from_slice(
                            &settings.x_axis_color.linear().to_f32_array(),
                        ),
                        y_axis_color: Vec3::from_slice(
                            &settings.y_axis_color.linear().to_f32_array(),
                        ),
                        minor_line_color: Vec4::from_slice(
                            &settings.minor_line_color.linear().to_f32_array(),
                        ),
                        major_line_color: Vec4::from_slice(
                            &settings.major_line_color.linear().to_f32_array(),
                        ),
                    }),
            });
    }

    position_uniforms
        .uniforms
        .write_buffer(&render_device, &render_queue);

    settings_uniforms
        .uniforms
        .write_buffer(&render_device, &render_queue);
}

fn prepare_bind_groups_for_infinite_grids(
    mut commands: Commands,
    position_uniforms: Res<InfiniteGridUniforms>,
    settings_uniforms: Res<GridDisplaySettingsUniforms>,
    pipeline: Res<InfiniteGridPipeline>,
    render_device: Res<RenderDevice>,
) {
    let bind_group = if let Some((position_binding, settings_binding)) = position_uniforms
        .uniforms
        .binding()
        .zip(settings_uniforms.uniforms.binding())
    {
        render_device.create_bind_group(
            "infinite-grid-bind-group",
            &pipeline.infinite_grid_layout,
            &BindGroupEntries::sequential((position_binding.clone(), settings_binding.clone())),
        )
    } else {
        return;
    };
    commands.insert_resource(InfiniteGridBindGroup { value: bind_group });
}

#[allow(clippy::too_many_arguments)]
fn queue_infinite_grids(
    pipeline_cache: Res<PipelineCache>,
    transparent_draw_functions: Res<DrawFunctions<Transparent2d>>,
    pipeline: Res<InfiniteGridPipeline>,
    mut pipelines: ResMut<SpecializedRenderPipelines<InfiniteGridPipeline>>,
    infinite_grids: Query<(Entity, &ExtractedInfiniteGrid)>,
    mut transparent_render_phases: ResMut<ViewSortedRenderPhases<Transparent2d>>,
    mut views: Query<(
        Entity,
        &VisibleEntities,
        &ExtractedView,
    )>,
    msaa: Res<Msaa>,
) {
    let draw_function_id = transparent_draw_functions
        .read()
        .get_id::<DrawInfiniteGrid>()
        .unwrap();

    for (view_entity, entities, view) in views.iter_mut() {
        let Some(transparent_phase) = transparent_render_phases.get_mut(&view_entity) else {
            continue;
        };

        let mesh_key = MeshPipelineKey::from_hdr(view.hdr);
        let base_pipeline = pipelines.specialize(
            &pipeline_cache,
            &pipeline,
            GridPipelineKey {
                mesh_key,
                sample_count: msaa.samples(),
            },
        );

        for entity in entities.entities.iter().flat_map(|x| x.1.iter()) {
            if let Some(infinite_grid) = infinite_grids
                .get(*entity)
                .iter()
                .filter(|(_, grid)| plane_check(&grid.transform, view.transform.translation()))
                .map(|(_, grid)| grid)
                .next()
            {
                transparent_phase.add(Transparent2d {
                    sort_key: FloatOrd(infinite_grid.transform.translation().z),
                    pipeline: base_pipeline,
                    entity: *entity,
                    draw_function: draw_function_id,
                    batch_range: 0..1,
                    extra_index: PhaseItemExtraIndex::NONE,
                });
            }
        }
    }
}

fn plane_check(plane: &GlobalTransform, point: Vec3) -> bool {
    plane.up().dot(plane.translation() - point).abs() > f32::EPSILON
}

type DrawInfiniteGrid = (
    SetItemPipeline,
    SetGridViewBindGroup<0>,
    SetInfiniteGridBindGroup<1>,
    FinishDrawInfiniteGrid,
);

#[derive(Resource)]
struct InfiniteGridPipeline {
    view_layout: BindGroupLayout,
    infinite_grid_layout: BindGroupLayout,
}

impl FromWorld for InfiniteGridPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let view_layout = render_device.create_bind_group_layout(
            Some("grid-view-bind-group-layout"),
            &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: BufferSize::new(ViewUniform::min_size().into()),
                },
                count: None,
            }],
        );
        let infinite_grid_layout = render_device.create_bind_group_layout(
            Some("infinite-grid-bind-group-layout"),
            &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: BufferSize::new(InfiniteGridUniform::min_size().into()),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: BufferSize::new(
                            GridDisplaySettingsUniform::min_size().into(),
                        ),
                    },
                    count: None,
                },
            ],
        );

        Self {
            view_layout,
            infinite_grid_layout,
        }
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct GridPipelineKey {
    mesh_key: MeshPipelineKey,
    sample_count: u32,
}

impl SpecializedRenderPipeline for InfiniteGridPipeline {
    type Key = GridPipelineKey;

    fn specialize(&self, key: Self::Key) -> RenderPipelineDescriptor {
        let format = match key.mesh_key.contains(MeshPipelineKey::HDR) {
            true => ViewTarget::TEXTURE_FORMAT_HDR,
            false => TextureFormat::bevy_default(),
        };

        RenderPipelineDescriptor {
            label: Some(Cow::from("grid-render-pipeline")),
            layout: vec![self.view_layout.clone(), self.infinite_grid_layout.clone()],
            push_constant_ranges: Vec::new(),
            vertex: VertexState {
                shader: SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: Cow::Borrowed("vertex"),
                buffers: vec![],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: bevy::render::render_resource::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: key.sample_count,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(FragmentState {
                shader: SHADER_HANDLE,
                shader_defs: vec![],
                entry_point: Cow::Borrowed("fragment"),
                targets: vec![Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
        }
    }
}

pub fn render_app_builder(app: &mut App) {
    app.world_mut()
        .resource_mut::<Assets<Shader>>()
        .get_or_insert_with(&SHADER_HANDLE, || Shader::from_wgsl(PLANE_RENDER, file!()));

    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
    render_app
        .init_resource::<ViewUniforms>()
        .init_resource::<InfiniteGridUniforms>()
        .init_resource::<GridDisplaySettingsUniforms>()
        .init_resource::<InfiniteGridPipeline>()
        .init_resource::<SpecializedRenderPipelines<InfiniteGridPipeline>>()
        .add_render_command::<Transparent2d, DrawInfiniteGrid>()
        .add_systems(
            ExtractSchedule,
            extract_infinite_grids, // order to minimize move overhead
        )
        .add_systems(ExtractSchedule, extract_per_camera_settings)
        .add_systems(Render, (prepare_infinite_grids,).in_set(RenderSet::Prepare))
        .add_systems(
            Render,
            (
                prepare_bind_groups_for_infinite_grids,
                prepare_grid_view_bind_groups,
            )
                .in_set(RenderSet::PrepareBindGroups),
        )
        .add_systems(Render, queue_infinite_grids.in_set(RenderSet::Queue));
}
