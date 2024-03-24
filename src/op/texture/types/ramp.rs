use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::render_resource::ShaderType;

use crate::op::{OpPlugin, OpType};
use crate::op::texture::{impl_op, TextureOp};
use crate::op::texture::render::TextureOpRenderPlugin;
use crate::param::{ParamBundle, ParamName, ParamOrder, ParamValue};

#[derive(Default)]
pub struct TextureOpRampPlugin;

impl Plugin for TextureOpRampPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ExtractComponentPlugin::<OpType<TextureOpRamp>>::default(),
            OpPlugin::<TextureOpRamp>::default(),
            TextureOpRenderPlugin::<TextureOpRamp>::default(),
        ));
    }
}

#[derive(Component, Clone, Default, Debug)]
pub struct TextureOpRamp;

impl_op!(TextureOpRamp, 0, 1);

impl TextureOp for TextureOpRamp {
    const SHADER: &'static str = "shaders/texture/ramp.wgsl";
    type Uniform = TextureRampSettings;

    fn params() -> Vec<ParamBundle> {
        vec![
            ParamBundle {
                name: ParamName("Color A".to_string()),
                value: ParamValue::Color(Vec4::new(1.0, 0.0, 0.0, 1.0)),
                order: ParamOrder(0),
                ..default()
            },
            ParamBundle {
                name: ParamName("Color B".to_string()),
                value: ParamValue::Color(Vec4::new(0.0, 0.0, 1.0, 1.0)),
                order: ParamOrder(1),
                ..default()
            },
            ParamBundle {
                name: ParamName("Mode".to_string()),
                value: ParamValue::U32(0),
                order: ParamOrder(1),
                ..default()
            },
        ]
    }

    fn update_uniform(uniform: &mut Self::Uniform, params: &Vec<(&ParamName, &ParamValue)>) {
        for (name, value) in params {
            match name.as_str() {
                "Color A" => {
                    if let ParamValue::Color(color) = value {
                        uniform.color_a = *color;
                    }
                }
                "Color B" => {
                    if let ParamValue::Color(color) = value {
                        uniform.color_b = *color;
                    }
                }
                "Mode" => {
                    if let ParamValue::U32(mode) = value {
                        uniform.mode = *mode;
                    }
                }
                _ => {}
            }
        }
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum TextureRampMode {
    #[default]
    Horizontal = 0,
    Vertical = 1,
    Circular = 2,
}

impl TextureRampMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            TextureRampMode::Horizontal => "Horizontal",
            TextureRampMode::Vertical => "Vertical",
            TextureRampMode::Circular => "Circular",
        }
    }

    pub fn as_u32(&self) -> u32 {
        *self as u32
    }

    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(TextureRampMode::Horizontal),
            1 => Some(TextureRampMode::Vertical),
            2 => Some(TextureRampMode::Circular),
            _ => None,
        }
    }
}

// This is the component that will get passed to the shader
#[derive(Component, Default, Debug, Clone, Copy, ExtractComponent, ShaderType)]
pub struct TextureRampSettings {
    pub color_a: Vec4,
    pub color_b: Vec4,
    pub mode: u32,
    // WebGL2 structs must be 16 byte aligned.
    #[cfg(feature = "webgl2")]
    _webgl2_padding: Vec3,
}
