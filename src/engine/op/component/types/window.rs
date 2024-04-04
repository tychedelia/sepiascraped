use bevy::asset::AssetContainer;
use bevy::ecs::system::lifetimeless::*;
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::view::RenderLayers;
use bevy::window::WindowRef;

use crate::index::CompositeIndex2;
use crate::engine::op::component::CATEGORY;
use crate::engine::op::OpRef;
use crate::engine::op::{Op, OpInputs, OpOutputs, OpPlugin, OpType};
use crate::engine::op::{
    OpExecute, OpImage, OpOnConnect, OpOnDisconnect, OpShouldExecute, OpSpawn, OpUpdate,
};
use crate::engine::param::{ParamBundle, ParamName, ParamOrder, ParamValue};
use crate::render_layers::RenderLayerManager;
use crate::engine::graph::event::{Connect, Disconnect};
use crate::engine::op::OpName;

#[derive(Default)]
pub struct ComponentOpWindowPlugin;

impl Plugin for ComponentOpWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(OpPlugin::<ComponentOpWindow>::default());
    }
}

#[derive(Component, Clone, Debug)]
pub struct WindowTexture(Entity);

#[derive(Component, ExtractComponent, Clone, Default, Debug)]
pub struct ComponentOpWindow;

impl OpUpdate for ComponentOpWindow {
    type Param = (
        SCommands,
        SQuery<(Write<Window>, Option<Read<WindowTexture>>), With<OpType<ComponentOpWindow>>>,
        SQuery<Read<OpImage>>,
        SQuery<Read<ParamValue>>,
        SRes<CompositeIndex2<OpRef, ParamName>>,
        SRes<Assets<Image>>,
    );

    fn update<'w>(entity: Entity, param: &mut SystemParamItem<'w, '_, Self::Param>) {
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

        let image = images.get(&texture.0.clone()).unwrap();

        let scale_factor = window.resolution.scale_factor();
        let window_size = image.size_f32() * scale_factor;
        window
            .resolution
            .set_physical_resolution(window_size.x as u32, window_size.y as u32);

        commands
            .entity(entity)
            .insert(OpImage(texture.0.clone()))
            .insert(SpriteBundle {
                texture: texture.0.clone(),
                ..default()
            })
            .insert(WindowTexture(*texture_entity));
    }
}

impl OpSpawn for ComponentOpWindow {
    type Param = (SQuery<Read<OpName>>, SResMut<RenderLayerManager>);
    type Bundle = (
        Window,
        Camera2dBundle,
        RenderLayers,
        OpImage,
        OpInputs,
        OpOutputs,
    );

    fn params(bundle: &Self::Bundle) -> Vec<ParamBundle> {
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

    fn create_bundle<'w>(
        entity: Entity,
        (name_q, layer_manager): &mut SystemParamItem<'w, '_, Self::Param>,
    ) -> Self::Bundle {
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
            RenderLayers::from_layer(layer_manager.next_open_layer()),
            OpImage::default(),
            OpInputs::default(),
            OpOutputs::default(),
        )
    }
}

impl OpShouldExecute for ComponentOpWindow {
    type Param = ();

    fn should_execute<'w>(
        entity: Entity,
        param: &mut SystemParamItem<'w, '_, Self::Param>,
    ) -> bool {
        true
    }
}

impl OpExecute for ComponentOpWindow {
    fn execute(&self, entity: Entity, world: &mut World) {}
}

impl OpOnConnect for ComponentOpWindow {
    type Param = ();

    fn on_connect<'w>(
        entity: Entity,
        event: Connect,
        fully_connected: bool,
        param: &mut SystemParamItem<'w, '_, Self::Param>,
    ) {
    }
}

impl OpOnDisconnect for ComponentOpWindow {
    type Param = ();

    fn on_disconnect<'w>(
        entity: Entity,
        event: Disconnect,
        fully_connected: bool,
        param: &mut SystemParamItem<'w, '_, Self::Param>,
    ) {
    }
}

impl Op for ComponentOpWindow {
    const CATEGORY: &'static str = CATEGORY;
    type OpType = OpType<ComponentOpWindow>;
}
