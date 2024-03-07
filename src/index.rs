use std::collections::BTreeMap;
use std::fmt::Debug;
use std::hash::Hash;

use bevy::app::*;
use bevy::prelude::{
    Added, Component, Deref, DerefMut, Entity, Query, RemovedComponents, ResMut, Resource,
};

#[derive(Default)]
pub struct UniqueIndexPlugin<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T> Plugin for UniqueIndexPlugin<T>
where
    T: Component + Debug + Clone + Ord + Hash + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(UniqueIndex::<T>(BTreeMap::new()))
            .add_systems(First, (insert_unique_index::<T>, remove_unique_index::<T>));
    }
}

impl<T> UniqueIndexPlugin<T> {
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub(crate) struct UniqueIndex<T>(BTreeMap<T, Entity>);

fn insert_unique_index<T>(
    mut index: ResMut<UniqueIndex<T>>,
    insert_q: Query<(Entity, &T), Added<T>>,
) where
    T: Component + Debug + Clone + Ord + Hash + Send + Sync + 'static,
{
    for (entity, component) in insert_q.iter() {
        if index.0.contains_key(component) {
            panic!("Index already contains component: {:?}", component);
        }
        index.insert(component.clone(), entity);
    }
}

fn remove_unique_index<T>(
    mut index: ResMut<UniqueIndex<T>>,
    mut removed_evt: RemovedComponents<T>,
    reverse_lookup_q: Query<&T>,
) where
    T: Component + Debug + Clone + Ord + Hash + Send + Sync + 'static,
{
    for entity in removed_evt.read() {
        if let Ok(component) = reverse_lookup_q.get(entity) {
            index.remove(component);
        }
    }
}

#[derive(Default)]
pub struct IndexPlugin<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T> Plugin for IndexPlugin<T>
where
    T: Component + Debug + Clone + Ord + Hash + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(Index::<T>(BTreeMap::new()))
            .add_systems(First, (insert_index::<T>, remove_index::<T>));
    }
}

impl<T> IndexPlugin<T> {
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub(crate) struct Index<T>(BTreeMap<T, Vec<Entity>>);

fn insert_index<T>(mut index: ResMut<Index<T>>, insert_q: Query<(Entity, &T), Added<T>>)
where
    T: Component + Debug + Clone + Ord + Hash + Send + Sync + 'static,
{
    for (entity, component) in insert_q.iter() {
        index
            .entry(component.clone())
            .or_insert_with(Vec::new)
            .push(entity);
    }
}

fn remove_index<T>(
    mut index: ResMut<Index<T>>,
    mut removed_evt: RemovedComponents<T>,
    reverse_lookup_q: Query<&T>,
) where
    T: Component + Debug + Clone + Ord + Hash + Send + Sync + 'static,
{
    for entity in removed_evt.read() {
        let component = reverse_lookup_q.get(entity).unwrap();
        index
            .entry(component.clone())
            .or_insert_with(Vec::new)
            .retain(|e| *e != entity);
    }
}

#[derive(Default)]
pub struct CompositeIndex2Plugin<T, U> {
    _marker: std::marker::PhantomData<(T, U)>,
}

impl<T, U> CompositeIndex2Plugin<T, U> {
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T, U> Plugin for CompositeIndex2Plugin<T, U>
where
    T: Component + Debug + Clone + Ord + Hash + Send + Sync + 'static,
    U: Component + Debug + Clone + Ord + Hash + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(CompositeIndex2::<T, U>(BTreeMap::new()))
            .add_systems(
                First,
                (
                    insert_composite_index::<T, U>,
                    remove_composite_index::<T, U>,
                ),
            );
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub(crate) struct CompositeIndex2<T, U>(BTreeMap<(T, U), Entity>);

fn insert_composite_index<T, U>(
    mut index: ResMut<CompositeIndex2<T, U>>,
    insert_q: Query<(Entity, &T, &U), (Added<T>, Added<U>)>,
) where
    T: Component + Debug + Clone + Ord + Hash + Send + Sync + 'static,
    U: Component + Debug + Clone + Ord + Hash + Send + Sync + 'static,
{
    for (entity, component1, component2) in insert_q.iter() {
        if index
            .0
            .contains_key(&(component1.clone(), component2.clone()))
        {
            panic!(
                "Index already contains components: {:?}, {:?}",
                component1, component2
            );
        }
        index.insert((component1.clone(), component2.clone()), entity);
    }
}

fn remove_composite_index<T, U>(
    mut index: ResMut<CompositeIndex2<T, U>>,
    mut removed1_evt: RemovedComponents<T>,
    mut removed2_evt: RemovedComponents<U>,
    reverse_lookup_q: Query<(&T, &U)>,
) where
    T: Component + Debug + Clone + Ord + Hash + Send + Sync + 'static,
    U: Component + Debug + Clone + Ord + Hash + Send + Sync + 'static,
{
    for entity in removed1_evt.read() {
        if let Ok((c1, c2)) = reverse_lookup_q.get(entity) {
            index.remove(&(c1.clone(), c2.clone()));
        }
    }
    for entity in removed2_evt.read() {
        if let Ok((c1, c2)) = reverse_lookup_q.get(entity) {
            index.remove(&(c1.clone(), c2.clone()));
        }
    }
}
