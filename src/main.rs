use crate::index::UniqueIndexPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_prototype_lyon::plugin::ShapePlugin;

use crate::param::ParamPlugin;
use crate::render::RenderPlugin;
use crate::script::ScriptPlugin;
use crate::texture::operator::composite::TextureOpComposite;
use crate::texture::operator::noise::TextureOpNoise;
use crate::texture::operator::ramp::TextureOpRamp;
use crate::texture::{TextureOp, TextureOpType, TexturePlugin};
use crate::ui::UiPlugin;

mod index;
mod param;
mod render;
mod script;
mod texture;
mod ui;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        DefaultPlugins,
        // DefaultPlugins.build().disable::<LogPlugin>(),
        ScriptPlugin,
        ParamPlugin,
        EguiPlugin,
        RenderPlugin,
        TexturePlugin,
        UiPlugin,
        ShapePlugin,
        UniqueIndexPlugin::<OpName>::default(),
    ))
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
