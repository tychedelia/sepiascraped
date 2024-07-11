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
        app.insert_resource(UniqueIndex::<T>(BTreeMap::new()));

        let mut world = app.world_mut();
        world
            .register_component_hooks::<T>()
            .on_insert(|mut world, entity, component_id| {
                let component = world.get::<T>(entity).unwrap().clone();
                let mut index = world.resource_mut::<UniqueIndex<T>>();
                if index.0.contains_key(&component) {
                    panic!("Index already contains component: {:?}", component);
                }
                index.insert(component.clone(), entity);
            })
            .on_remove(|mut world, entity, component_id| {
                let component = world.get::<T>(entity).unwrap().clone();
                let mut index = world.resource_mut::<UniqueIndex<T>>();
                index.remove(&component);
            });
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
pub struct UniqueIndex<T>(BTreeMap<T, Entity>);

#[derive(Default)]
pub struct IndexPlugin<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T> Plugin for IndexPlugin<T>
where
    T: Component + Debug + Clone + Ord + Hash + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(Index::<T>(BTreeMap::new()));

        let mut world = app.world_mut();
        world
            .register_component_hooks::<T>()
            .on_add(|mut world, entity, component_id| {
                let component = world.get::<T>(entity).unwrap().clone();
                let mut index = world.resource_mut::<Index<T>>();
                index
                    .entry(component)
                    .or_insert_with(Vec::new)
                    .push(entity);
            })
            .on_remove(|mut world, entity, component_id| {
                let component = world.get::<T>(entity).unwrap().clone();
                let mut index = world.resource_mut::<Index<T>>();
                index
                    .entry(component)
                    .or_insert_with(Vec::new)
                    .retain(|e| *e != entity);
            });
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
pub struct Index<T>(BTreeMap<T, Vec<Entity>>);

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
        app.insert_resource(CompositeIndex2::<T, U>(BTreeMap::new()));

        let mut world = app.world_mut();
        world
            .register_component_hooks::<T>()
            .on_add(|mut world, entity, component_id| {
                let component1 = world.get::<T>(entity).unwrap().clone();
                let Some(component2) = world.get::<U>(entity) else {
                    return;
                };
                let component2 = component2.clone();
                let mut index = world.resource_mut::<CompositeIndex2<T, U>>();
                if let Some(index_e) = index.0.get(&(component1.clone(), component2.clone())) {
                    if *index_e != entity {
                        panic!(
                            "Index already contains components: {:?}, {:?}",
                            component1, component2
                        );
                    }
                }
                index.insert((component1, component2), entity);
            })
            .on_remove(|mut world, entity, component_id| {
                let component1 = world.get::<T>(entity).unwrap().clone();
                let component2 = world.get::<U>(entity).unwrap().clone();
                let mut index = world.resource_mut::<CompositeIndex2<T, U>>();
                index.remove(&(component1, component2));
            });

        world
            .register_component_hooks::<U>()
            .on_add(|mut world, entity, component_id| {
                let component1 = world.get::<T>(entity).unwrap().clone();
                let component2 = world.get::<U>(entity).unwrap().clone();
                let mut index = world.resource_mut::<CompositeIndex2<T, U>>();
                if let Some(index_e) = index.0.get(&(component1.clone(), component2.clone())) {
                    if *index_e != entity {
                        panic!(
                            "Index already contains components: {:?}, {:?}",
                            component1, component2
                        );
                    }
                }

                index.insert((component1, component2), entity);
            })
            .on_remove(|mut world, entity, component_id| {
                let component1 = world.get::<T>(entity).unwrap().clone();
                let component2 = world.get::<U>(entity).unwrap().clone();
                let mut index = world.resource_mut::<CompositeIndex2<T, U>>();
                index.remove(&(component1.clone(), component2.clone()));
            });
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct CompositeIndex2<T, U>(BTreeMap<(T, U), Entity>);
