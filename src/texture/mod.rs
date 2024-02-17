use bevy::ecs::query::QueryData;
use bevy::prelude::*;
use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::render_graph::{RenderGraphApp, RenderLabel, RenderSubGraph, ViewNode};
use bevy::render::render_resource::ShaderType;
use bevy::render::texture::BevyDefault;
use bevy_egui::EguiContexts;

use crate::ui::event::{Connect, Disconnect};
use crate::ui::UiState;

pub mod node;
pub mod render;

pub struct TexturePlugin;

impl Plugin for TexturePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<TextureNodeImage>::default(),
            ExtractComponentPlugin::<TextureNodeType>::default(),
            ExtractComponentPlugin::<TextureNodeInputs>::default(),
            node::ramp::TextureRampPlugin,
            node::composite::CompositePlugin,
        ));
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct TextureNode;

#[derive(Component, Clone, ExtractComponent, Default)]
pub struct TextureNodeType(pub String);

#[derive(Component, Clone, Debug, Deref, DerefMut, ExtractComponent, Default)]
pub struct TextureNodeImage(pub Handle<Image>);

#[derive(Component, ExtractComponent, Clone, Default)]
pub struct TextureNodeInputs {
    pub(crate) count: usize,
    pub(crate) connections: Vec<Handle<Image>>,
}

#[derive(Component, Default)]
pub struct TextureNodeOutputs {
    pub(crate) count: usize,
}

#[derive(Bundle, Default)]
pub struct TextureNodeBundle {
    pub camera: Camera3dBundle,
    pub node: TextureNode,
    pub node_type: TextureNodeType,
    pub image: TextureNodeImage,
    pub inputs: TextureNodeInputs,
    pub outputs: TextureNodeOutputs,
}

#[derive(Default)]
pub struct TextureNodePlugin<P> {
    _marker: std::marker::PhantomData<P>,
}

impl<T> Plugin for TextureNodePlugin<T>
where
    T: NodeType + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (T::side_panel_ui, T::connect_handler, T::disconnect_handler),
        );
    }
}

pub trait NodeType {
    type Bundle: Bundle;
    type SidePanelQuery: QueryData;
    type ConnectNodeQuery: QueryData = (&'static mut TextureNodeInputs);
    type ConnectInputQuery: QueryData = (&'static TextureNodeImage);
    type DisconnectNodeQuery: QueryData = (&'static mut TextureNodeInputs);
    type DisconnectInputQuery: QueryData = (&'static TextureNodeImage);

    fn side_panel_ui(
        ui_state: ResMut<UiState>,
        egui_contexts: EguiContexts,
        selected_node: Query<Self::SidePanelQuery>,
    );

    fn connect_handler(
        ev_connect: EventReader<Connect>,
        node_q: Query<Self::ConnectNodeQuery>,
        input_q: Query<Self::ConnectInputQuery>,
    );

    fn add_image_inputs(
        ev_connect: &mut EventReader<Connect>,
        node_q: &mut Query<&mut TextureNodeInputs>,
        input_q: Query<&TextureNodeImage>,
    ) {
        for ev in ev_connect.read() {
            if let Ok((mut input)) = node_q.get_mut(ev.input) {
                let input: &mut Mut<TextureNodeInputs> = &mut input;
                if let Ok(image) = input_q.get(ev.output) {
                    input.connections.push(image.0.clone());
                }
            }
        }
    }

    fn disconnect_handler(
        ev_disconnect: EventReader<Disconnect>,
        node_q: Query<Self::DisconnectNodeQuery>,
        input_q: Query<Self::DisconnectInputQuery>,
    );

    fn remove_image_inputs(
        ev_disconnect: &mut EventReader<Disconnect>,
        node_q: &mut Query<&mut TextureNodeInputs>,
        input_q: Query<&TextureNodeImage>,
    ) {
        for ev in ev_disconnect.read() {
            if let Ok((mut input)) = node_q.get_mut(ev.input) {
                if let Ok(image) = input_q.get(ev.output) {
                    input.connections = input
                        .connections
                        .iter()
                        .filter(|i| *i != &image.0)
                        .cloned()
                        .collect();
                }
            }
        }
    }
}
