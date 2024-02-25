use crate::index::{CompositeIndex2, UniqueIndex};
use crate::param::{ParamName, ParamValue};
use crate::OpName;
use bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell;
use bevy::prelude::*;
use steel::gc::unsafe_erased_pointers::CustomReference;
use steel::rvals::{Custom, SteelVector};
use steel::steel_vm::engine::Engine;
use steel::steel_vm::register_fn::RegisterFn;
use steel::SteelVal;
use steel_derive::Steel;
use crate::ui::graph::OpRef;

pub struct ScriptPlugin;

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, (update));
    }
}

#[derive(Default, Deref, DerefMut)]
struct LispEngine(Engine);

#[derive(Debug, Steel, PartialEq, Clone)]
pub struct EntityRef(Entity);

#[derive(Debug, Deref, DerefMut, Clone)]
struct WorldHolder<'w>(UnsafeWorldCell<'w>);

impl<'w> CustomReference for WorldHolder<'w> {}

steel::custom_reference!(WorldHolder<'a>);

fn setup(world: &mut World) {
    let mut engine = Engine::new();
    engine.register_value("*world*", SteelVal::Void);
    world.insert_non_send_resource(LispEngine(engine));
    let world_cell = world.as_unsafe_world_cell();
    unsafe {
        world_cell
            .world_mut()
            .get_non_send_resource_mut::<LispEngine>()
            .unwrap()
            .with_mut_reference::<WorldHolder, WorldHolder>(&mut WorldHolder(world_cell))
            .consume(|engine, args| {
                let world = args[0].clone();
                engine
                    .update_value("*world*", world)
                    .expect("TODO: panic message");
                engine.register_fn("-op", op).register_fn("-param", param);
                let prog = engine
                    .emit_raw_program_no_path(
                        r#"
                        (define (op name)
                            (-op *world* name))
                        (define (param entity name)
                            (when entity
                                (-param *world* entity name)))
                    "#,
                    )
                    .unwrap();
                engine.run_raw_program(prog).unwrap();
            });
    }
}

fn update(world: &mut World) {
    let world_cell = world.as_unsafe_world_cell();
    unsafe {
        let res = world_cell
            .world_mut()
            .get_non_send_resource_mut::<LispEngine>()
            .unwrap()
            .with_mut_reference::<WorldHolder, WorldHolder>(&mut WorldHolder(world_cell))
            .consume(|engine, args| {
                let world = args[0].clone();
                engine
                    .update_value("*world*", world)
                    .expect("TODO: panic message");

                engine.compile_and_run_raw_program(
                    r#"
                            (+ (param (op "ramp4") "Mode") 11)
                    "#,
                ).unwrap_or_else(|e| {
                    println!("Error: {:?}", e);
                    vec![SteelVal::Void]
                })
            });
        println!("res: {:?}", res);
    }
}

fn op(world: &mut WorldHolder, name: String) -> Option<EntityRef> {
    let world = unsafe { world.world() };
    let index = world.get_resource::<UniqueIndex<OpName>>().unwrap();
    let entity = index.get(&OpName(name));
    if let Some(entity) = entity {
        Some(EntityRef(entity.clone()))
    } else {
        None
    }
}

fn param(world: &mut WorldHolder, entity: EntityRef, name: String) -> SteelVal {
    let world = unsafe { world.world() };
    let index = world.get_resource::<CompositeIndex2<OpRef, ParamName>>().unwrap();
    let name = index.get(&(OpRef(entity.0), ParamName(name))).map_or(SteelVal::Void, |entity| {
        let value = world.get::<ParamValue>(*entity).unwrap();
        SteelVal::from(value.clone())
    });
    name
}

impl From<ParamValue> for SteelVal {
    fn from(value: ParamValue) -> Self {
        match value {
            ParamValue::None => SteelVal::Void,
            ParamValue::F32(x) => SteelVal::from(x),
            ParamValue::U32(x) => SteelVal::from(x),
            _ => unimplemented!(),
        }
    }
}
