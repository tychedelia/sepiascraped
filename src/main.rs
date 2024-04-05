use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::view::RenderLayers;
use bevy::window::WindowRef;
use iyes_perf_ui::prelude::*;
use engine::op::OpName;
use crate::engine::op::{OpImage, OpInputs, OpOutputs};

use crate::engine::SepiascrapedEnginePlugin;
use crate::index::UniqueIndexPlugin;
use crate::render_layers::{RenderLayerManager, RenderLayerPlugin};
use crate::ui::SepiascrapedUiPlugin;

mod index;
mod render_layers;
mod ui;
mod engine;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins,
        SepiascrapedUiPlugin,
        SepiascrapedEnginePlugin,
        RenderLayerPlugin,
    ))
        // .add_systems(Startup, startup)
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
    );
    // bevy_mod_debugdump::print_schedule_graph(&mut app, Update);
    app.run();
}

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

fn startup(
    mut commands: Commands, mut layer_manager: ResMut<RenderLayerManager>
) {
        let window=  commands.spawn(Window {
            title: "foo".to_string(),
            ..default()
        }).id();
        commands.spawn((
            Camera2dBundle {
                camera: Camera {
                    target: RenderTarget::Window(WindowRef::Entity(window)),
                    ..default()
                },
                ..default()
            },
            RenderLayers::from_layer(layer_manager.next_open_layer()),
            OpImage::default(),
            OpInputs::default(),
            OpOutputs::default(),
        ));
    }