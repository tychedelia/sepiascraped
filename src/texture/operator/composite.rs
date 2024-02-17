use bevy::prelude::*;
use bevy::render::extract_component::{ComponentUniforms, ExtractComponent};
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_graph::{RenderLabel, RenderSubGraph};
use bevy::render::render_resource::binding_types::{sampler, texture_2d, uniform_buffer};
use bevy::render::render_resource::{
    IntoBindGroupLayoutEntryBuilderArray, IntoBindingArray, SamplerBindingType, ShaderType,
    TextureSampleType,
};
use bevy_egui::{egui, EguiContexts};

use crate::texture::render::{TextureOpRenderNode, TextureOpRenderPlugin};
use crate::texture::{Op, TextureOpBundle, TextureOpInputs, TextureOpPlugin};
use crate::ui::event::{Connect, Disconnect};
use crate::ui::graph::SelectedNode;
use crate::ui::UiState;

#[derive(Default)]
pub struct CompositePlugin;

impl Plugin for CompositePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            TextureOpPlugin::<CompositePlugin>::default(),
            TextureOpRenderPlugin::<CompositePlugin, 5>::default(),
        ));
    }
}

impl TextureOpRenderNode<5> for CompositePlugin {
    const SHADER: &'static str = "shaders/texture/composite.wgsl";
    type Uniform = CompositeSettings;

    fn render_sub_graph() -> impl RenderSubGraph {
        CompositeSubGraph
    }

    fn render_label() -> impl RenderLabel {
        CompositeLabel
    }

    fn bind_group_layout_entries() -> impl IntoBindGroupLayoutEntryBuilderArray<5> {
        (
            uniform_buffer::<CompositeSettings>(true),
            texture_2d(TextureSampleType::Float { filterable: true }),
            sampler(SamplerBindingType::Filtering),
            texture_2d(TextureSampleType::Float { filterable: true }),
            sampler(SamplerBindingType::Filtering),
        )
    }

    fn bind_group_entries<'a>(
        inputs: &'a TextureOpInputs,
        world: &'a World,
    ) -> impl IntoBindingArray<'a, 5> {
        let settings_uniforms = world.resource::<ComponentUniforms<CompositeSettings>>();
        let settings_binding = settings_uniforms.uniforms().binding().unwrap();
        let images = world.resource::<RenderAssets<Image>>();
        let inputs = inputs
            .connections
            .iter()
            .map(|(k, v)| v.clone())
            .collect::<Vec<Handle<Image>>>();
        (
            settings_binding.clone(),
            &images.get(&inputs[0]).unwrap().texture_view,
            &images.get(&inputs[0]).unwrap().sampler,
            &images.get(&inputs[1]).unwrap().texture_view,
            &images.get(&inputs[1]).unwrap().sampler,
        )
    }
}

#[derive(Bundle, Default)]
pub struct CompositeNodeBundle {
    node: TextureOpBundle,
    settings: CompositeSettings,
}

impl Op for CompositePlugin {
    type Bundle = ();
    type SidePanelQuery = (
        Entity,
        &'static mut CompositeSettings,
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
                egui::SidePanel::left("composite_side_panel")
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.heading("Composite");
                        let mut mode =
                            CompositeMode::from_u32(settings.mode).expect("Invalid mode");
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
