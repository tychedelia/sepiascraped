use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::render_resource::ShaderType;

use crate::engine::op::texture::render::TextureOpRenderPlugin;
use crate::engine::op::texture::{create_bundle, on_connect, on_disconnect, params, update, DefaultTextureBundle, DefaultTextureOnConnectParam, DefaultTextureSpawnParam, DefaultTextureUpdateParam, TextureOp, CATEGORY, DefaultTextureOnDisconnectParam};
use crate::engine::op::{Op, OpExecute, OpOnConnect, OpOnDisconnect, OpPlugin, OpShouldExecute, OpSpawn, OpType, OpUpdate};
use crate::engine::param::{ParamBundle, ParamName, ParamOrder, ParamValue};
use crate::engine::graph::event::{Connect, Disconnect};

#[derive(Default)]
pub struct TextureOpCompositePlugin;

impl Plugin for TextureOpCompositePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<OpType<TextureOpComposite>>::default(),
            OpPlugin::<TextureOpComposite>::default(),
            TextureOpRenderPlugin::<TextureOpComposite>::default(),
        ));
    }
}

impl Op for TextureOpComposite {
    const INPUTS: usize = 2;
    const OUTPUTS: usize = 1;
    const CATEGORY: &'static str = CATEGORY;

    type OpType = OpType<Self>;
}

impl OpSpawn for TextureOpComposite {
    type Param = DefaultTextureSpawnParam;
    type Bundle = DefaultTextureBundle<Self>;

    fn params(bundle: &Self::Bundle) -> Vec<ParamBundle> {
        params::<Self>(bundle)
    }

    fn create_bundle<'w>(
        entity: Entity,
        param: &mut SystemParamItem<'w, '_, Self::Param>,
    ) -> Self::Bundle {
        create_bundle::<Self>(entity, param)
    }
}

impl OpUpdate for TextureOpComposite {
    type Param = DefaultTextureUpdateParam<Self>;

    fn update<'w>(entity: Entity, param: &mut SystemParamItem<'w, '_, Self::Param>) {
        update::<Self>(entity, param)
    }
}

impl OpShouldExecute for TextureOpComposite {
    type Param = ();

    fn should_execute<'w>(
        entity: Entity,
        param: &mut SystemParamItem<'w, '_, Self::Param>,
    ) -> bool {
        false
    }
}

impl OpExecute for TextureOpComposite {
    fn execute(&self, entity: Entity, world: &mut World) {}
}

impl OpOnConnect for TextureOpComposite {
    type Param = DefaultTextureOnConnectParam;

    fn on_connect<'w>(
        entity: Entity,
        event: Connect,
        fully_connected: bool,
        param: &mut SystemParamItem<'w, '_, Self::Param>,
    ) {
        on_connect(entity, event, fully_connected, param)
    }
}

impl OpOnDisconnect for TextureOpComposite {
    type Param = DefaultTextureOnDisconnectParam;

    fn on_disconnect<'w>(
        entity: Entity,
        event: Disconnect,
        fully_connected: bool,
        param: &mut SystemParamItem<'w, '_, Self::Param>,
    ) {
        on_disconnect(entity, event, fully_connected, param)
    }
}

impl TextureOp for TextureOpComposite {
    const SHADER: &'static str = "shaders/texture/composite.wgsl";
    type Uniform = CompositeSettings;

    fn params() -> Vec<ParamBundle> {
        vec![ParamBundle {
            name: ParamName("Mode".to_string()),
            value: ParamValue::U32(0),
            order: ParamOrder(0),
            ..default()
        }]
    }

    fn update_uniform(uniform: &mut Self::Uniform, params: &Vec<(&ParamName, &ParamValue)>) {
        for (name, value) in params {
            match name.as_str() {
                "Mode" => {
                    if let ParamValue::U32(value) = value {
                        uniform.mode = *value;
                    }
                }
                _ => {}
            }
        }
    }
}

#[derive(Component, ExtractComponent, Clone, Default, Debug)]
pub struct TextureOpComposite;

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum CompositeMode {
    #[default]
    Add = 0,
    Multiply = 1,
    Subtract = 2,
    Divide = 3,
}

impl CompositeMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            CompositeMode::Add => "Add",
            CompositeMode::Multiply => "Multiply",
            CompositeMode::Subtract => "Subtract",
            CompositeMode::Divide => "Divide",
        }
    }

    pub fn as_u32(&self) -> u32 {
        *self as u32
    }

    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(CompositeMode::Add),
            1 => Some(CompositeMode::Multiply),
            2 => Some(CompositeMode::Subtract),
            3 => Some(CompositeMode::Divide),
            _ => None,
        }
    }
}

// This is the component that will get passed to the shader
#[derive(Component, Default, Debug, Clone, Copy, ExtractComponent, ShaderType)]
pub struct CompositeSettings {
    pub mode: u32,
    // WebGL2 structs must be 16 byte aligned.
    #[cfg(feature = "webgl2")]
    _webgl2_padding: Vec3,
}
