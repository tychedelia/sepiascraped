use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::render_graph::{RenderLabel, RenderSubGraph};
use bevy::render::render_resource::ShaderType;
use bevy_egui::{egui, EguiContexts};

use crate::texture::render::TextureOpRenderPlugin;
use crate::texture::{spawn_op, TextureOpMeta, TextureOpType, TextureOpUi};
use crate::ui::graph::SelectedNode;
use crate::ui::UiState;

#[derive(Default)]
pub struct TextureOpCompositePlugin;

impl Plugin for TextureOpCompositePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<TextureOpType<TextureOpComposite>>::default(),
            TextureOpRenderPlugin::<TextureOpComposite>::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, spawn_op::<TextureOpComposite>);
    }
}

impl TextureOpMeta for TextureOpComposite {
    const SHADER: &'static str = "shaders/texture/composite.wgsl";
    const INPUTS: usize = 2;
    const OUTPUTS: usize = 0;
    type Uniform = CompositeSettings;
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

fn setup(world: &mut World) {
    let cb = world.register_system(side_panel_ui);
    world.spawn((
        TextureOpType::<TextureOpComposite>::default(),
        TextureOpUi(cb),
    ));
}

#[derive(Component, Clone, Default, Debug)]
pub struct TextureOpComposite;

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

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderSubGraph)]
struct CompositeSubGraph;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct CompositeLabel;
