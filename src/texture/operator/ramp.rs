use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::render_resource::ShaderType;
use bevy_egui::egui::{Align, CollapsingHeader};
use bevy_egui::{egui, EguiContexts};

use crate::texture::render::TextureOpRenderPlugin;
use crate::texture::{spawn_op, TextureOpMeta, TextureOpType, TextureOpUi};
use crate::ui::graph::SelectedNode;
use crate::ui::UiState;

#[derive(Default)]
pub struct TextureOpRampPlugin;

impl Plugin for TextureOpRampPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<TextureOpType<TextureOpRamp>>::default(),
            TextureOpRenderPlugin::<TextureOpRamp>::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, spawn_op::<TextureOpRamp>);
    }
}

#[derive(Component, Clone, Default, Debug)]
pub struct TextureOpRamp;

impl TextureOpMeta for TextureOpRamp {
    const SHADER: &'static str = "shaders/texture/ramp.wgsl";
    const INPUTS: usize = 0;
    const OUTPUTS: usize = 1;
    type Uniform = TextureRampSettings;
}

fn side_panel_ui(
    mut ui_state: ResMut<UiState>,
    mut egui_contexts: EguiContexts,
    mut selected_node: Query<(Entity, &mut TextureRampSettings, &SelectedNode)>,
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
                                ui.color_edit_button_rgba_premultiplied(settings.color_a.as_mut());
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

fn setup(world: &mut World) {
    let cb = world.register_system(side_panel_ui);
    world.spawn((TextureOpType::<TextureOpRamp>::default(), TextureOpUi(cb)));
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
