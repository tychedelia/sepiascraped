use bevy::ecs::query::QueryData;
use bevy::prelude::*;
use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::render_graph::{RenderGraphApp, RenderLabel, RenderSubGraph, ViewNode};
use bevy::render::render_resource::ShaderType;
use bevy::render::texture::BevyDefault;
use bevy::utils::HashMap;
use bevy_egui::EguiContexts;

use crate::ui::event::{Connect, Disconnect};
use crate::ui::UiState;

pub mod operator;
pub mod render;

pub struct TexturePlugin;

impl Plugin for TexturePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<TextureOpImage>::default(),
            ExtractComponentPlugin::<TextureOpType>::default(),
            ExtractComponentPlugin::<TextureOpInputs>::default(),
            operator::ramp::TextureRampPlugin,
            operator::composite::TextureCompositePlugin,
        ));
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct TextureOp;

#[derive(Component, Clone, ExtractComponent, Default)]
pub struct TextureOpType(pub(crate) &'static str);

#[derive(Component, Clone, Debug, Deref, DerefMut, ExtractComponent, Default)]
pub struct TextureOpImage(pub Handle<Image>);

#[derive(Component, ExtractComponent, Clone, Default, Debug)]
pub struct TextureOpInputs {
    pub(crate) count: usize,
    pub(crate) connections: HashMap<Entity, Handle<Image>>,
}

#[derive(Component, Default)]
pub struct TextureOpOutputs {
    pub(crate) count: usize,
}

#[derive(Bundle, Default)]
pub struct TextureOpBundle {
    pub camera: Camera3dBundle,
    pub op: TextureOp,
    pub op_type: TextureOpType,
    pub image: TextureOpImage,
    pub inputs: TextureOpInputs,
    pub outputs: TextureOpOutputs,
}

#[derive(Default)]
pub struct TextureOpPlugin<P> {
    _marker: std::marker::PhantomData<P>,
}

impl<T> Plugin for TextureOpPlugin<T>
where
    T: Op + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (T::side_panel_ui, T::connect_handler, T::disconnect_handler),
        );
    }
}

pub trait Op {
    type Bundle: Bundle;
    type SidePanelQuery: QueryData;
    type ConnectOpQuery: QueryData = (&'static mut TextureOpInputs);
    type ConnectInputQuery: QueryData = (&'static TextureOpImage);
    type DisconnectOpQuery: QueryData = (&'static mut TextureOpInputs);
    type DisconnectInputQuery: QueryData = (&'static TextureOpImage);

    fn side_panel_ui(
        ui_state: ResMut<UiState>,
        egui_contexts: EguiContexts,
        selected_op: Query<Self::SidePanelQuery>,
    );

    fn connect_handler(
        ev_connect: EventReader<Connect>,
        node_q: Query<Self::ConnectOpQuery>,
        input_q: Query<Self::ConnectInputQuery>,
    );

    fn add_image_inputs(
        ev_connect: &mut EventReader<Connect>,
        op_q: &mut Query<&mut TextureOpInputs>,
        input_q: Query<&TextureOpImage>,
    ) {
        for ev in ev_connect.read() {
            if let Ok((mut input)) = op_q.get_mut(ev.input) {
                if let Ok(image) = input_q.get(ev.output) {
                    input.connections.insert(ev.output, image.0.clone());
                }
            }
        }
    }

    fn disconnect_handler(
        ev_disconnect: EventReader<Disconnect>,
        op_q: Query<Self::DisconnectOpQuery>,
        input_q: Query<Self::DisconnectInputQuery>,
    );

    fn remove_image_inputs(
        ev_disconnect: &mut EventReader<Disconnect>,
        op_q: &mut Query<&mut TextureOpInputs>,
        input_q: Query<&TextureOpImage>,
    ) {
        for ev in ev_disconnect.read() {
            if let Ok((mut input)) = op_q.get_mut(ev.input) {
                if let Ok(image) = input_q.get(ev.output) {
                    input.connections.remove(&ev.output);
                }
            }
        }
    }
}
