use crate::engine::graph::event::{Connect, Disconnect};
use crate::engine::op::texture::types::composite::TextureOpComposite;
use crate::engine::op::{
    Op, OpExecute, OpOnConnect, OpOnDisconnect, OpPlugin, OpShouldExecute, OpSpawn, OpType,
    OpUpdate,
};
use crate::engine::param::{ParamBundle, ParamName, ParamValue};
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;

pub struct TextureOpRenderPlugin;

impl Plugin for TextureOpRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(OpPlugin::<TextureOpComposite>::default());
    }
}

#[derive(Component, ExtractComponent, Clone, Default, Debug)]
pub struct TextureOpRender;

impl OpSpawn for TextureOpRender {
    type Param = ();
    type Bundle = ();

    fn params(bundle: &Self::Bundle) -> Vec<ParamBundle> {
        vec![
            ParamBundle {
                name: ParamName("Camera".to_string()),
                value: ParamValue::CameraOps(vec![]),
                ..default()
            },
            ParamBundle {
                name: ParamName("Light".to_string()),
                value: ParamValue::LightOps(vec![]),
                ..default()
            },
        ]
    }

    fn create_bundle<'w>(
        entity: Entity,
        param: &mut SystemParamItem<'w, '_, Self::Param>,
    ) -> Self::Bundle {
        todo!()
    }
}

impl OpUpdate for TextureOpRender {
    type Param = ();

    fn update<'w>(entity: Entity, param: &mut SystemParamItem<'w, '_, Self::Param>) {
        todo!()
    }
}

impl OpShouldExecute for TextureOpRender {
    type Param = ();
}

impl OpExecute for TextureOpRender {
    fn execute(&self, entity: Entity, world: &mut World) {
        todo!()
    }
}

impl OpOnConnect for TextureOpRender {
    type Param = ();

    fn on_connect<'w>(
        entity: Entity,
        event: Connect,
        fully_connected: bool,
        param: &mut SystemParamItem<'w, '_, Self::Param>,
    ) {
        todo!()
    }
}

impl OpOnDisconnect for TextureOpRender {
    type Param = ();

    fn on_disconnect<'w>(
        entity: Entity,
        event: Disconnect,
        fully_connected: bool,
        param: &mut SystemParamItem<'w, '_, Self::Param>,
    ) {
        todo!()
    }
}

impl Op for TextureOpRender {
    const INPUTS: usize = 0;
    const OUTPUTS: usize = 1;
    const CATEGORY: &'static str = "Texture";
    type OpType = OpType<Self>;
}
