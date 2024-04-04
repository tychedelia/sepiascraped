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
use bevy::diagnostic::{
    EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, SystemInformationDiagnosticsPlugin,
};
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_prototype_lyon::plugin::ShapePlugin;
use iyes_perf_ui::prelude::*;

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
        (
            Sets::Ui,
            Sets::Script,
            Sets::Spawn,
            Sets::Graph,
            Sets::Params,
            Sets::Execute,
        )
            .chain(),
    )
    .add_systems(Startup, setup);
    bevy_mod_debugdump::print_schedule_graph(&mut app, Update);
    app.run();
}

#[derive(Component, Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OpName(pub String);

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {}

/// The system sets for the application.
#[derive(SystemSet, Hash, PartialEq, Eq, Clone, Debug)]
enum Sets {
    /// Ui updates and events
    Ui,
    /// Run all scripts
    Script,
    /// Spawn ops
    Spawn,
    /// Updates to the op graph
    Graph,
    /// Param related behavior, i.e. preparing an op to execute
    Params,
    /// Execute ops
    Execute,
}
