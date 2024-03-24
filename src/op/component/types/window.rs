use bevy::asset::AssetContainer;
use bevy::ecs::system::lifetimeless::*;
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::view::RenderLayers;
use bevy::window::WindowRef;

use crate::index::CompositeIndex2;
use crate::op::{Op, OpPlugin, OpType};
use crate::op::OpRef;
use crate::op::texture::TextureOpImage;
use crate::OpName;
use crate::param::{ParamBundle, ParamName, ParamOrder, ParamValue};

#[derive(Default)]
pub struct ComponentOpWindowPlugin;

impl Plugin for ComponentOpWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(OpPlugin::<ComponentOpWindow>::default());
    }
}

#[derive(Component, Clone, Debug)]
pub struct WindowTexture(Entity);

#[derive(Component, Clone, Default, Debug)]
pub struct ComponentOpWindow;

impl Op for ComponentOpWindow {
    type OpType = OpType<ComponentOpWindow>;
    type UpdateParam = (
        SCommands,
        SQuery<
            (Write<Window>, Option<Read<WindowTexture>>),
            With<OpType<ComponentOpWindow>>,
        >,
        SQuery<Read<TextureOpImage>>,
        SQuery<Read<ParamValue>>,
        SRes<CompositeIndex2<OpRef, ParamName>>,
        SRes<Assets<Image>>,
    );
    type BundleParam = (SQuery<Read<OpName>>, SQuery<Entity, With<Window>>);
    type Bundle = (Window, Camera2dBundle, RenderLayers);

    fn update<'w>(entity: Entity, param: &mut SystemParamItem<'w, '_, Self::UpdateParam>) {
        let (commands, self_q, texture_q, param_q, param_index, images) = param;

        let (mut window, curr_window_texture) = self_q.get_mut(entity).unwrap();

        let param_entity = param_index.get(&(OpRef(entity), ParamName("Texture".to_string())));
        let Some(param_entity) = param_entity else {
            return;
        };

        let Ok(param_value) = param_q.get(*param_entity) else {
            return;
        };

        let ParamValue::TextureOp(texture_entity) = param_value else {
            return;
        };

        let Some(texture_entity) = texture_entity else {
            return;
        };

        let Ok(texture) = texture_q.get(*texture_entity) else {
            return;
        };

        if let Some(curr_window_texture) = curr_window_texture {
            if curr_window_texture.0 == *texture_entity {
                return;
            }
        }

        let image = images.get(texture.0.clone()).unwrap();

        let scale_factor = window.resolution.scale_factor();
        let window_size = image.size_f32() * scale_factor;
        window
            .resolution
            .set_physical_resolution(window_size.x as u32, window_size.y as u32);

        commands
            .entity(entity)
            .insert(SpriteBundle {
                texture: texture.0.clone(),
                ..default()
            })
            .insert(WindowTexture(*texture_entity));
    }

    fn create_bundle<'w>(
        entity: Entity,
        (name_q, count_q): &mut SystemParamItem<'w, '_, Self::BundleParam>,
    ) -> Self::Bundle {
        let mut count = count_q.iter().count();
        if count > 32 {
            panic!("Too many windows")
        }

        let name = name_q.get(entity).unwrap();
        (
            Window {
                title: name.0.clone(),
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
        )
    }

    fn params() -> Vec<ParamBundle> {
        vec![
            ParamBundle {
                name: ParamName("Texture".to_string()),
                value: ParamValue::TextureOp(None),
                order: ParamOrder(0),
                ..default()
            },
            ParamBundle {
                name: ParamName("Open".to_string()),
                value: ParamValue::Bool(false),
                order: ParamOrder(1),
                ..default()
            },
        ]
    }
}
