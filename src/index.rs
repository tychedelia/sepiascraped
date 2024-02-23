use std::collections::BTreeMap;
use std::hash::Hash;

use bevy::app::*;
use bevy::prelude::{
    Added, Component, Deref, DerefMut, Entity, Query, RemovedComponents, ResMut, Resource,
};

#[derive(Default)]
pub struct IndexPlugin<T> {
    _marker: std::marker::PhantomData<T>,
}

impl<T> Plugin for IndexPlugin<T>
where
    T: Default + Component + Clone + Ord + Hash + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.init_resource::<Index<T>>()
            .add_systems(Update, (insert_index::<T>, remove_index::<T>));
    }
}

#[derive(Resource, Default, Deref, DerefMut)]
pub struct Index<T>(BTreeMap<T, Entity>);

fn insert_index<T>(mut index: ResMut<Index<T>>, insert_q: Query<(Entity, &T), Added<T>>)
where
    T: Component + Clone + Ord + Hash + Send + Sync + 'static,
{
    for (entity, component) in insert_q.iter() {
        index.insert(component.clone(), entity);
    }
}

fn remove_index<T>(
    mut index: ResMut<Index<T>>,
    mut removed_evt: RemovedComponents<T>,
    reverse_lookup_q: Query<&T>,
) where
    T: Component + Clone + Ord + Hash + Send + Sync + 'static,
{
    for entity in removed_evt.read() {
        let component = reverse_lookup_q.get(entity).unwrap();
        index.remove(component);
    }
}
