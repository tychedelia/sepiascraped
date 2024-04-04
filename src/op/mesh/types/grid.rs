use bevy::color::palettes::css::GRAY;
use bevy::ecs::system::lifetimeless::*;
use bevy::ecs::system::{StaticSystemParam, SystemParamItem};
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::mesh::Indices;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::view::{CameraLayer, RenderLayers};
use bevy::utils::HashMap;
use std::f32::consts::PI;
use std::ops::DerefMut;

use crate::op::mesh::{MeshOpBundle, MeshOpHandle, MeshOpInputMeshes, CATEGORY};
use crate::op::{
    Op, OpExecute, OpImage, OpInputs, OpOnConnect, OpOnDisconnect, OpOutputs, OpPlugin, OpRef,
    OpShouldExecute, OpSpawn, OpType, OpUpdate,
};
use crate::param::{IntoParams, ParamBundle, ParamValue, Params};
use crate::render_layers::RenderLayerManager;
use crate::ui::event::{Connect, Disconnect};

#[derive(Default)]
pub struct MeshOpGridPlugin;

impl Plugin for MeshOpGridPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(OpPlugin::<MeshOpGrid>::default());
    }
}

#[derive(Component, ExtractComponent, Clone, Default, Debug)]
pub struct MeshOpGrid;

impl OpSpawn for MeshOpGrid {
    type Param = (
        SCommands,
        SResMut<Assets<Mesh>>,
        SResMut<Assets<Image>>,
        SResMut<Assets<StandardMaterial>>,
        SResMut<RenderLayerManager>,
    );
    type Bundle = (MeshOpBundle, MeshOpInputMeshes, RenderLayers);

    fn params(bundle: &Self::Bundle) -> Vec<ParamBundle> {
        [vec![], bundle.0.pbr.transform.as_params()].concat()
    }

    fn create_bundle<'w>(
        entity: Entity,
        (commands, meshes, images, materials, layer_manager): &mut SystemParamItem<
            'w,
            '_,
            Self::Param,
        >,
    ) -> Self::Bundle {
        let mesh = meshes.add(Mesh::from(Grid::default()));
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
            CameraLayer::new(new_layer),
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
            RenderLayers::from_layer(new_layer),
        ));

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
}

impl OpUpdate for MeshOpGrid {
    type Param = (SQuery<Write<Transform>>, Params<'static, 'static>);

    fn update<'w>(entity: Entity, param: &mut SystemParamItem<'w, '_, Self::Param>) {
        let (transform, params) = param;

        params.get_mut(entity, "Translation").map(|mut param| {
            if let ParamValue::Vec3(translation) = param.deref_mut() {
                transform.get_mut(entity).unwrap().translation = *translation;
            }
        });
        params.get_mut(entity, "Rotation").map(|mut param| {
            if let ParamValue::Quat(rotation) = param.deref_mut() {
                transform.get_mut(entity).unwrap().rotation = *rotation;
            }
        });
        params.get_mut(entity, "Scale").map(|mut param| {
            if let ParamValue::Vec3(scale) = param.deref_mut() {
                transform.get_mut(entity).unwrap().scale = *scale;
            }
        });
    }
}

impl OpShouldExecute for MeshOpGrid {
    type Param = ();

    fn should_execute<'w>(
        entity: Entity,
        param: &mut SystemParamItem<'w, '_, Self::Param>,
    ) -> bool {
        true
    }
}

impl OpExecute for MeshOpGrid {
    fn execute(&self, entity: Entity, world: &mut World) {}
}

impl OpOnConnect for MeshOpGrid {
    type Param = ();

    fn on_connect<'w>(
        entity: Entity,
        event: Connect,
        fully_connected: bool,
        param: &mut SystemParamItem<'w, '_, Self::Param>,
    ) {
    }
}

impl OpOnDisconnect for MeshOpGrid {
    type Param = ();

    fn on_disconnect<'w>(
        entity: Entity,
        event: Disconnect,
        fully_connected: bool,
        param: &mut SystemParamItem<'w, '_, Self::Param>,
    ) {
    }
}

impl Op for MeshOpGrid {
    const INPUTS: usize = 0;
    const OUTPUTS: usize = 1;
    const CATEGORY: &'static str = CATEGORY;
    type OpType = OpType<MeshOpGrid>;
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Grid {
    rows: usize,
    columns: usize,
    normal: Dir3,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            rows: 10,
            columns: 10,
            normal: Dir3::Y,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct GridMeshBuilder {
    /// The [`Grid`] shape.
    pub grid: Grid,
    /// Half the size of the grid mesh.
    pub half_size: Vec2,
}

impl Default for GridMeshBuilder {
    fn default() -> Self {
        Self {
            grid: Grid::default(),
            half_size: Vec2::ONE,
        }
    }
}

impl GridMeshBuilder {
    /// Creates a new [`GridMeshBuilder`] from a given normal and size.
    #[inline]
    pub fn new(normal: Dir3, size: Vec2) -> Self {
        Self {
            grid: Grid {
                normal,
                ..Default::default()
            },
            half_size: size / 2.0,
        }
    }

    /// Creates a new [`GridMeshBuilder`] from the given size, with the normal pointing upwards.
    #[inline]
    pub fn from_size(size: Vec2) -> Self {
        Self {
            half_size: size / 2.0,
            ..Default::default()
        }
    }

    /// Creates a new [`GridMeshBuilder`] from the given length, with the normal pointing upwards,
    /// and the resulting [`GridMeshBuilder`] being a square.
    #[inline]
    pub fn from_length(length: f32) -> Self {
        Self {
            half_size: Vec2::splat(length) / 2.0,
            ..Default::default()
        }
    }

    /// Sets the normal of the grid, aka the direction the grid is facing.
    #[inline]
    #[doc(alias = "facing")]
    pub fn normal(mut self, normal: Dir3) -> Self {
        self.grid = Grid {
            normal,
            ..Default::default()
        };
        self
    }

    /// Sets the size of the grid mesh.
    #[inline]
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.half_size = Vec2::new(width, height) / 2.0;
        self
    }

    /// Builds a [`Mesh`] based on the configuration in `self`.
    pub fn build(&self) -> Mesh {
        let rotation = Quat::from_rotation_arc(Vec3::Y, *self.grid.normal);
        let mut positions = Vec::new();
        let mut normals = Vec::new();
        let mut uvs = Vec::new();
        let mut indices = Vec::new();

        for row in 0..self.grid.rows {
            for column in 0..self.grid.columns {
                let x = (column as f32 / self.grid.columns as f32 - 0.5) * self.half_size.x;
                let z = (row as f32 / self.grid.rows as f32 - 0.5) * self.half_size.y;
                let position = rotation * Vec3::new(x, 0.0, z);
                let normal = rotation * Vec3::Y;
                let uv = Vec2::new(
                    column as f32 / self.grid.columns as f32,
                    row as f32 / self.grid.rows as f32,
                );
                positions.push(position.to_array());
                normals.push(normal.to_array());
                uvs.push(uv.to_array());
            }
        }

        // Write the indices
        for row in 0..self.grid.rows - 1 {
            for column in 0..self.grid.columns - 1 {
                let i = row * self.grid.columns + column;
                let j = i + 1;
                let k = i + self.grid.columns;
                let l = k + 1;
                indices.push(i as u32);
                indices.push(j as u32);
                indices.push(k as u32);
                indices.push(k as u32);
                indices.push(j as u32);
                indices.push(l as u32);
            }
        }

        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_indices(Indices::U32(indices))
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    }
}

impl Meshable for Grid {
    type Output = GridMeshBuilder;

    fn mesh(&self) -> Self::Output {
        GridMeshBuilder {
            grid: *self,
            ..Default::default()
        }
    }
}

impl From<Grid> for Mesh {
    fn from(grid: Grid) -> Self {
        grid.mesh().build()
    }
}

impl From<GridMeshBuilder> for Mesh {
    fn from(grid: GridMeshBuilder) -> Self {
        grid.build()
    }
}
