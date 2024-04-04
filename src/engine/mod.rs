use bevy::prelude::*;

pub mod op;
pub mod param;
pub mod render;
pub mod script;
pub mod graph;

pub struct SepiascrapedEnginePlugin;

impl Plugin for SepiascrapedEnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            script::ScriptPlugin,
            param::ParamPlugin,
            graph::GraphPlugin,
            render::RenderPlugin,
            op::OpsPlugin,
        ));
    }
}