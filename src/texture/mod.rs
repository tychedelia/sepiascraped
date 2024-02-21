use std::marker::PhantomData;

use bevy::ecs::query::QueryData;
use bevy::ecs::system::SystemId;
use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::mesh::MeshVertexAttribute;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::sprite::{Material2d, Mesh2dHandle};
use bevy::ui::AlignSelf::Start;
use bevy::utils::HashMap;
use bevy_egui::EguiContexts;

use operator::composite::TextureOpCompositePlugin;
use operator::ramp::TextureOpRampPlugin;

use crate::ui::event::{Connect, Disconnect};
use crate::ui::graph::{NodeMaterial, SelectedNode};
use crate::ui::UiState;

pub mod operator;
pub mod render;

pub struct TexturePlugin;

impl Plugin for TexturePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<TextureOpImage>::default(),
            ExtractComponentPlugin::<TextureOpInputs>::default(),
            TextureOpRampPlugin,
            TextureOpCompositePlugin,
        ))
        .add_systems(Startup, startup);
    }
}

fn startup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
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

    /// All black
    image.resize(size);

    let image = images.add(image);
    commands.insert_resource(TextureOpDefaultImage(image));
}

#[derive(Resource, Clone, Default)]
pub struct TextureOpDefaultImage(pub Handle<Image>);

#[derive(Component, Clone, Copy, Default)]
pub struct TextureOp;

#[derive(Component, Clone, ExtractComponent, Default)]
pub struct TextureOpType<T: Sync + Send + 'static>(PhantomData<T>);

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

#[derive(Component)]
pub struct TextureOpUi(pub SystemId);

#[derive(Bundle, Default)]
pub struct TextureOpBundle {
    pub camera: Camera3dBundle,
    pub op: TextureOp,
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
            (side_panel_ui, connect_handler, disconnect_handler),
        );
    }
}

fn side_panel_ui(
    mut commands: Commands,
    selected_q: Query<&TextureOpUi, With<SelectedNode>>
) {
    let ui = selected_q.single();
    commands.run_system(ui.0);
}

fn connect_handler(
    mut ev_connect: EventReader<Connect>,
    mut op_q: Query<(&mut TextureOpInputs, &TextureOpImage, &Handle<NodeMaterial>)>,
    input_q: Query<&TextureOpImage>,
    mut materials: ResMut<Assets<NodeMaterial>>,
) {
    for ev in ev_connect.read() {
        if let Ok((mut input, my_image, material)) = op_q.get_mut(ev.input) {
            if let Ok(image) = input_q.get(ev.output) {
                info!("I'm connecting");
                input.connections.insert(ev.output, image.0.clone());
                if input.is_fully_connected() {
                    info!("I'm fully connected");
                    if let Some(mut material) = materials.get_mut(material) {
                        material.color_texture = my_image.0.clone();
                    }
                }
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