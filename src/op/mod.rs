use std::fmt::Debug;
use std::marker::PhantomData;

use bevy::ecs::system::{ReadOnlySystemParam, StaticSystemParam, SystemParam, SystemParamItem};
use bevy::prelude::*;
use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::utils::HashMap;

use crate::event::SpawnOp;
use crate::index::UniqueIndexPlugin;
use crate::param::ParamBundle;
use crate::ui::event::{Connect, Disconnect};
use crate::ui::graph::GraphRef;
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
    T: Op + Component + ExtractComponent + Send + Sync + Debug + 'static,
{
    fn build(&self, app: &mut App) {
        app
            .add_systems(
            Update,
            (
                (spawn::<T>, on_connect::<T>, on_disconnect::<T>).in_set(Sets::Graph),
                update::<T>.in_set(Sets::Params),
            ).chain(),
        )
        ;
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
pub struct OpInputConfig {
    pub count: usize,
}

#[derive(Component, ExtractComponent, Clone, Default, Debug)]
pub struct OpInputs<T>
where
    T: Op + Component + ExtractComponent + Debug + Send + Sync + 'static,
{
    pub count: usize,
    pub connections: HashMap<Entity, T::ConnectionData>,
}

impl<T> OpInputs<T>
where
    T: Op + Component + ExtractComponent + Debug + Send + Sync + 'static,
{
    pub fn is_fully_connected(&self) -> bool {
        self.count == 0 || self.connections.len() == self.count
    }
}

#[derive(Component, ExtractComponent, Clone, Default, Debug)]
pub struct OpOutputConfig {
    pub count: usize,
}

#[derive(Component, Default)]
pub struct OpOutputs {
    pub count: usize,
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
    /// The on connect parameter.
    type OnConnectParam: SystemParam + 'static;
    /// The connection data parameter.
    type ConnectionDataParam: SystemParam + 'static;
    /// The on disconnect parameter.
    type OnDisconnectParam: SystemParam + 'static;
    /// The bundle type of this op;
    type Bundle: Bundle;
    /// Connection data.
    type ConnectionData: Default + Clone + Debug + Send + Sync + 'static;

    /// Update the op, i.e. to apply updates from the UI.
    fn update<'w>(entity: Entity, param: &mut SystemParamItem<'w, '_, Self::UpdateParam>);

    /// Create the op bundle.
    fn create_bundle<'w>(
        entity: Entity,
        param: &mut SystemParamItem<'w, '_, Self::BundleParam>,
    ) -> Self::Bundle;

    /// Get the parameters for the op.
    fn params(bundle: &Self::Bundle) -> Vec<ParamBundle>;

    fn on_connect<'w>(
        entity: Entity,
        event: Connect,
        fully_connected: bool,
        param: &mut SystemParamItem<'w, '_, Self::OnConnectParam>,
    ) {
    }

    fn on_disconnect<'w>(
        entity: Entity,
        event: Disconnect,
        fully_connected: bool,
        param: &mut SystemParamItem<'w, '_, Self::OnDisconnectParam>,
    ) {
    }

    fn connection_data<'w>(
        entity: Entity,
        param: &mut SystemParamItem<'w, '_, Self::ConnectionDataParam>,
    ) -> Self::ConnectionData;
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

fn on_connect<T>(
    mut ev_connect: EventReader<Connect>,
    mut op_q: Query<&mut OpInputs<T>, With<OpType<T>>>,
    input_q: Query<&OpImage>,
    connection_data_param: StaticSystemParam<T::ConnectionDataParam>,
    param: StaticSystemParam<T::OnConnectParam>,
) where
    T: Op + Component + ExtractComponent + Debug + Send + Sync + 'static,
{
    let mut param = param.into_inner();
    let mut connection_data_param = connection_data_param.into_inner();
    for ev in ev_connect.read() {
        if let Ok(mut input) = op_q.get_mut(ev.input) {
            input.connections.insert(
                ev.output,
                T::connection_data(ev.output, &mut connection_data_param),
            );
            T::on_connect(ev.input, *ev, input.is_fully_connected(), &mut param);
        }
    }
}

fn on_disconnect<T>(
    mut commands: Commands,
    mut ev_disconnect: EventReader<Disconnect>,
    mut op_q: Query<&mut OpInputs<T>, With<OpType<T>>>,
    param: StaticSystemParam<T::OnDisconnectParam>,
) where
    T: Op + Component + ExtractComponent + Debug + Send + Sync + 'static,
{
    let mut param = param.into_inner();
    for ev in ev_disconnect.read() {
        if let Ok(mut input) = op_q.get_mut(ev.input) {
            input.connections.remove(&ev.output);
            T::on_disconnect(ev.input, *ev, input.is_fully_connected(), &mut param);
        }
    }
}

#[derive(Resource, Clone, Default)]
pub struct OpDefaultImage(pub Handle<Image>);
