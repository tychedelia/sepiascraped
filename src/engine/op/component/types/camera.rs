use bevy::asset::AssetContainer;
use bevy::ecs::system::lifetimeless::*;
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::view::RenderLayers;

use crate::engine::graph::event::{Connect, Disconnect};
use crate::engine::op::component::CATEGORY;
use crate::engine::op::OpName;
use crate::engine::op::OpRef;
use crate::engine::op::{Op, OpInputs, OpOutputs, OpPlugin, OpType};
use crate::engine::op::{
    OpExecute, OpImage, OpOnConnect, OpOnDisconnect, OpShouldExecute, OpSpawn, OpUpdate,
};
use crate::engine::param::{IntoParams, ParamBundle, ParamName, ParamOrder, ParamValue};
use crate::index::{CompositeIndex2, IndexPlugin};
use crate::render_layers::RenderLayerManager;

#[derive(Default)]
pub struct ComponentOpCameraPlugin;

impl Plugin for ComponentOpCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(OpPlugin::<ComponentOpCamera>::default());
    }
}

#[derive(Component, Clone, Debug)]
pub struct CameraTexture(Entity);

#[derive(Component, ExtractComponent, Clone, Default, Debug)]
pub struct ComponentOpCamera;

impl OpUpdate for ComponentOpCamera {
    type Param = ();

    fn update<'w>(entity: Entity, param: &mut SystemParamItem<'w, '_, Self::Param>) {}
}

impl OpSpawn for ComponentOpCamera {
    type Param = (SQuery<Read<OpName>>, SResMut<RenderLayerManager>);
    type Bundle = (Camera3dBundle, RenderLayers, OpImage, OpInputs, OpOutputs);

    fn params(bundle: &Self::Bundle) -> Vec<ParamBundle> {
        [vec![], bundle.0.transform.as_params()].concat()
    }

    fn create_bundle<'w>(
        entity: Entity,
        (name_q, layer_manager): &mut SystemParamItem<'w, '_, Self::Param>,
    ) -> Self::Bundle {
        (
            Camera3dBundle {
                transform: Transform::default(),
                ..Default::default()
            },
            RenderLayers::from_layers(&[layer_manager.next_open_layer()]),
            OpImage::default(),
            OpInputs::default(),
            OpOutputs::default(),
        )
    }
}

impl OpShouldExecute for ComponentOpCamera {
    type Param = ();

    fn should_execute<'w>(
        entity: Entity,
        param: &mut SystemParamItem<'w, '_, Self::Param>,
    ) -> bool {
        true
    }
}

impl OpExecute for ComponentOpCamera {
    fn execute(&self, entity: Entity, world: &mut World) {}
}

impl OpOnConnect for ComponentOpCamera {
    type Param = ();

    fn on_connect<'w>(
        entity: Entity,
        event: Connect,
        fully_connected: bool,
        param: &mut SystemParamItem<'w, '_, Self::Param>,
    ) {
    }
}

impl OpOnDisconnect for ComponentOpCamera {
    type Param = ();

    fn on_disconnect<'w>(
        entity: Entity,
        event: Disconnect,
        fully_connected: bool,
        param: &mut SystemParamItem<'w, '_, Self::Param>,
    ) {
    }
}

impl Op for ComponentOpCamera {
    const INPUTS: usize = 0;
    const OUTPUTS: usize = 0;
    const CATEGORY: &'static str = CATEGORY;

    type OpType = OpType<ComponentOpCamera>;
}
