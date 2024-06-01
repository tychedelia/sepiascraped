use bevy::asset::AssetContainer;
use bevy::color::palettes::basic::GRAY;
use bevy::ecs::system::lifetimeless::*;
use bevy::ecs::system::SystemParamItem;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::view::RenderLayers;
use bevy::utils::hashbrown::HashMap;
use std::f32::consts::PI;
use std::ops::DerefMut;

use crate::engine::graph::event::{Connect, Disconnect};
use crate::engine::op::component::CATEGORY;
use crate::engine::op::material::MaterialOpHandle;
use crate::engine::op::OpName;
use crate::engine::op::OpRef;
use crate::engine::op::{Op, OpInputs, OpOutputs, OpPlugin, OpType};
use crate::engine::op::{
    OpExecute, OpImage, OpOnConnect, OpOnDisconnect, OpShouldExecute, OpSpawn, OpUpdate,
};
use crate::engine::param::{IntoParams, ParamBundle, ParamName, ParamOrder, ParamValue, Params};
use crate::index::CompositeIndex2;
use crate::render_layers::RenderLayerManager;

#[derive(Default)]
pub struct ComponentOpGeomPlugin;

impl Plugin for ComponentOpGeomPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(OpPlugin::<ComponentOpGeom>::default());
    }
}

#[derive(Component, Clone, Debug)]
pub struct GeomTexture(Entity);

#[derive(Component, ExtractComponent, Clone, Default, Debug)]
pub struct ComponentOpGeom;

impl OpUpdate for ComponentOpGeom {
    type Param = (
        SQuery<(Write<Handle<StandardMaterial>>)>,
        SQuery<(Write<MaterialOpHandle<StandardMaterial>>)>,
        SQuery<Write<Transform>>,
        SQuery<Read<OpInputs>>,
        Params<'static, 'static>,
    );

    fn update<'w>(entity: Entity, param: &mut SystemParamItem<'w, '_, Self::Param>) {
        let (mat_q, mat_op_q, transform_q, inputs_q, params) = param;

        // TODO: support other material types
        params.get_mut(entity, "Material").map(|mut param| {
            if let Some(mat_op) = param.as_material_op() {
                let their_mat = mat_op_q.get(mat_op).unwrap().clone();
                if let Ok(mut our_mat) = mat_q.get_mut(entity) {
                    *our_mat = their_mat.0;
                }
            }
        });

        let my_inputs = inputs_q.get(entity).unwrap();
        if my_inputs.is_fully_connected() {
            let (input, _) = my_inputs.connections[&0];
            let input_transform = transform_q.get(input).unwrap().clone();
            let mut our_transform = transform_q.get_mut(entity).unwrap();
            *our_transform = input_transform;
        }
    }
}

impl OpSpawn for ComponentOpGeom {
    type Param = (
        SCommands,
        SResMut<Assets<Image>>,
        SResMut<RenderLayerManager>,
    );
    type Bundle = (RenderLayers, OpImage, OpInputs, OpOutputs);

    fn params(bundle: &Self::Bundle) -> Vec<ParamBundle> {
        vec![ParamBundle {
            name: ParamName("Material".to_string()),
            value: ParamValue::MaterialOp(None),
            order: ParamOrder(0),
            ..default()
        }]
    }

    fn create_bundle<'w>(
        entity: Entity,
        (commands, images, layer_manager): &mut SystemParamItem<'w, '_, Self::Param>,
    ) -> Self::Bundle {
        let image = OpImage::new_image(512, 512);
        let image = images.add(image);

        let new_layer = layer_manager.next_open_layer();

        commands.spawn((
            OpRef(entity),
            Camera3dBundle {
                transform: Transform::from_xyz(0.0, 0.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
                camera: Camera {
                    target: RenderTarget::Image(image.clone()),
                    ..default()
                },
                ..default()
            },
            RenderLayers::from_layers(&[new_layer]),
        ));

        commands.spawn((
            OpRef(entity),
            PointLightBundle {
                point_light: PointLight {
                    shadows_enabled: true,
                    intensity: 10_000_000.,
                    range: 100.0,
                    ..default()
                },
                transform: Transform::from_xyz(8.0, 16.0, 8.0),
                ..default()
            },
            RenderLayers::from_layers(&[new_layer]),
        ));

        (
            RenderLayers::from_layers(&[new_layer]),
            OpImage(image),
            OpInputs::new(Self::INPUTS),
            OpOutputs::default(),
        )
    }
}

impl OpShouldExecute for ComponentOpGeom {
    type Param = ();

    fn should_execute<'w>(
        entity: Entity,
        param: &mut SystemParamItem<'w, '_, Self::Param>,
    ) -> bool {
        false
    }
}

impl OpExecute for ComponentOpGeom {
    fn execute(&self, entity: Entity, world: &mut World) {}
}

impl OpOnConnect for ComponentOpGeom {
    type Param = (
        SCommands,
        SQuery<(Write<Transform>, Write<Handle<Mesh>>)>,
        SQuery<Read<RenderLayers>>,
    );

    fn on_connect<'w>(
        entity: Entity,
        event: Connect,
        fully_connected: bool,
        param: &mut SystemParamItem<'w, '_, Self::Param>,
    ) {
        if fully_connected {
            let (commands, item_q, layer_q) = param;
            let (input_transform, input_mesh) = item_q.get(event.output).unwrap();
            let our_layer = layer_q.get(entity).unwrap();
            commands.entity(entity).insert((
                PbrBundle {
                    mesh: input_mesh.clone(),
                    transform: input_transform.clone(),
                    ..default()
                },
                our_layer.clone(),
            ));
        }
    }
}

impl OpOnDisconnect for ComponentOpGeom {
    type Param = ();

    fn on_disconnect<'w>(
        entity: Entity,
        event: Disconnect,
        fully_connected: bool,
        param: &mut SystemParamItem<'w, '_, Self::Param>,
    ) {
    }
}

impl Op for ComponentOpGeom {
    const INPUTS: usize = 1;
    const OUTPUTS: usize = 0;
    const CATEGORY: &'static str = CATEGORY;

    type OpType = OpType<ComponentOpGeom>;
}
