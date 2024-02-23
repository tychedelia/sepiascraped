use bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell;
use bevy::prelude::*;
use steel::rvals::Custom;
use steel::steel_vm::engine::Engine;
use steel::steel_vm::register_fn::RegisterFn;
use steel::SteelVal;
use steel_derive::Steel;
use crate::index::Index;
use crate::OpName;

pub struct ScriptPlugin;

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (update, print_counts));
    }
}

#[derive(Default, Deref, DerefMut)]
struct LispEngine(Engine);

#[derive(Component)]
pub struct Counter {
    count: u32,
}

#[derive(Debug, Steel, PartialEq, Clone)]
pub struct EntityRef(Entity);

#[derive(Debug, Steel, Clone)]
struct WorldHolder(*mut World);

impl WorldHolder {
    fn world(&self) -> &World {
        unsafe { &*self.0 }
    }

    fn world_mut(&mut self) -> &mut World {
        unsafe { &mut *self.0 }
    }
}

trait WorldScope {
    fn with_world_scope<F, R>(&mut self, world: UnsafeWorldCell<'_>, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R;
}

impl WorldScope for Engine {
    fn with_world_scope<F, R>(&mut self, world: UnsafeWorldCell<'_>, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let world = WorldHolder(unsafe { world.world_mut() });
        self.register_external_value("world", world);
        f(self)
        // TODO: remove the world from the engine
    }
}

fn setup(world: &mut World) {
    let engine = Engine::new();
    world.insert_non_send_resource(LispEngine(engine));
    let world_cell = world.as_unsafe_world_cell();
    unsafe {
        world_cell
            .world_mut()
            .get_non_send_resource_mut::<LispEngine>()
            .unwrap()
            .with_world_scope(world_cell, |engine| {
                engine
                    .register_fn("op", op)
                    .register_fn("param", param)
                    .register_fn("get-count", get_count)
                    .register_fn("set-count", set_count)
                    .register_fn("new-counter", new_counter);
                let prog = engine.emit_raw_program_no_path(
                    r#"
                            (define counter (new-counter world))
                    "#)
                    .unwrap();
                engine.run_raw_program(prog).unwrap();
            });
    }
}

fn update(world: &mut World) {
    let world_cell = world.as_unsafe_world_cell();
    unsafe {
        world_cell
            .world_mut()
            .get_non_send_resource_mut::<LispEngine>()
            .unwrap()
            .with_world_scope(world_cell, |engine| {
                engine.compile_and_run_raw_program(
                    r#"
                            (op world "ramp4")
                            (set-count world counter (+ (get-count world counter) 1))
                    "#)
                    .unwrap();
            });
    }
}

fn op(world: &WorldHolder, name: String) -> Option<EntityRef> {
    let world = world.world();
    let index = world.get_resource::<Index<OpName>>().unwrap();
    let entity = index.get(&OpName(name));
    if let Some(entity) = entity {
        Some(EntityRef(entity.clone()))
    } else {
        None
    }
}

fn param(world: &WorldHolder, entity: EntityRef, name: String) -> SteelVal {
    let world = world.world();
    let entity = world.get_entity(entity.0).unwrap();
    let children = entity.get::<Children>().unwrap();

}

fn new_counter(mut world: &mut WorldHolder) -> EntityRef {
    let world = world.world_mut();
    let entity = world.spawn(Counter { count: 0 }).id();
    EntityRef(entity)
}

fn get_count(world: &WorldHolder, entity: EntityRef) -> u32 {
    println!("Getting count");
    let world = world.world();
    let entity = world.get_entity(entity.0).unwrap();
    let counter = entity.get::<Counter>().unwrap();
    counter.count
}

fn set_count(world: &mut WorldHolder, entity: EntityRef, count: u32) {
    let world = world.world_mut();
    let mut entity = world.get_entity_mut(entity.0).unwrap();
    let mut counter = entity.get_mut::<Counter>().unwrap();
    counter.count = count;
}

fn print_counts(count_q: Query<&Counter>) {
    for count in count_q.iter() {
        println!("Count: {}", count.count);
    }
}
