use std::fmt::Debug;

use crate::op::mesh::types::cuboid::MeshOpCuboidPlugin;
use crate::op::mesh::types::grid::MeshOpGridPlugin;
use crate::op::mesh::types::noise::MeshOpNoisePlugin;
use crate::op::mesh::types::plane::MeshOpPlanePlugin;
use crate::op::{Op, OpImage, OpInputs, OpOutputs};
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::mesh::{Indices, VertexAttributeValues};
use bevy::render::primitives::Aabb;
use bevy::render::render_resource::encase::internal::WriteInto;
use bevy::render::render_resource::ShaderType;
use bevy::render::view::RenderLayers;

use crate::op::texture::TextureOp;

pub mod types;

pub const CATEGORY: &str = "Mesh";

pub struct MeshPlugin;

impl Plugin for MeshPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MeshOpCuboidPlugin,
            MeshOpNoisePlugin,
            MeshOpPlanePlugin,
            MeshOpGridPlugin,
        ));
    }
}

#[derive(Component, Deref, DerefMut, Clone, Debug)]
pub struct MeshOpHandle(pub Handle<Mesh>);

#[derive(Component, ExtractComponent, Deref, DerefMut, Clone, Debug, Default)]
pub struct MeshOpInputMeshes(pub Vec<Handle<Mesh>>);

#[derive(Bundle)]
pub struct MeshOpBundle {
    mesh: MeshOpHandle,
    pbr: PbrBundle,
    image: OpImage,
    inputs: OpInputs,
    outputs: OpOutputs,
}

pub trait MeshExt {
    fn points(&self) -> &[[f32; 3]];
    fn points_mut(&mut self) -> &mut Vec<[f32; 3]>;
    fn colors(&self) -> &[[f32; 4]];
    fn colors_mut(&mut self) -> &mut Vec<[f32; 4]>;
    fn tex_coords(&self) -> &[[f32; 2]];
    fn tex_coords_mut(&mut self) -> &mut Vec<[f32; 2]>;
    fn get_index(&self, index: usize) -> u32;
    fn count_indices(&self) -> usize;
    fn push_index(&mut self, index: u32);
}

impl MeshExt for Mesh {
    fn points(&self) -> &[[f32; 3]] {
        let points = self
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("Mesh must have ATTRIBUTE_POSITION attribute");

        match points {
            VertexAttributeValues::Float32x3(points) => points,
            _ => panic!("Mesh ATTRIBUTE_POSITION attribute must be of type Float32x3"),
        }
    }

    fn points_mut(&mut self) -> &mut Vec<[f32; 3]> {
        let points = self
            .attribute_mut(Mesh::ATTRIBUTE_POSITION)
            .expect("Mesh must have ATTRIBUTE_POSITION attribute");

        match points {
            VertexAttributeValues::Float32x3(points) => points,
            _ => panic!("Mesh ATTRIBUTE_POSITION attribute must be of type Float32x3"),
        }
    }

    fn colors(&self) -> &[[f32; 4]] {
        let colors = self
            .attribute(Mesh::ATTRIBUTE_COLOR)
            .expect("Mesh must have ATTRIBUTE_COLOR attribute");

        match colors {
            VertexAttributeValues::Float32x4(colors) => colors,
            _ => panic!("Mesh ATTRIBUTE_COLOR attribute must be of type Float32x4"),
        }
    }

    fn colors_mut(&mut self) -> &mut Vec<[f32; 4]> {
        let colors = self
            .attribute_mut(Mesh::ATTRIBUTE_COLOR)
            .expect("Mesh must have ATTRIBUTE_COLOR attribute");

        match colors {
            VertexAttributeValues::Float32x4(colors) => colors,
            _ => panic!("Mesh ATTRIBUTE_COLOR attribute must be of type Float32x4"),
        }
    }

    fn tex_coords(&self) -> &[[f32; 2]] {
        let tex_coords = self
            .attribute(Mesh::ATTRIBUTE_UV_0)
            .expect("Mesh must have ATTRIBUTE_UV_0 attribute");

        match tex_coords {
            VertexAttributeValues::Float32x2(tex_coords) => tex_coords,
            _ => panic!("Mesh ATTRIBUTE_UV_0 attribute must be of type Float32x2"),
        }
    }

    fn tex_coords_mut(&mut self) -> &mut Vec<[f32; 2]> {
        let tex_coords = self
            .attribute_mut(Mesh::ATTRIBUTE_UV_0)
            .expect("Mesh must have ATTRIBUTE_UV_0 attribute");

        match tex_coords {
            VertexAttributeValues::Float32x2(tex_coords) => tex_coords,
            _ => panic!("Mesh ATTRIBUTE_UV_0 attribute must be of type Float32x2"),
        }
    }

    fn get_index(&self, index: usize) -> u32 {
        match self.indices() {
            Some(Indices::U32(indices)) => indices[index],
            _ => panic!("Mesh must have U32 indices"),
        }
    }

    fn count_indices(&self) -> usize {
        match self.indices() {
            Some(Indices::U32(indices)) => indices.len(),
            _ => panic!("Mesh must have U32 indices"),
        }
    }

    fn push_index(&mut self, index: u32) {
        match self.indices_mut() {
            Some(Indices::U32(indices)) => indices.push(index),
            _ => panic!("Mesh must have U32 indices"),
        }
    }
}
