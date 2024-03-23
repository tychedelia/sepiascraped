use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContexts};
use bevy_mod_picking::DefaultPickingPlugins;

use camera::CameraControllerPlugin;

use crate::index::UniqueIndex;
use crate::op::texture::TextureOp;
use crate::OpName;
use crate::param::{ParamName, ParamValue, ScriptedParamError};
use crate::Sets::Ui;
use crate::ui::event::ClickNode;
use crate::ui::graph::{GraphPlugin, SelectedNode};
use crate::ui::grid::InfiniteGridPlugin;

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
        ))
        .add_event::<ClickNode>()
        .add_systems(Startup, ui_setup)
        .add_systems(Update, ui.in_set(Ui))
        .init_resource::<UiState>()
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0,
        });
    }
}

#[derive(Component, Default)]
pub struct UiCamera;

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
    commands.spawn((
        UiCamera,
        Camera2dBundle {
            transform: Transform::from_translation(Vec3::new(0.1, 0.1, 0.0)),
            ..default()
        },
        camera::CameraController::default(),
    ));
}

pub fn ui(
    mut commands: Commands,
    mut ui_state: ResMut<UiState>,
    windows: Query<&Window, With<PrimaryWindow>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut egui_contexts: EguiContexts,
) {
    let ctx = egui_contexts.ctx_mut();

    ui_state.top_panel = Some(
        egui::TopBottomPanel::top("top_panel")
            .resizable(false)
            .show(ctx, |ui| {})
            .response,
    );
}

pub fn selected_node_ui(
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
