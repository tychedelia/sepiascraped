use bevy::prelude::*;
use bevy_mod_picking::events::{Down, Pointer};
use bevy_mod_picking::prelude::ListenerInput;

#[derive(Event, Deref)]
pub struct ClickNode(Entity);

impl From<ListenerInput<Pointer<Down>>> for ClickNode {
    fn from(event: ListenerInput<Pointer<Down>>) -> Self {
        ClickNode(event.target)
    }
}

#[derive(Event)]
pub struct Deselect;

#[derive(Event, Debug)]
pub struct Connect {
    pub from: Entity,
    pub to: Entity,
}