use bevy::ecs::system::lifetimeless::*;
use bevy::ecs::system::{StaticSystemParam, SystemParamItem, SystemState};
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::view::RenderLayers;
use rand::{Rng, SeedableRng};
use std::f32::consts::PI;
use std::ops::DerefMut;
use bevy::color::palettes::css::GRAY;

use crate::op::mesh::{MeshExt, MeshOpBundle, MeshOpHandle, MeshOpInputMeshes, CATEGORY};
use crate::op::{
    Op, OpExecute, OpImage, OpInputs, OpOnConnect, OpOnDisconnect, OpOutputs, OpPlugin,
    OpShouldExecute, OpSpawn, OpType, OpUpdate,
};
use crate::param::{IntoParams, ParamBundle, ParamName, ParamOrder, ParamValue, Params};
use crate::render_layers::RenderLayerManager;
use crate::ui::event::{Connect, Disconnect};

#[derive(Default)]
pub struct MeshOpNoisePlugin;

impl Plugin for MeshOpNoisePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(OpPlugin::<MeshOpNoise>::default());
    }
}

#[derive(Component, ExtractComponent, Clone, Default, Debug)]
pub struct MeshOpNoise;

impl OpSpawn for MeshOpNoise {
    type Param = (
        SCommands,
        SResMut<Assets<Mesh>>,
        SResMut<Assets<Image>>,
        SResMut<Assets<StandardMaterial>>,
        SResMut<RenderLayerManager>,
    );
    type Bundle = (MeshOpBundle, MeshOpInputMeshes, RenderLayers);

    fn create_bundle<'w>(
        entity: Entity,
        (commands, meshes, images, materials, layer_manager): &mut SystemParamItem<
            'w,
            '_,
            Self::Param,
        >,
    ) -> Self::Bundle {
        let image = OpImage::new_image(512, 512);
        let image = images.add(image);

        let new_layer = layer_manager.next_open_layer();

        commands.spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(0.0, 0.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
                camera: Camera {
                    target: RenderTarget::Image(image.clone()),
                    ..default()
                },
                ..default()
            },
            RenderLayers::from_layer(new_layer),
        ));

        commands.spawn((
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
            RenderLayers::from_layer(new_layer),
        ));

        let mesh = meshes.add(Mesh::from(Circle::new(0.0001)));
        (
            MeshOpBundle {
                mesh: MeshOpHandle(mesh.clone()),
                pbr: PbrBundle {
                    mesh,
                    material: materials.add(Color::from(GRAY)),
                    transform: Transform::from_xyz(0.0, 0.0, 0.0)
                        .with_rotation(Quat::from_rotation_x(-PI / 4.0)),
                    ..default()
                },
                image: OpImage(image),
                inputs: OpInputs {
                    count: Self::INPUTS,
                    connections: Vec::new(),
                },
                outputs: OpOutputs {
                    count: Self::OUTPUTS,
                },
            },
            MeshOpInputMeshes::default(),
            RenderLayers::from_layer(new_layer),
        )
    }

    fn params(bundle: &Self::Bundle) -> Vec<ParamBundle> {
        [
            vec![
                ParamBundle {
                    name: ParamName("Strength".to_string()),
                    value: ParamValue::F32(0.1),
                    order: ParamOrder(0),
                    ..default()
                },
                ParamBundle {
                    name: ParamName("Seed".to_string()),
                    value: ParamValue::U32(0),
                    order: ParamOrder(1),
                    ..default()
                },
            ],
            bundle.0.pbr.transform.as_params(),
        ]
        .concat()
    }
}

impl OpUpdate for MeshOpNoise {
    type Param = (SQuery<(Write<Transform>,)>, Params<'static, 'static>);

    fn update<'w>(entity: Entity, param: &mut SystemParamItem<'w, '_, Self::Param>) {
        let (me_q, params) = param;
        let (mut transform,) = me_q.get_mut(entity).unwrap();

        params.get_mut(entity, "Translation").map(|mut param| {
            if let ParamValue::Vec3(translation) = param.deref_mut() {
                transform.translation = *translation;
            }
        });
        params.get_mut(entity, "Rotation").map(|mut param| {
            if let ParamValue::Quat(rotation) = param.deref_mut() {
                transform.rotation = *rotation;
            }
        });
        params.get_mut(entity, "Scale").map(|mut param| {
            if let ParamValue::Vec3(scale) = param.deref_mut() {
                transform.scale = *scale;
            }
        });
    }
}

impl OpShouldExecute for MeshOpNoise {
    type Param = ();
}

impl OpExecute for MeshOpNoise {
    fn execute(&self, entity: Entity, world: &mut World) {
        let mut params = SystemState::<Params>::new(world);
        let mut params = params.get_mut(world);
        let seed = params.get(entity, "Seed").unwrap().as_u32();
        let strength = params.get(entity, "Strength").unwrap().as_f32();

        let inputs = world.entity(entity).get::<OpInputs>().unwrap();
        if !inputs.is_fully_connected() {
            return;
        }
        let input = inputs.connections[0];
        let input_mesh = world.entity(input).get::<MeshOpHandle>().unwrap().clone();
        let my_mesh = world.entity(entity).get::<MeshOpHandle>().unwrap().clone();

        let mut meshes = world.get_resource_mut::<Assets<Mesh>>().unwrap();
        let input_mesh = meshes.get(&input_mesh.0).unwrap().clone();
        let mut mesh = meshes.get_mut(&my_mesh.0).unwrap();
        *mesh = input_mesh.clone();

        let [a, b, c, d] = seed.to_le_bytes();
        let mut rng = rand::rngs::SmallRng::from_seed([
            a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d, a, b, c, d, a,
            b, c, d,
        ]);
        mesh.points_mut().iter_mut().for_each(|n| {
            let n = n.as_mut();
            let strength = strength.clone();
            let x = rng.gen_range(-strength..strength);
            let y = rng.gen_range(-strength..strength);
            let z = rng.gen_range(-strength..strength);
            n[0] = n[0] + x;
            n[1] = n[1] + y;
            n[2] = n[2] + z;
        });
    }
}

impl OpOnConnect for MeshOpNoise {
    type Param = (SQuery<Read<MeshOpHandle>>, SQuery<Write<MeshOpInputMeshes>>);

    fn on_connect<'w>(
        entity: Entity,
        event: Connect,
        fully_connected: bool,
        param: &mut SystemParamItem<'w, '_, Self::Param>,
    ) {
        let (mesh_q, inputs_meshes_q) = param;
        let mesh = mesh_q.get(event.output).unwrap();
        let mut inputs_meshes = inputs_meshes_q.get_mut(entity).unwrap();
        inputs_meshes.push(mesh.0.clone());
    }
}

impl OpOnDisconnect for MeshOpNoise {
    type Param = ();

    fn on_disconnect<'w>(
        entity: Entity,
        event: Disconnect,
        fully_connected: bool,
        param: &mut SystemParamItem<'w, '_, Self::Param>,
    ) {
        todo!()
    }
}

impl Op for MeshOpNoise {
    const INPUTS: usize = 1;
    const OUTPUTS: usize = 1;
    const CATEGORY: &'static str = CATEGORY;

    type OpType = OpType<MeshOpNoise>;
}
