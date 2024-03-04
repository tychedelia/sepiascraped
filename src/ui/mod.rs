use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::{egui, EguiContexts};
use bevy_mod_picking::DefaultPickingPlugins;

use crate::texture::operator::composite::TextureOpComposite;
use crate::texture::operator::noise::TextureOpNoise;
use crate::texture::operator::ramp::TextureOpRamp;
use crate::texture::{TextureOp, TextureOpType};
use crate::Sets::Ui;
use camera::CameraControllerPlugin;
use crate::script::repl::ScriptRepl;

use crate::ui::event::ClickNode;
use crate::ui::graph::GraphPlugin;
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
    repl_q: Query<Entity, With<ScriptRepl>>
) {
    let ctx = egui_contexts.ctx_mut();

    if keys.just_pressed(KeyCode::Tab) {
        let window = windows.single();
        // let pos = window.cursor_position().unwrap();
        // ui_state.node_menu = Some(NodeMenuState { pos: (pos.x, pos.y)});
    }
    if keys.just_released(KeyCode::Escape) {
        ui_state.node_menu = None;
    }

    if let Some(node_menu) = &ui_state.node_menu {
        egui::Window::new("Node Info")
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .fixed_pos(node_menu.pos)
            .show(ctx, |ui| {
                if ui.button("Ramp").clicked() {
                    commands.spawn((TextureOp, TextureOpType::<TextureOpRamp>::default()));
                }
                if ui.button("Noise").clicked() {
                    commands.spawn((TextureOp, TextureOpType::<TextureOpNoise>::default()));
                }
                if ui.button("Composite").clicked() {
                    commands.spawn((TextureOp, TextureOpType::<TextureOpComposite>::default()));
                }
            });
    }

    ui_state.top_panel = Some(
        egui::TopBottomPanel::top("top_panel")
            .resizable(false)
            .show(ctx, |ui| {
                if ui.button("repl").clicked() {
                    if repl_q.is_empty() {
                        commands.spawn((ScriptRepl, ));
                    }
                }
            })
            .response,
    );
}
