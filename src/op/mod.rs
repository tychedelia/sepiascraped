use std::fmt::Debug;
use std::marker::PhantomData;

use bevy::ecs::system::{ReadOnlySystemParam, StaticSystemParam, SystemParam, SystemParamItem};
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};
use bevy::utils::HashMap;

use crate::event::SpawnOp;
use crate::index::UniqueIndexPlugin;
use crate::param::ParamBundle;
use crate::{OpName, Sets};

pub mod component;
pub mod material;
pub mod mesh;
pub mod texture;

#[derive(Default)]
pub struct OpPlugin<T: Op> {
    _marker: std::marker::PhantomData<T>,
}

impl<T> Plugin for OpPlugin<T>
where
    T: Op + Component + Send + Sync + Debug + 'static,
{
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn::<T>.in_set(Sets::Graph),
                update::<T>.in_set(Sets::Params),
            )
                .chain(),
        );
    }
}

#[derive(Component, Deref, DerefMut, Copy, Clone, PartialEq, Eq, Hash, Debug, Ord, PartialOrd)]
pub struct OpRef(pub Entity);

#[derive(Component, Clone, ExtractComponent, Default, Debug)]
pub struct OpType<T: Debug + Sync + Send + 'static>(PhantomData<T>);

#[derive(Component, Clone, Debug)]
pub struct OpTypeName(pub String);

impl<T> OpType<T>
where
    T: Debug + Sync + Send + 'static,
{
    pub fn name() -> &'static str {
        std::any::type_name::<T>().split("::").nth(4).unwrap()
    }
}

#[derive(Component, Default, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct OpCategory(pub &'static str);

impl OpCategory {
    pub fn to_color(&self) -> Color {
        match self.0 {
            "Component" => Color::SILVER,
            "Material" => Color::SALMON,
            "Mesh" => Color::NAVY,
            "Texture" => Color::PURPLE,
            _ => panic!("Unknown category: {}", self.0),
        }
    }

    pub fn is_component(&self) -> bool {
        self.0 == "Component"
    }

    pub fn is_material(&self) -> bool {
        self.0 == "Material"
    }

    pub fn is_mesh(&self) -> bool {
        self.0 == "Mesh"
    }

    pub fn is_texture(&self) -> bool {
        self.0 == "Texture"
    }
}

#[derive(Component, ExtractComponent, Clone, Default, Debug)]
pub struct OpInputs {
    pub(crate) count: usize,
    pub(crate) connections: HashMap<Entity, Handle<Image>>,
}

impl OpInputs {
    pub fn is_fully_connected(&self) -> bool {
        self.count == 0 || self.connections.len() == self.count
    }
}

#[derive(Component, Default)]
pub struct OpOutputs {
    pub(crate) count: usize,
}

#[derive(Component, Clone, Debug, Deref, DerefMut, ExtractComponent, Default)]
pub struct OpImage(pub Handle<Image>);

impl OpImage {
    pub fn new_image(width: u32, height: u32) -> Image {
        let size = Extent3d {
            width,
            height,
            ..default()
        };

        let mut image = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            ..default()
        };

        image.resize(size);

        image
    }
}


pub trait Op {
    const INPUTS: usize = 0;
    const OUTPUTS: usize = 0;
    const CATEGORY: &'static str;

    /// The type of the op.
    type OpType: Debug + Component + ExtractComponent + Send + Sync + 'static;
    /// The update parameter.
    type UpdateParam: SystemParam + 'static;
    /// The bundle parameter.
    type BundleParam: SystemParam + 'static;
    /// The bundle type of this op;
    type Bundle: Bundle;

    /// Update the op, i.e. to apply updates from the UI.
    fn update<'w>(entity: Entity, param: &mut SystemParamItem<'w, '_, Self::UpdateParam>);

    /// Create the op bundle.
    fn create_bundle<'w>(
        entity: Entity,
        param: &mut SystemParamItem<'w, '_, Self::BundleParam>,
    ) -> Self::Bundle;

    /// Get the parameters for the op.
    fn params(bundle: &Self::Bundle) -> Vec<ParamBundle>;
}

fn update<'w, 's, T>(
    ops_q: Query<Entity, With<OpType<T>>>,
    param: StaticSystemParam<T::UpdateParam>,
) where
    T: Op + Component + Debug + Send + Sync + 'static,
{
    let mut param = param.into_inner();
    for entity in ops_q.iter() {
        T::update(entity, &mut param);
    }
}

fn spawn<'w, 's, T>(
    mut commands: Commands,
    param: StaticSystemParam<T::BundleParam>,
    added_q: Query<Entity, Added<OpType<T>>>,
    mut spawn_op_evt: EventWriter<SpawnOp>,
) where
    T: Op + Component + Debug + Send + Sync + 'static,
{
    let mut param = param.into_inner();

    for entity in added_q.iter() {
        let bundle = T::create_bundle(entity, &mut param);
        let params = T::params(&bundle);
        commands
            .entity(entity)
            .insert((
                OpCategory(T::CATEGORY),
                OpTypeName(OpType::<T>::name().to_string()),
                bundle,
            ))
            .with_children(|parent| {
                params.into_iter().for_each(|param| {
                    parent.spawn((OpRef(parent.parent_entity()), param));
                });
            });

        spawn_op_evt.send(SpawnOp(entity));
    }
}

#[derive(Resource, Clone, Default)]
pub struct OpDefaultImage(pub Handle<Image>);
