use crate::op::component::{spawn_component_op, ComponentOpMeta, ComponentOpType};
use crate::param::{ParamBundle, ParamName, ParamOrder, ParamValue};
use crate::OpName;
use bevy::math::Vec4;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::window::WindowRef;

#[derive(Default)]
pub struct ComponentOpWindowPlugin;

impl Plugin for ComponentOpWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (spawn_component_op::<ComponentOpWindow>, update));
    }
}

fn update() {

}

#[derive(Component, Clone, Default, Debug)]
pub struct ComponentOpWindow;

impl ComponentOpMeta for ComponentOpWindow {
    type OpType = ComponentOpType<ComponentOpWindow>;

    fn bundle(entity: Entity, op_name: &OpName, commands: &mut Commands) -> impl Bundle {
        (
            Window {
                title: op_name.0.clone(),
                ..default()
            },
            Camera2dBundle {
                camera: Camera {
                    target: RenderTarget::Window(WindowRef::Entity(entity)),
                    ..default()
                },
                ..default()
            },
        )
    }

    fn params() -> Vec<ParamBundle> {
        vec![ParamBundle {
            name: ParamName("Texture".to_string()),
            value: ParamValue::Color(Vec4::new(1.0, 0.0, 0.0, 1.0)),
            order: ParamOrder(0),
            ..default()
        }]
    }
}
