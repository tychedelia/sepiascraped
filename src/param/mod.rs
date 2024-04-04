use bevy::ecs::system::SystemParam;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::ops::DerefMut;

use bevy::prelude::*;
use bevy::utils::AHasher;

use crate::index::{CompositeIndex2, CompositeIndex2Plugin};
use crate::op::{OpCategory, OpRef};
use crate::script::update;
use crate::Sets;

pub struct ParamPlugin;

impl Plugin for ParamPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, validate.in_set(Sets::Params))
            .add_plugins(CompositeIndex2Plugin::<OpRef, ParamName>::new());
    }
}

#[derive(Bundle, Default, Clone)]
pub struct ParamBundle {
    pub name: ParamName,
    pub value: ParamValue,
    pub order: ParamOrder,
    pub page: ParamPage,
}

#[derive(Component, Clone, Default, Debug)]
pub struct ParamPage(pub String);
#[derive(Component, Clone, Default, Debug, Eq, Ord, PartialOrd, PartialEq)]
pub struct ParamOrder(pub u32);
#[derive(Component, Clone, Default, Debug)]
pub struct Param;
#[derive(
    Component, Deref, DerefMut, Default, Clone, PartialEq, Eq, Hash, Debug, Ord, PartialOrd,
)]
pub struct ParamName(pub String);

trait ParamType: Default {}

#[derive(Component, PartialEq, Clone, Debug, Default)]
pub enum ParamValue {
    #[default]
    None,
    F32(f32),
    U32(u32),
    UVec2(UVec2),
    Vec2(Vec2),
    Vec3(Vec3),
    Quat(Quat),
    Color(Vec4),
    Bool(bool),
    TextureOp(Option<Entity>),
    MeshOp(Option<Entity>),
    MaterialOp(Option<Entity>),
}

impl ParamValue {
    pub fn as_f32(&self) -> f32 {
        match self {
            ParamValue::F32(v) => *v,
            _ => panic!("Param is not an f32"),
        }
    }

    pub fn as_u32(&self) -> u32 {
        match self {
            ParamValue::U32(v) => *v,
            _ => panic!("Param is not a u32"),
        }
    }

    pub fn as_uvec2(&self) -> UVec2 {
        match self {
            ParamValue::UVec2(v) => *v,
            _ => panic!("Param is not a UVec2"),
        }
    }

    pub fn as_vec2(&self) -> Vec2 {
        match self {
            ParamValue::Vec2(v) => *v,
            _ => panic!("Param is not a Vec2"),
        }
    }

    pub fn as_vec3(&self) -> Vec3 {
        match self {
            ParamValue::Vec3(v) => *v,
            _ => panic!("Param is not a Vec3"),
        }
    }

    pub fn as_quat(&self) -> Quat {
        match self {
            ParamValue::Quat(v) => *v,
            _ => panic!("Param is not a Quat"),
        }
    }

    pub fn as_color(&self) -> Vec4 {
        match self {
            ParamValue::Color(v) => *v,
            _ => panic!("Param is not a Color"),
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            ParamValue::Bool(v) => *v,
            _ => panic!("Param is not a bool"),
        }
    }

    pub fn as_texture_op(&self) -> Option<Entity> {
        match self {
            ParamValue::TextureOp(v) => v.clone(),
            _ => panic!("Param is not a TextureOp"),
        }
    }

    pub fn as_mesh_op(&self) -> Option<Entity> {
        match self {
            ParamValue::MeshOp(v) => v.clone(),
            _ => panic!("Param is not a MeshOp"),
        }
    }

    pub fn as_material_op(&self) -> Option<Entity> {
        match self {
            ParamValue::MaterialOp(v) => v.clone(),
            _ => panic!("Param is not a MaterialOp"),
        }
    }
}

impl Hash for ParamValue {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ParamValue::None => 0.hash(state),
            ParamValue::F32(v) => v.to_bits().hash(state),
            ParamValue::U32(v) => v.hash(state),
            ParamValue::UVec2(v) => v.hash(state),
            ParamValue::Vec2(v) => {
                v.x.to_bits().hash(state);
                v.y.to_bits().hash(state);
            }
            ParamValue::Vec3(v) => {
                v.x.to_bits().hash(state);
                v.y.to_bits().hash(state);
                v.z.to_bits().hash(state);
            }
            ParamValue::Quat(v) => {
                v.x.to_bits().hash(state);
                v.y.to_bits().hash(state);
                v.z.to_bits().hash(state);
                v.w.to_bits().hash(state);
            }
            ParamValue::Color(v) => {
                v.x.to_bits().hash(state);
                v.y.to_bits().hash(state);
                v.z.to_bits().hash(state);
                v.w.to_bits().hash(state);
            }
            ParamValue::Bool(v) => v.hash(state),
            ParamValue::TextureOp(v) => v.hash(state),
            ParamValue::MeshOp(v) => v.hash(state),
            ParamValue::MaterialOp(v) => v.hash(state),
        }
    }
}

#[derive(Component, Deref, DerefMut, Default, Debug)]
pub struct ParamHash(pub u64);

#[derive(Component, Default, Debug)]
pub struct ScriptedParam;
#[derive(Component, Default, Debug)]
pub struct ScriptedParamError(pub String);

pub fn validate(
    mut commands: Commands,
    mut params_q: Query<(Entity, &mut ParamValue), Changed<ParamValue>>,
    category_q: Query<&OpCategory>,
) {
    for (entity, mut param_value) in params_q.iter_mut() {
        match param_value.deref_mut() {
            ParamValue::TextureOp(Some(e)) => {
                let category = category_q.get(*e).unwrap();
                if !category.is_texture() {
                    commands
                        .entity(entity)
                        .insert(ScriptedParamError("Invalid texture".to_string()));
                    *param_value = ParamValue::TextureOp(None);
                }
            }
            ParamValue::MeshOp(Some(e)) => {
                let category = category_q.get(*e).unwrap();
                if !category.is_mesh() {
                    commands
                        .entity(entity)
                        .insert(ScriptedParamError("Invalid mesh".to_string()));
                    *param_value = ParamValue::MeshOp(None);
                }
            }
            _ => {}
        }
    }
}

#[derive(SystemParam)]
pub struct Params<'w, 's> {
    parent_q: Query<'w, 's, &'static Children>,
    params_q: Query<'w, 's, &'static mut ParamValue>,
    param_idx: Res<'w, CompositeIndex2<OpRef, ParamName>>,
}

impl<'w, 's> Params<'w, 's> {
    // Get the hash of all the parameters for an entity
    pub fn hash(&self, entity: Entity) -> u64 {
        self.parent_q
            .get(entity)
            .iter()
            .flat_map(|c| c.iter().map(|e| self.params_q.get(*e)))
            .filter_map(|p| p.ok())
            .fold(AHasher::default(), |mut h, p| {
                p.hash(&mut h);
                h
            })
            .finish()
    }

    pub fn get_all(&self, entity: Entity) -> Vec<&ParamValue> {
        self.parent_q
            .get(entity)
            .iter()
            .flat_map(|c| c.iter().map(|e| self.params_q.get(*e)))
            .filter_map(|p| p.ok())
            .collect()
    }

    pub fn get(&self, entity: Entity, name: impl Into<String>) -> Option<&ParamValue> {
        self.param_idx
            .get(&(OpRef(entity), ParamName(name.into())))
            .map(|e| self.params_q.get(*e).unwrap())
    }

    pub fn get_mut(&mut self, entity: Entity, name: impl Into<String>) -> Option<Mut<ParamValue>> {
        self.param_idx
            .get(&(OpRef(entity), ParamName(name.into())))
            .map(|e| self.params_q.get_mut(*e).unwrap())
    }
}

pub trait IntoParams {
    fn as_params(&self) -> Vec<ParamBundle>;
}

/// IntoParams for Transform
impl IntoParams for Transform {
    fn as_params(&self) -> Vec<ParamBundle> {
        vec![
            ParamBundle {
                name: ParamName("Translation".to_string()),
                value: ParamValue::Vec3(self.translation),
                order: ParamOrder(0),
                ..default()
            },
            ParamBundle {
                name: ParamName("Rotation".to_string()),
                value: ParamValue::Quat(self.rotation),
                order: ParamOrder(1),
                ..default()
            },
            ParamBundle {
                name: ParamName("Scale".to_string()),
                value: ParamValue::Vec3(self.scale),
                order: ParamOrder(2),
                ..default()
            },
        ]
    }
}
