use bevy::prelude::*;
use bevy::render::camera::CameraRenderGraph;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::render_resource::{Extent3d, ShaderType, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};
use bevy::utils::hashbrown::HashMap;
use bevy_egui::{egui, EguiContexts};
use bevy_egui::egui::{Align, CollapsingHeader};

use crate::texture::{TextureOp, TextureOpBundle, TextureOpImage, TextureOpInputs, TextureOpOutputs, TextureOpType, TextureOpUi};
use crate::texture::render::{TextureOpRender, TextureOpRenderPlugin, TextureOpSubGraph};
use crate::ui::graph::SelectedNode;
use crate::ui::UiState;

#[derive(Default)]
pub struct TextureOpRampPlugin;

impl Plugin for TextureOpRampPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<TextureOpType<TextureOpRamp>>::default(),
            TextureOpRenderPlugin::<TextureOpRampPlugin>::default(),
        ))
        .add_systems(Startup, setup);
    }
}

#[derive(Component, Clone, Default)]
pub struct TextureOpRamp;

impl TextureOpRender for TextureOpRampPlugin {
    const SHADER: &'static str = "shaders/texture/ramp.wgsl";
    type OpType = TextureOpType<TextureOpRamp>;
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

fn spawn_op(mut commands: Commands, mut images: ResMut<Assets<Image>>, added_q: Query<Entity, Added<TextureOpType<TextureOpRamp>>>) {
    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };

    // This is the texture that will be rendered to.
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

    commands.spawn((
        TextureOpBundle {
            camera: Camera3dBundle {
                camera_render_graph: CameraRenderGraph::new(TextureOpSubGraph),
                camera: Camera {
                    order: 2,
                    target: image.clone().into(),
                    ..default()
                },
                ..default()
            },
            op: TextureOp,
            image: TextureOpImage(image.clone()),
            inputs: TextureOpInputs {
                count: 0,
                connections: HashMap::new(),
            },
            outputs: TextureOpOutputs { count: 1 },
        },
        TextureRampSettings {
            color_a: Vec4::new(1.0, 0.0, 0.0, 1.0),
            color_b: Vec4::new(0.0, 0.5, 1.0, 1.0),
            mode: 2,
        },
    ));
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
