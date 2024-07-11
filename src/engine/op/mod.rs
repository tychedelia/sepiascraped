use bevy::color::palettes::css::{NAVY, PURPLE, SALMON, SILVER};
use std::fmt::Debug;
use std::marker::PhantomData;

use crate::engine::graph::event::{Connect, Disconnect};
use crate::engine::graph::GraphState;
use crate::engine::op::component::ComponentPlugin;
use crate::engine::op::material::MaterialPlugin;
use crate::engine::op::mesh::MeshPlugin;
use crate::engine::op::texture::TexturePlugin;
use crate::engine::param::{validate, ParamBundle, ParamHash, Params};
use crate::index::UniqueIndexPlugin;
use crate::Sets;
use bevy::ecs::system::{ReadOnlySystemParam, StaticSystemParam, SystemParam, SystemParamItem};
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::utils::HashMap;

pub mod component;
pub mod material;
pub mod mesh;
pub mod texture;

#[derive(Default)]
pub struct OpsPlugin;

impl Plugin for OpsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ComponentPlugin,
            MaterialPlugin,
            MeshPlugin,
            TexturePlugin,
            UniqueIndexPlugin::<OpName>::default(),
        ))
        .add_systems(Last, ensure_despawn);
    }
}

#[derive(Default)]
pub struct OpPlugin<T: Op> {
    _marker: PhantomData<T>,
}

impl<T> Plugin for OpPlugin<T>
where
    T: Op + Component + ExtractComponent + Send + Sync + Debug + Default + 'static,
{
    fn build(&self, app: &mut App) {
        app.add_systems(Update, apply_deferred.after(spawn::<T>))
            .insert_resource(AmbientLight {
                color: Color::WHITE,
                brightness: 0.05,
            })
            .add_systems(
                Update,
                (
                    spawn::<T>.in_set(Sets::Spawn),
                    (
                        update::<T>,
                        should_execute::<T>,
                    )
                        .chain()
                        .before(validate)
                        .in_set(Sets::Params),
                    execute.in_set(Sets::Execute),
                ),
            );
    }
}

#[derive(Component, Deref, DerefMut, Copy, Clone, PartialEq, Eq, Hash, Debug, Ord, PartialOrd)]
pub struct OpRef(pub Entity);

#[derive(Component, Clone, ExtractComponent, Default, Debug)]
pub struct OpType<T: Debug + Sync + Send + 'static>(PhantomData<T>);

#[derive(Component, Clone, Debug, Default, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct OpTypeName(pub &'static str);

impl<T> OpType<T>
where
    T: Debug + Sync + Send + 'static,
{
    pub fn name() -> &'static str {
        std::any::type_name::<T>().split("::").nth(5).unwrap()
    }
}

#[derive(Component, Default, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct OpCategory(pub &'static str);

impl OpCategory {
    pub fn to_color(&self) -> Color {
        match self.0 {
            "Component" => Color::from(SILVER),
            "Material" => Color::from(SALMON),
            "Mesh" => Color::from(NAVY),
            "Texture" => Color::from(PURPLE),
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
    pub count: usize,
    pub connections: HashMap<u8, (Entity, u8)>,
    pub category: Option<OpCategory>,
}

impl OpInputs {
    fn new(count: usize) -> Self {
        Self {
            count,
            connections: HashMap::new(),
            category: None,
        }
    }

    pub fn is_fully_connected(&self) -> bool {
        self.count == 0 || self.connections.len() == self.count
    }

    pub fn with_category(mut self, category: &'static str) -> Self {
        self.category = Some(OpCategory(category));
        self
    }
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

#[derive(Component, Clone, Debug, Default)]
pub struct Execute;

#[derive(Component, Deref, DerefMut)]
pub struct OpDynExecute(Box<dyn OpExecute + Send + Sync + 'static>);

#[derive(Resource, Clone, Default)]
pub struct OpDefaultImage(pub Handle<Image>);

// ~~~~ Op ~~~~

/// How the op spawns, including the components that are required for its main op-type specific
/// behavior, as well as its parameters. Should handle any initialization required for setting
/// up the op.
pub trait OpSpawn {
    type Param: SystemParam + 'static;
    type Bundle: Bundle;

    /// Get the initial parameters for the op.
    fn params(bundle: &Self::Bundle) -> Vec<ParamBundle>;

    /// Create the op bundle.
    fn create_bundle<'w>(
        entity: Entity,
        param: &mut SystemParamItem<'w, '_, Self::Param>,
    ) -> Self::Bundle;
}

fn spawn<'w, 's, T>(
    mut commands: Commands,
    param: StaticSystemParam<<T as OpSpawn>::Param>,
    added_q: Query<Entity, Added<OpType<T>>>,
) where
    T: Op + Component + ExtractComponent + Debug + Default + Send + Sync + 'static,
{
    let mut param = param.into_inner();

    for entity in added_q.iter() {
        let bundle = T::create_bundle(entity, &mut param);
        let params = T::params(&bundle);
        commands
            .entity(entity)
            .insert((
                OpCategory(T::CATEGORY),
                OpDynExecute(Box::new(T::default())),
                OpTypeName(OpType::<T>::name()),
                ParamHash(0),
                bundle,
            ))
            .observe(on_connect::<T>)
            .observe(on_disconnect::<T>)
            .with_children(|parent| {
                params.into_iter().for_each(|param| {
                    parent.spawn((OpRef(parent.parent_entity()), param));
                });
            });
    }
}

/// Update behavior for the op. This should be cheap, mostly updating things like params
/// or other static data that does NOT rely on graph ordering. In other words, updates
/// data that is safe to update prior to execution.
pub trait OpUpdate {
    type Param: SystemParam + 'static;

    /// Update the op, i.e. to apply updates from the UI.
    fn update<'w>(entity: Entity, param: &mut SystemParamItem<'w, '_, Self::Param>);
}

fn update<'w, 's, T>(
    mut ops_q: Query<Entity, With<OpType<T>>>,
    param: StaticSystemParam<<T as OpUpdate>::Param>,
) where
    T: Op + Component + Debug + Send + Sync + 'static,
{
    let mut param = param.into_inner();
    for entity in ops_q.iter_mut() {
        T::update(entity, &mut param);
    }
}

/// Suggests whether the op should execute this frame. Note, the op will always execute if its
/// params change OR one of its dependencies change.
pub trait OpShouldExecute {
    type Param: SystemParam + 'static;

    /// Should this op execute? Note, returning false does not guarantee that the op will *not*
    /// execute, only suggesting that it does not need to.
    fn should_execute<'w>(
        entity: Entity,
        param: &mut SystemParamItem<'w, '_, Self::Param>,
    ) -> bool {
        false
    }
}

fn should_execute<'w, 's, T>(
    mut commands: Commands,
    mut ops_q: Query<(Entity, &mut ParamHash), With<OpType<T>>>,
    param: StaticSystemParam<<T as OpShouldExecute>::Param>,
    params: Params,
) where
    T: Op + Component + Debug + Send + Sync + 'static,
{
    let mut param = param.into_inner();
    for (entity, mut hash) in ops_q.iter_mut() {
        // Update the hash and mark this op as execute if the parameters have changed.
        let new_hash = params.hash(entity);
        if hash.0 != new_hash {
            hash.0 = new_hash;
            commands.entity(entity).insert(Execute);
        }

        // Mark the op as execute if it suggests we should
        if T::should_execute(entity, &mut param) {
            commands.entity(entity).insert(Execute);
        }
    }
}

/// Execute this operator, with exclusive access to [World]. Ops can assume that their
/// data dependencies have executed before them.
pub trait OpExecute {
    /// Execute the op.
    fn execute(&self, entity: Entity, world: &mut World);
}

fn execute(
    world: &mut World,
    // graph_state: Res<GraphState>,
    // mut ops_q: Query<(&mut OpDynExecute), With<Execute>>
) {
    let graph_state = world.get_resource::<GraphState>().unwrap();
    let sorted =
        petgraph::algo::toposort(&graph_state.graph, None).expect("There should not be a cycle");
    let entities = sorted
        .iter()
        .map(|idx| graph_state.entity_map[idx].clone())
        .collect::<Vec<Entity>>();

    unsafe {
        let world_cell = world.as_unsafe_world_cell();
        let mut ops_q = world_cell
            .world_mut()
            .query_filtered::<&OpDynExecute, With<Execute>>();
        for entity in entities {
            if let Ok(mut op) = ops_q.get(world_cell.world(), entity) {
                op.execute(entity, &mut world_cell.world_mut());
            }
        }
    }
}

/// Handler for when a new connection event occurs in the ui.
trait OpOnConnect {
    type Param: SystemParam + 'static;

    /// Run op connection logic.
    fn on_connect<'w>(
        entity: Entity,
        event: Connect,
        fully_connected: bool,
        param: &mut SystemParamItem<'w, '_, Self::Param>,
    );
}

fn on_connect<T>(
    mut trigger: Trigger<Connect>,
    mut op_q: Query<&mut OpInputs, With<OpType<T>>>,
    mut ev_disconnect: EventWriter<Disconnect>,
    param: StaticSystemParam<<T as OpOnConnect>::Param>,
) where
    T: Op + Component + ExtractComponent + Debug + Send + Sync + 'static,
{
    let ev = trigger.event();
    let mut param = param.into_inner();
    if let Ok(mut input) = op_q.get_mut(ev.input) {
        if let Some((prev, prev_port)) = input.connections.get(&ev.input_port) {
            ev_disconnect.send(Disconnect {
                output: *prev,
                input: ev.input,
                output_port: *prev_port,
                input_port: ev.input_port,
            });
        }

        input
            .connections
            .insert(ev.input_port, (ev.output, ev.output_port));
        T::on_connect(ev.input, *ev, input.is_fully_connected(), &mut param);
    }
}

/// Handler for when a new disconnection event occurs in the ui.
trait OpOnDisconnect {
    type Param: SystemParam + 'static;

    /// Run op disconnection logic.
    fn on_disconnect<'w>(
        entity: Entity,
        event: Disconnect,
        fully_connected: bool,
        param: &mut SystemParamItem<'w, '_, Self::Param>,
    );
}

fn on_disconnect<T>(
    trigger: Trigger<Disconnect>,
    mut op_q: Query<&mut OpInputs, With<OpType<T>>>,
    param: StaticSystemParam<<T as OpOnDisconnect>::Param>,
) where
    T: Op + Component + ExtractComponent + Debug + Send + Sync + 'static,
{
    let mut param = param.into_inner();
    let ev = trigger.event();
    if let Ok(mut input) = op_q.get_mut(ev.input) {
        input.connections.retain(|k, (e, _)| e != &ev.output);
        T::on_disconnect(ev.input, *ev, input.is_fully_connected(), &mut param);
    }
}

/// An op.
pub trait Op:
    OpSpawn + OpUpdate + OpShouldExecute + OpExecute + OpOnConnect + OpOnDisconnect
{
    /// The number of inputs this op provides.
    const INPUTS: usize = 0;
    /// The number of outputs this op provides.
    const OUTPUTS: usize = 0;
    /// The category of this op.
    const CATEGORY: &'static str;

    /// The type of the op.
    type OpType: Debug + Component + ExtractComponent + Send + Sync + 'static;
}

#[derive(Component, Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OpName(pub String);

fn ensure_despawn(
    mut commands: Commands,
    mut removed: RemovedComponents<OpName>,
    op_refs: Query<&OpRef>,
) {
    for entity in removed.read() {
        for op_ref in op_refs.iter() {
            if op_ref.0 == entity {
                warn!("Op {} was removed but not despawned", entity);
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
