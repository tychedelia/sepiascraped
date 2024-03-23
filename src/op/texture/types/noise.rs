use crate::param::{ParamBundle, ParamName, ParamOrder, ParamValue};
use crate::Sets::{Graph, Uniforms};
use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::render_resource::ShaderType;
use crate::op::{OpPlugin, OpType};

use crate::op::texture::render::TextureOpRenderPlugin;
use crate::op::texture::{TextureOp};

#[derive(Default)]
pub struct TextureOpNoisePlugin;

impl Plugin for TextureOpNoisePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<OpType<TextureOpNoise>>::default(),
            OpPlugin::<OpType<TextureOpNoise>>::default(),
            TextureOpRenderPlugin::<TextureOpNoise>::default(),
        ));
    }
}

#[derive(Component, Clone, Default, Debug)]
pub struct TextureOpNoise;

impl TextureOp for TextureOpNoise {
    const INPUTS: usize = 0;
    const OUTPUTS: usize = 1;
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
