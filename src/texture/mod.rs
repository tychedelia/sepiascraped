use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Deref;

use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemId;
use bevy::prelude::*;
use bevy::render::camera::CameraRenderGraph;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::render_resource::encase::internal::WriteInto;
use bevy::render::render_resource::{
    Extent3d, ShaderType, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::sprite::Material2d;
use bevy::utils::HashMap;
use bevy_egui::egui::{Align, CollapsingHeader};
use bevy_egui::{egui, EguiContexts};

use operator::composite::TextureOpCompositePlugin;
use operator::ramp::TextureOpRampPlugin;

use crate::index::UniqueIndexPlugin;
use crate::param::{
    ParamBundle, ParamName, ParamOrder, ParamValue, ScriptedParam, ScriptedParamValue,
};
use crate::texture::event::SpawnOp;
use crate::texture::operator::composite::CompositeMode;
use crate::texture::operator::ramp::{TextureRampMode, TextureRampSettings};
use crate::texture::render::TextureOpSubGraph;
use crate::ui::event::{Connect, Disconnect};
use crate::ui::graph::{GraphRef, NodeMaterial, OpRef, SelectedNode};
use crate::ui::UiState;
use crate::OpName;

mod event;
pub mod operator;
pub mod render;

pub struct TexturePlugin;

impl Plugin for TexturePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnOp>()
            .add_plugins((
                ExtractComponentPlugin::<TextureOpImage>::default(),
                ExtractComponentPlugin::<TextureOpInputs>::default(),
                TextureOpPlugin,
                TextureOpRampPlugin,
                TextureOpCompositePlugin,
            ))
            .add_systems(Startup, setup);
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

fn spawn_op<T>(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    added_q: Query<Entity, (With<TextureOp>, Added<TextureOpType<T>>)>,
    existing_q: Query<Entity, (With<TextureOp>, With<TextureOpType<T>>)>,
    mut spawn_op_evt: EventWriter<SpawnOp>,
) where
    T: TextureOpMeta + Debug + Send + Sync + 'static,
{
    let mut count = existing_q.iter().len();
    for entity in added_q.iter() {
        count = count + 1;

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

        commands
            .entity(entity)
            .insert((
                OpName(format!("{}{}", TextureOpType::<T>::name(), count)),
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
            ))
            .with_children(|parent| {
                T::params().into_iter().for_each(|param| {
                    parent.spawn((OpRef(parent.parent_entity()), param));
                });
            });
        spawn_op_evt.send(SpawnOp(entity));
    }
}

#[derive(Resource, Clone, Default)]
pub struct TextureOpDefaultImage(pub Handle<Image>);

#[derive(Component, Clone, Copy, Default)]
pub struct TextureOp;

#[derive(Component, Clone, ExtractComponent, Default, Debug)]
pub struct TextureOpType<T: Debug + Sync + Send + 'static>(PhantomData<T>);

impl<T> TextureOpType<T>
where
    T: Debug + Sync + Send + 'static,
{
    pub fn name() -> &'static str {
        std::any::type_name::<T>().split("::").nth(3).unwrap()
    }
}

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

#[derive(Default)]
pub struct TextureOpPlugin;

impl Plugin for TextureOpPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                side_panel_ui,
                update_materials,
                connect_handler,
                disconnect_handler,
            ),
        );
    }
}

fn update_uniform<T>(
    mut selected_node_q: Query<(&Children, &mut T::Uniform), With<SelectedNode>>,
    mut params_q: Query<(&ParamName, &ParamValue)>,
) where
    T: TextureOpMeta,
{
    if let Ok((children, mut uniform)) = selected_node_q.get_single_mut() {
        let params = children
            .iter()
            .filter_map(|entity| params_q.get(*entity).ok())
            .collect();
        T::update_uniform(&mut uniform, &params);
    }
}

fn side_panel_ui(
    mut commands: Commands,
    mut ui_state: ResMut<UiState>,
    mut egui_contexts: EguiContexts,
    selected_q: Query<&Children, With<SelectedNode>>,
    mut params_q: Query<(
        Entity,
        &ParamName,
        &mut ParamValue,
        Option<&mut ScriptedParamValue>,
    )>,
) {
    if let Ok(children) = selected_q.get_single() {
        ui_state.side_panel = Some(
            egui::SidePanel::left("side_panel")
                .resizable(false)
                .show(egui_contexts.ctx_mut(), |ui| {
                    egui::Grid::new("texture_ramp_params").show(ui, |ui| {
                        ui.heading("Params");
                        ui.end_row();
                        ui.separator();
                        ui.end_row();
                        for entity in children {
                            let (param, name, mut value, mut scripted_value) =
                                params_q.get_mut(*entity).expect("Failed to get param");
                            match value.as_mut() {
                                ParamValue::Color(color) => {
                                    let collapse = ui
                                        .with_layout(
                                            egui::Layout::left_to_right(Align::Min),
                                            |ui| {
                                                ui.set_max_width(100.0);
                                                let collapse =
                                                    CollapsingHeader::new(name.0.clone())
                                                        .show(ui, |ui| {});
                                                ui.color_edit_button_rgba_premultiplied(
                                                    color.as_mut(),
                                                );
                                                collapse
                                            },
                                        )
                                        .inner;
                                    if collapse.fully_open() {
                                        ui.end_row();
                                        if let Some(mut scripted_value) = scripted_value {
                                            ui.add(egui::TextEdit::singleline(
                                                &mut scripted_value.0,
                                            ));
                                        } else {
                                            let mut s = String::new();
                                            ui.add(egui::TextEdit::singleline(&mut s));
                                            if !s.is_empty() {
                                                commands
                                                    .entity(param)
                                                    .insert((ScriptedParam, ScriptedParamValue(s)));
                                            }
                                        };
                                    }
                                    ui.end_row();
                                }
                                _ => {}
                            }
                        }
                    });
                })
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

pub trait TextureOpMeta: Debug + Clone + Send + Sync + 'static {
    const SHADER: &'static str;
    const INPUTS: usize;
    const OUTPUTS: usize;
    type OpType: Debug + Component + ExtractComponent + Send + Sync + 'static = TextureOpType<Self>;
    type Uniform: Component + ExtractComponent + ShaderType + WriteInto + Clone + Default;

    fn params() -> Vec<ParamBundle>;

    fn update_uniform(uniform: &mut Self::Uniform, params: &Vec<(&ParamName, &ParamValue)>);
}
