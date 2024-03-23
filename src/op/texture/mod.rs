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
use crate::{OpName, Sets};
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
                    selected_node_ui,
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

impl <T> Op for OpType<T>
    where T: TextureOp + Component + Clone + Debug + Send + Sync + 'static
{
    const INPUTS: usize = <T as TextureOp>::INPUTS;
    const OUTPUTS: usize = <T as TextureOp>::OUTPUTS;

    type OpType = OpType<Self>;
    type UpdateParam = (
        SQuery<(Read<Children>, Write<T::Uniform>)>,
        SQuery<(
            Read<ParamName>, Read<ParamValue>
        )>,
    );
    type BundleParam = (SResMut<Assets<Image>>);
    type Bundle = (TextureOpBundle, T::Uniform);


    fn update<'w>(entity: Entity, param: &mut SystemParamItem<'w, '_, Self::UpdateParam>) {
        let (self_q, params_q) = param;

        let (children, mut uniform) = self_q.get_mut(entity).unwrap();

        let params = children
            .iter()
            .filter_map(|entity| params_q.get(*entity).ok())
            .collect();

        T::update_uniform(&mut uniform, &params)
    }

    fn create_bundle<'w>(entity: Entity, (mut images): &mut SystemParamItem<'w, '_, Self::BundleParam>) -> Self::Bundle
    {
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

        let image = images.add(image);

        (
            TextureOpBundle {
                camera: Camera3dBundle {
                    camera_render_graph: CameraRenderGraph::new(TextureOpSubGraph),
                    camera: Camera {
                        order: 3,
                        target: image.clone().into(),
                        ..default()
                    },
                    ..default()
                },
                image: TextureOpImage(image.clone()),
                inputs: TextureOpInputs {
                    count: T::INPUTS,
                    connections: HashMap::new(),
                },
                outputs: TextureOpOutputs { count: T::OUTPUTS },
            },
            T::Uniform::default(),
        )
    }

    fn params() -> Vec<ParamBundle> {
        let common_params = vec![
            ParamBundle {
                name: ParamName("Resolution".to_string()),
                value: ParamValue::Vec2(Vec2::new(512.0, 512.0)),
                order: ParamOrder(0),
                page: ParamPage("Common".to_string()),
                ..default()
            },
            ParamBundle {
                name: ParamName("View".to_string()),
                value: ParamValue::Bool(false),
                order: ParamOrder(1),
                page: ParamPage("Common".to_string()),
                ..default()
            },
        ];

        [common_params, <T as TextureOp>::params()]
            .concat()
    }
}

fn selected_node_ui(
    mut commands: Commands,
    mut ui_state: ResMut<UiState>,
    mut egui_contexts: EguiContexts,
    selected_q: Query<&Children, With<SelectedNode>>,
    mut params_q: Query<(
        Entity,
        &ParamName,
        &mut ParamValue,
        Option<&ScriptedParamError>,
    )>,
    mut op_name_q: Query<&OpName>,
    op_name_idx: Res<UniqueIndex<OpName>>,
) {
    if let Ok(children) = selected_q.get_single() {
        ui_state.node_info = Some(
            egui::Window::new("node_info")
                .resizable(false)
                .collapsible(false)
                .movable(false)
                .show(egui_contexts.ctx_mut(), |ui| {
                    egui::Grid::new("texture_ramp_params").show(ui, |ui| {
                        ui.heading("Params");
                        ui.end_row();
                        ui.separator();
                        ui.end_row();
                        for entity in children {
                            let (param, name, mut value, script_error) =
                                params_q.get_mut(*entity).expect("Failed to get param");
                            ui.label(name.0.clone());
                            match value.as_mut() {
                                ParamValue::Color(color) => {
                                    ui.color_edit_button_rgba_premultiplied(color.as_mut());
                                }
                                ParamValue::F32(f) => {
                                    ui.add(egui::Slider::new(f, 0.0..=100.0));
                                }
                                ParamValue::Vec2(v) => {
                                    ui.add(egui::DragValue::new(&mut v.x));
                                    ui.add(egui::DragValue::new(&mut v.y));
                                }
                                ParamValue::None => {}
                                ParamValue::U32(x) => {
                                    ui.add(egui::Slider::new(x, 0..=100));
                                }
                                ParamValue::Bool(x) => {
                                    ui.checkbox(x, "");
                                }
                                ParamValue::TextureOp(x) => {

                                    let mut name = if let Some(entity) = x {
                                        let name = op_name_q.get(*entity).unwrap();
                                        name.0.clone()
                                    } else {
                                        String::new()
                                    };
                                    ui.text_edit_singleline(&mut name);
                                    if !name.is_empty() {
                                        if let Some(entity) = op_name_idx.get(&OpName(name)) {
                                            *x = Some(entity.clone());
                                        }
                                    }
                                }
                            }
                            ui.end_row();
                            if let Some(error) = script_error {
                                let prev_color = ui.visuals_mut().override_text_color;
                                ui.visuals_mut().override_text_color = Some(egui::Color32::RED);
                                ui.label(error.0.clone());
                                ui.visuals_mut().override_text_color = prev_color;
                                ui.end_row();
                            }
                        }
                    })
                })
                .unwrap()
                .inner
                .unwrap()
                .response,
        );
    }
}

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

pub trait TextureOp {
    const INPUTS: usize = 0;
    const OUTPUTS: usize = 0;

    const SHADER: &'static str;
    type Uniform: Component + ExtractComponent + ShaderType + WriteInto + Clone + Default;

    fn params() -> Vec<ParamBundle>;

    fn update_uniform(uniform: &mut Self::Uniform, params: &Vec<(&ParamName, &ParamValue)>);
}
