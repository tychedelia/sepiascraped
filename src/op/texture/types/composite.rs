use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::render_resource::ShaderType;

use crate::op::{Op, OpPlugin, OpType};
use crate::op::texture::{impl_op, TextureOp};
use crate::op::texture::render::TextureOpRenderPlugin;
use crate::param::{ParamBundle, ParamName, ParamOrder, ParamValue};

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

impl_op!(TextureOpComposite, 2, 1);

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
