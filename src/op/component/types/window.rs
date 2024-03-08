use crate::index::CompositeIndex2;
use crate::op::component::{spawn_component_op, ComponentOp, ComponentOpMeta, ComponentOpType};
use crate::op::texture::TextureOpImage;
use crate::param::{ParamBundle, ParamName, ParamOrder, ParamValue};
use crate::ui::graph::OpRef;
use crate::OpName;
use bevy::asset::AssetContainer;
use bevy::math::Vec4;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::view::RenderLayers;
use bevy::window::WindowRef;

#[derive(Default)]
pub struct ComponentOpWindowPlugin;

impl Plugin for ComponentOpWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_component_op::<ComponentOpWindow>, spawn_op, update).chain(),
        );
    }
}

#[derive(Component, Clone, Debug)]
pub struct WindowTexture(Entity);

fn update(
    mut commands: Commands,
    mut self_q: Query<(Entity, &mut Window), With<ComponentOpType<ComponentOpWindow>>>,
    texture_q: Query<&TextureOpImage>,
    param_q: Query<&ParamValue>,
    param_index: Res<CompositeIndex2<OpRef, ParamName>>,
    images: Res<Assets<Image>>,
) {
    for (entity, mut window) in self_q.iter_mut() {
        let param_entity = param_index.get(&(OpRef(entity), ParamName("Texture".to_string())));
        if let Some(param_entity) = param_entity {
            if let Ok(param_value) = param_q.get(*param_entity) {
                if let ParamValue::TextureOp(texture_entity) = param_value {
                    if let Some(texture_entity) = texture_entity {
                        if let Ok(texture) = texture_q.get(*texture_entity) {
                            let image = images.get(texture.0.clone()).unwrap();

                            window
                                .resolution
                                .set_physical_resolution(image.width(), image.height());

                            commands
                                .entity(entity)
                                .insert(SpriteBundle {
                                    texture: texture.0.clone(),
                                    ..default()
                                })
                                .insert(WindowTexture(*texture_entity));
                        }
                    }
                }
            }
        }
    }
}

fn spawn_op(
    mut commands: Commands,
    added_q: Query<
        (Entity, &OpName),
        (With<ComponentOp>, Added<ComponentOpType<ComponentOpWindow>>),
    >,
    count_q: Query<Entity, With<Window>>,
) {
    let mut count = count_q.iter().count();
    if count > 32 {
        panic!("Too many windows")
    }
    for (entity, op_name) in added_q.iter() {
        count += 1;
        commands.entity(entity).insert((
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
            RenderLayers::layer(count as u8),
        ));
    }
}

#[derive(Component, Clone, Default, Debug)]
pub struct ComponentOpWindow;

impl ComponentOpMeta for ComponentOpWindow {
    type OpType = ComponentOpType<ComponentOpWindow>;

    fn params() -> Vec<ParamBundle> {
        vec![ParamBundle {
            name: ParamName("Texture".to_string()),
            value: ParamValue::TextureOp(None),
            order: ParamOrder(0),
            ..default()
        }]
    }
}
