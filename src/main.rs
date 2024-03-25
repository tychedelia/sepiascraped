use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_prototype_lyon::plugin::ShapePlugin;

use crate::event::SpawnOp;
use crate::index::UniqueIndexPlugin;
use crate::op::component::ComponentPlugin;
use crate::op::material::MaterialPlugin;
use crate::op::mesh::MeshPlugin;
use crate::op::texture::TexturePlugin;
use crate::param::ParamPlugin;
use crate::render::RenderPlugin;
use crate::render_layers::{RenderLayerManager, RenderLayerPlugin};
use crate::script::ScriptPlugin;
use crate::ui::UiPlugin;

mod event;
mod index;
mod op;
mod param;
mod render;
mod render_layers;
mod script;
mod ui;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins,
        ScriptPlugin,
        ParamPlugin,
        EguiPlugin,
        RenderPlugin,
        TexturePlugin,
        MeshPlugin,
        MaterialPlugin,
        ComponentPlugin,
        UiPlugin,
        ShapePlugin,
        RenderLayerPlugin,
        UniqueIndexPlugin::<OpName>::default(),
    ))
    .add_event::<SpawnOp>()
    .configure_sets(
        Update,
        (Sets::Ui, Sets::Graph, Sets::Params, Sets::Uniforms).chain(),
    )
    .add_systems(Startup, setup);
    // bevy_mod_debugdump::print_schedule_graph(&mut app, Update);
    app.run();
}

#[derive(Component, Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OpName(pub String);

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    // commands.spawn((TextureOp, TextureOpType::<TextureOpNoise>::default()));
    // commands.spawn((TextureOp, TextureOpType::<TextureOpComposite>::default()));
    // commands.spawn((TextureOp, TextureOpType::<TextureOpComposite>::default()));
}

#[derive(SystemSet, Hash, PartialEq, Eq, Clone, Debug)]
enum Sets {
    Ui,
    Graph,
    Params,
    Uniforms,
}
