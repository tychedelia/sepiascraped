use bevy::core::FrameCount;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, SystemInformationDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::render::camera::CameraOutputMode;
use bevy_egui::egui::util::cache::FrameCache;
use bevy_egui::egui::TextEdit;
use bevy_egui::{egui, EguiContexts};
use bevy_mod_picking::DefaultPickingPlugins;
use egui_autocomplete::AutoCompleteTextEdit;
use iyes_perf_ui::prelude::PerfUiEntryFPS;
use iyes_perf_ui::{PerfUiCompleteBundle, PerfUiPlugin, PerfUiRoot};
use std::collections::BTreeSet;
use steel_parser::ast::IteratorExtensions;

use camera::CameraControllerPlugin;

use crate::index::{Index, IndexPlugin, UniqueIndex};
use crate::op::texture::TextureOp;
use crate::op::{OpCategory, OpTypeName};
use crate::param::{ParamName, ParamValue, ScriptedParam, ScriptedParamError};
use crate::ui::event::ClickNode;
use crate::ui::graph::{GraphPlugin, SelectedNode};
use crate::ui::grid::InfiniteGridPlugin;
use crate::OpName;
use crate::Sets::Ui;

mod camera;
pub mod event;
pub mod graph;
pub mod grid;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            GraphPlugin,
            CameraControllerPlugin,
            InfiniteGridPlugin,
            DefaultPickingPlugins,
            IndexPlugin::<OpCategory>::default(),
            FrameTimeDiagnosticsPlugin,
            SystemInformationDiagnosticsPlugin,
            PerfUiPlugin,
        ))
        .add_event::<ClickNode>()
        .add_systems(Startup, ui_setup)
        .add_systems(
            Update,
            (toggle_camera, init_params, ui, selected_node_ui).in_set(Ui),
        )
        .init_resource::<UiState>()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0,
        });
    }
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Components
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[derive(Component, Default)]
pub struct UiCamera;

#[derive(Component, Default, Deref, DerefMut)]
pub struct UiText(pub String);

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Resources
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

#[derive(Resource, Default)]
pub struct UiState {
    pub top_panel: Option<egui::Response>,
    pub node_info: Option<egui::Response>,
    pub node_menu: Option<NodeMenuState>,
}

pub struct NodeMenuState {
    pub pos: (f32, f32),
}

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// Systems
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

pub fn ui_setup(mut commands: Commands) {
    commands.spawn((PerfUiCompleteBundle::default()));
    commands.spawn((
        UiCamera,
        Camera2dBundle {
            transform: Transform::from_translation(Vec3::new(0.1, 0.1, 0.0)),
            ..default()
        },
        camera::CameraController::default(),
    ));
}

fn toggle_camera(
    keys: Res<ButtonInput<KeyCode>>,
    mut selected_q: Query<(&mut Camera), With<SelectedNode>>,
) {
    if keys.just_pressed(KeyCode::KeyC) {
        let (mut camera) = selected_q.single_mut();
        match camera.output_mode {
            CameraOutputMode::Skip => {
                camera.output_mode = CameraOutputMode::default();
            }
            CameraOutputMode::Write { .. } => {
                camera.output_mode = CameraOutputMode::Skip;
            }
        }
    }
}

pub fn ui(
    mut time: ResMut<Time<Virtual>>,
    frame_count: Res<FrameCount>,
    mut ui_state: ResMut<UiState>,
    keys: Res<ButtonInput<KeyCode>>,
    mut egui_contexts: EguiContexts,
) {
    let ctx = egui_contexts.ctx_mut();

    if keys.just_pressed(KeyCode::Space) {
        if time.is_paused() {
            time.unpause();
        } else {
            time.pause();
        }
    }

    ui_state.top_panel = Some(
        egui::TopBottomPanel::top("top_panel")
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("Time: {:.2}", time.elapsed_seconds()));
                    ui.label(format!("Frames: {:.2}", frame_count.0));
                });
            })
            .response,
    );
}

pub fn init_params(
    mut commands: Commands,
    params_q: Query<(Entity, &ParamValue), Added<ParamValue>>,
) {
    for (entity, value) in params_q.iter() {
        match value {
            ParamValue::TextureOp(_) | ParamValue::MeshOp(_) => {
                commands.entity(entity).insert(UiText(String::new()));
            }
            _ => {}
        }
    }
}

pub fn selected_node_ui(
    mut commands: Commands,
    mut ui_state: ResMut<UiState>,
    mut egui_contexts: EguiContexts,
    selected_q: Query<(&Children, &OpTypeName), With<SelectedNode>>,
    mut params_q: Query<(
        Entity,
        &ParamName,
        &mut ParamValue,
        Has<ScriptedParam>,
        Option<&ScriptedParamError>,
        Option<&mut UiText>,
    )>,
    mut op_name_q: Query<&OpName>,
    op_name_idx: Res<UniqueIndex<OpName>>,
    category_idx: Res<Index<OpCategory>>,
) {
    if let Ok((children, op_type_name)) = selected_q.get_single() {
        ui_state.node_info = Some(
            egui::Window::new(&op_type_name.0)
                .anchor(egui::Align2::LEFT_TOP, egui::Vec2::new(10.0, 30.0))
                .resizable(false)
                .collapsible(false)
                .movable(false)
                .show(egui_contexts.ctx_mut(), |ui| {
                    egui::Grid::new("op_params")
                        .min_col_width(100.0)
                        .show(ui, |ui| {
                            ui.heading("Params");
                            ui.end_row();
                            ui.separator();
                            ui.end_row();
                            for entity in children {
                                let (param, name, mut value, is_scripted, script_error, ui_text) =
                                    params_q.get_mut(*entity).expect("Failed to get param");
                                ui.label(name.0.to_string() + if is_scripted { " *" } else { "" });

                                match value.as_mut() {
                                    ParamValue::Color(color) => {
                                        ui.add_enabled_ui(!is_scripted, |ui| {
                                            ui.color_edit_button_rgba_premultiplied(color.as_mut())
                                        });
                                    }
                                    ParamValue::F32(f) => {
                                        ui.add_enabled_ui(!is_scripted, |ui| {
                                            ui.add(egui::Slider::new(f, 0.0..=100.0))
                                        });
                                    }
                                    ParamValue::Vec2(v) => {
                                        ui.add_enabled_ui(!is_scripted, |ui| {
                                            ui.add(
                                                egui::DragValue::new(&mut v.x)
                                                    .clamp_range(0.0..=1.0)
                                                    .speed(0.05),
                                            );
                                            ui.add(
                                                egui::DragValue::new(&mut v.y)
                                                    .clamp_range(0.0..=1.0)
                                                    .speed(0.05),
                                            );
                                        });
                                    }
                                    ParamValue::None => {}
                                    ParamValue::U32(x) => {
                                        ui.add_enabled_ui(!is_scripted, |ui| {
                                            ui.add(egui::Slider::new(x, 0..=100))
                                        });
                                    }
                                    ParamValue::Bool(x) => {
                                        ui.add_enabled_ui(!is_scripted, |ui| ui.checkbox(x, ""));
                                    }
                                    ParamValue::TextureOp(x) => {
                                        let mut ui_text = ui_text.expect("Failed to get ui_text");

                                        if let Some(entity) = x {
                                            let name = op_name_q.get(*entity).unwrap();
                                            *ui_text = UiText(name.0.clone());
                                        };
                                        ui.add_enabled_ui(!is_scripted, |ui| {
                                            // TODO: compute this in resource
                                            let inputs = category_idx
                                                .get(&OpCategory(crate::op::texture::CATEGORY))
                                                .unwrap_or(&vec![])
                                                .iter()
                                                .map(|e| op_name_q.get(*e).unwrap().0.clone())
                                                .collect::<BTreeSet<_>>();
                                            ui.add(
                                                AutoCompleteTextEdit::new(&mut ui_text, inputs)
                                                    .max_suggestions(5)
                                                    .highlight_matches(true),
                                            );
                                        });
                                        if !ui_text.0.is_empty() {
                                            if let Some(entity) =
                                                op_name_idx.get(&OpName(ui_text.0.clone()))
                                            {
                                                *x = Some(entity.clone());
                                            }
                                        }
                                    }
                                    ParamValue::MeshOp(x) => {
                                        let mut ui_text = ui_text.expect("Failed to get ui_text");

                                        if let Some(entity) = x {
                                            let name = op_name_q.get(*entity).unwrap();
                                            *ui_text = UiText(name.0.clone());
                                        };
                                        ui.add_enabled_ui(!is_scripted, |ui| {
                                            // TODO: compute this in resource
                                            let inputs = category_idx
                                                .get(&OpCategory(crate::op::mesh::CATEGORY))
                                                .unwrap_or(&vec![])
                                                .iter()
                                                .map(|e| op_name_q.get(*e).unwrap().0.clone())
                                                .collect::<BTreeSet<_>>();
                                            ui.add(
                                                AutoCompleteTextEdit::new(&mut ui_text, inputs)
                                                    .max_suggestions(5)
                                                    .highlight_matches(true),
                                            );
                                        });
                                        if !ui_text.0.is_empty() {
                                            if let Some(entity) =
                                                op_name_idx.get(&OpName(ui_text.0.clone()))
                                            {
                                                *x = Some(entity.clone());
                                            }
                                        }
                                    }
                                    ParamValue::Vec3(v) => {
                                        ui.add_enabled_ui(!is_scripted, |ui| {
                                            ui.add(
                                                egui::DragValue::new(&mut v.x)
                                                    .clamp_range(-10.0..=10.0)
                                                    .speed(0.05),
                                            );
                                            ui.add(
                                                egui::DragValue::new(&mut v.y)
                                                    .clamp_range(-10.0..=10.0)
                                                    .speed(0.05),
                                            );
                                            ui.add(
                                                egui::DragValue::new(&mut v.z)
                                                    .clamp_range(-10.0..=10.0)
                                                    .speed(0.05),
                                            );
                                        });
                                    }
                                    ParamValue::Quat(v) => {
                                        ui.add_enabled_ui(!is_scripted, |ui| {
                                            ui.add(
                                                egui::DragValue::new(&mut v.x)
                                                    .clamp_range(-10.0..=10.0)
                                                    .speed(0.05),
                                            );
                                            ui.add(
                                                egui::DragValue::new(&mut v.y)
                                                    .clamp_range(-10.0..=10.0)
                                                    .speed(0.05),
                                            );
                                            ui.add(
                                                egui::DragValue::new(&mut v.z)
                                                    .clamp_range(-10.0..=10.0)
                                                    .speed(0.05),
                                            );
                                            ui.add(
                                                egui::DragValue::new(&mut v.w)
                                                    .clamp_range(0.0..=1.0)
                                                    .speed(0.05),
                                            );
                                        });
                                    }
                                    ParamValue::UVec2(x) => {
                                        ui.add_enabled_ui(!is_scripted, |ui| {
                                            ui.add(
                                                egui::DragValue::new(&mut x.x)
                                                    .clamp_range(0..=10000)
                                                    .speed(10.0),
                                            );
                                            ui.add(
                                                egui::DragValue::new(&mut x.y)
                                                    .clamp_range(0..=10000)
                                                    .speed(10.0),
                                            );
                                        });
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
