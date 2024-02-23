use bevy::prelude::*;

#[derive(Event, Deref)]
pub struct SpawnOp(pub Entity);