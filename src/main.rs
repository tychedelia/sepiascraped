use bevy::prelude::*;
use iyes_perf_ui::prelude::*;

use crate::engine::SepiascrapedEnginePlugin;
use crate::index::UniqueIndexPlugin;
use crate::render_layers::RenderLayerPlugin;
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
        UniqueIndexPlugin::<OpName>::default(),
    ))
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

#[derive(Component, Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OpName(pub String);

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
