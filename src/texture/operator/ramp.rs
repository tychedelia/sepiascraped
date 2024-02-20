use bevy::prelude::*;
use bevy::render::extract_component::{ComponentUniforms, ExtractComponent};
use bevy::render::render_graph::{RenderLabel, RenderSubGraph};
use bevy::render::render_resource::binding_types::uniform_buffer;
use bevy::render::render_resource::{
    IntoBindGroupLayoutEntryBuilderArray, IntoBindingArray, ShaderType,
};
use bevy_egui::egui::{Align, CollapsingHeader};
use bevy_egui::{egui, EguiContexts};

use crate::texture::render::{TextureOpRender, TextureOpRenderPlugin};
use crate::texture::{Op, TextureOpInputs, TextureOpPlugin, TextureOpType};
use crate::ui::event::{Connect, Disconnect};
use crate::ui::graph::SelectedNode;
use crate::ui::UiState;

#[derive(Default)]
pub struct TextureRampPlugin;

impl Plugin for TextureRampPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TextureOpPlugin::<TextureRampPlugin>::default(),
            TextureOpRenderPlugin::<TextureRampPlugin>::default(),
        ));
    }
}

impl TextureOpRender for TextureRampPlugin {
    const SHADER: &'static str = "shaders/texture/ramp.wgsl";
    const OP_TYPE: &'static str = "ramp";
    type Uniform = TextureRampSettings;
}

impl Op for TextureRampPlugin {
    type Bundle = ();
    type SidePanelQuery = (
        Entity,
        &'static mut TextureRampSettings,
        &'static SelectedNode,
    );

    fn side_panel_ui(
        mut ui_state: ResMut<UiState>,
        mut egui_contexts: EguiContexts,
        mut selected_node: Query<Self::SidePanelQuery>,
    ) {
        let ctx = egui_contexts.ctx_mut();
        if let Ok((entity, mut settings, _selected_node)) = selected_node.get_single_mut() {
            ui_state.side_panel = Some(
                egui::SidePanel::left("texture_ramp_side_panel")
                    .resizable(false)
                    .show(ctx, |ui| {
                        egui::Grid::new("texture_ramp_params").show(ui, |ui| {
                            ui.heading("Ramp");
                            ui.end_row();
                            ui.separator();
                            ui.end_row();

                            let collapse = ui
                                .with_layout(egui::Layout::left_to_right(Align::Min), |ui| {
                                    ui.set_max_width(100.0);
                                    let collapse = CollapsingHeader::new("").show(ui, |ui| {});
                                    ui.label("Color A");
                                    ui.color_edit_button_rgba_premultiplied(
                                        settings.color_a.as_mut(),
                                    );
                                    collapse
                                })
                                .inner;
                            if collapse.fully_open() {
                                ui.end_row();
                                ui.add(
                                    egui::TextEdit::singleline(&mut String::new())
                                        .hint_text("Write something here"),
                                );
                            }

                            ui.end_row();

                            ui.label("Color B");
                            ui.color_edit_button_rgba_premultiplied(settings.color_b.as_mut());
                            ui.end_row();
                            let mut mode =
                                TextureRampMode::from_u32(settings.mode).expect("Invalid mode");
                            egui::ComboBox::from_label("Mode")
                                .selected_text(format!("{mode:?}"))
                                .show_ui(ui, |ui| {
                                    ui.set_min_width(60.0);
                                    ui.selectable_value(
                                        &mut mode,
                                        TextureRampMode::Horizontal,
                                        "Horizontal",
                                    );
                                    ui.selectable_value(
                                        &mut mode,
                                        TextureRampMode::Vertical,
                                        "Vertical",
                                    );
                                    ui.selectable_value(
                                        &mut mode,
                                        TextureRampMode::Circular,
                                        "Circular",
                                    );
                                });
                            ui.end_row();
                            settings.mode = mode.as_u32();
                        });
                    })
                    .response,
            );
        }
    }

    fn connect_handler(
        mut ev_connect: EventReader<Connect>,
        mut node_q: Query<Self::ConnectOpQuery>,
        input_q: Query<Self::ConnectInputQuery>,
    ) {
        Self::add_image_inputs(&mut ev_connect, &mut node_q, input_q);
    }

    fn disconnect_handler(
        mut ev_disconnect: EventReader<Disconnect>,
        mut node_q: Query<Self::DisconnectOpQuery>,
        input_q: Query<Self::DisconnectInputQuery>,
    ) {
        Self::remove_image_inputs(&mut ev_disconnect, &mut node_q, input_q);
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
