use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::render_resource::ShaderType;

use crate::engine::graph::event::{Connect, Disconnect};
use crate::engine::op::texture::render::TextureOpRenderPlugin;
use crate::engine::op::texture::{
    create_bundle, on_connect, params, update, DefaultTextureBundle, DefaultTextureOnConnectParam,
    DefaultTextureSpawnParam, DefaultTextureUpdateParam, TextureOp, CATEGORY,
};
use crate::engine::op::{
    Op, OpExecute, OpInputs, OpOnConnect, OpOnDisconnect, OpPlugin, OpShouldExecute, OpSpawn,
    OpType, OpUpdate,
};
use crate::engine::param::{ParamBundle, ParamName, ParamOrder, ParamValue};

#[derive(Default)]
pub struct TextureOpNoisePlugin;

impl Plugin for TextureOpNoisePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<OpType<TextureOpNoise>>::default(),
            OpPlugin::<TextureOpNoise>::default(),
            TextureOpRenderPlugin::<TextureOpNoise>::default(),
        ));
    }
}

#[derive(Component, ExtractComponent, Clone, Default, Debug)]
pub struct TextureOpNoise;

impl Op for TextureOpNoise {
    const INPUTS: usize = 0;
    const OUTPUTS: usize = 1;
    const CATEGORY: &'static str = CATEGORY;

    type OpType = OpType<Self>;
}

impl OpSpawn for TextureOpNoise {
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

impl OpUpdate for TextureOpNoise {
    type Param = DefaultTextureUpdateParam<Self>;

    fn update<'w>(entity: Entity, param: &mut SystemParamItem<'w, '_, Self::Param>) {
        update::<Self>(entity, param)
    }
}

impl OpShouldExecute for TextureOpNoise {
    type Param = ();

    fn should_execute<'w>(
        entity: Entity,
        param: &mut SystemParamItem<'w, '_, Self::Param>,
    ) -> bool {
        false
    }
}

impl OpExecute for TextureOpNoise {
    fn execute(&self, entity: Entity, world: &mut World) {}
}

impl OpOnConnect for TextureOpNoise {
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

impl OpOnDisconnect for TextureOpNoise {
    type Param = ();

    fn on_disconnect<'w>(
        entity: Entity,
        event: Disconnect,
        fully_connected: bool,
        param: &mut SystemParamItem<'w, '_, Self::Param>,
    ) {
    }
}

impl TextureOp for TextureOpNoise {
    const SHADER: &'static str = "shaders/texture/noise.wgsl";
    type Uniform = TextureNoiseSettings;

    fn params() -> Vec<ParamBundle> {
        vec![
            ParamBundle {
                name: ParamName("Strength".to_string()),
                value: ParamValue::F32(10.0),
                order: ParamOrder(0),
                ..default()
            },
            ParamBundle {
                name: ParamName("B".to_string()),
                value: ParamValue::F32(10.0),
                order: ParamOrder(0),
                ..default()
            },
            ParamBundle {
                name: ParamName("C".to_string()),
                value: ParamValue::F32(10.0),
                order: ParamOrder(0),
                ..default()
            },
            ParamBundle {
                name: ParamName("Seed".to_string()),
                value: ParamValue::F32(10.0),
                order: ParamOrder(0),
                ..default()
            },
        ]
    }

    fn update_uniform(uniform: &mut Self::Uniform, params: &Vec<(&ParamName, &ParamValue)>) {
        for (name, value) in params {
            match name.as_str() {
                "Strength" => {
                    if let ParamValue::F32(value) = value {
                        uniform.strength = *value;
                    }
                }
                "B" => {
                    if let ParamValue::F32(value) = value {
                        uniform.b = *value;
                    }
                }
                "C" => {
                    if let ParamValue::F32(value) = value {
                        uniform.c = *value;
                    }
                }
                "Seed" => {
                    if let ParamValue::F32(value) = value {
                        uniform.seed = *value;
                    }
                }
                _ => {}
            }
        }
    }
}

// This is the component that will get passed to the shader
#[derive(Component, Default, Debug, Clone, Copy, ExtractComponent, ShaderType)]
pub struct TextureNoiseSettings {
    pub strength: f32,
    pub b: f32,
    pub c: f32,
    pub seed: f32,
    #[cfg(feature = "webgl2")]
    _webgl2_padding: Vec3,
}
