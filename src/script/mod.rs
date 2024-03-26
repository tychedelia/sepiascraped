use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;
use std::sync::mpsc::{Receiver, TryRecvError};

use bevy::app::AppExit;
use bevy::asset::AssetContainer;
use bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell;
use bevy::prelude::*;
use bevy::utils::{warn, HashMap};
use colored::Colorize;
use rand::Rng;
use rustyline::error::ReadlineError;
use rustyline::highlight::MatchingBracketHighlighter;
use rustyline::validate::MatchingBracketValidator;
use rustyline::{DefaultEditor, Editor};
use steel::gc::unsafe_erased_pointers::CustomReference;
use steel::rvals::{CustomType, IntoSteelVal};
use steel::steel_vm::engine::Engine;
use steel::steel_vm::register_fn::RegisterFn;
use steel::SteelVal;
use steel_derive::Steel;

use crate::index::{CompositeIndex2, UniqueIndex};
use crate::op::component::types::window::ComponentOpWindow;
use crate::op::material::types::standard::MaterialOpStandard;
use crate::op::mesh::types::cuboid::MeshOpCuboid;
use crate::op::texture::types::composite::TextureOpComposite;
use crate::op::texture::types::noise::TextureOpNoise;
use crate::op::texture::types::ramp::TextureOpRamp;
use crate::op::texture::TextureOp;
use crate::op::{OpCategory, OpRef, OpType};
use crate::param::{ParamName, ParamValue, ScriptedParam, ScriptedParamError};
use crate::script::asset::{ProgramCache, Script, ScriptAssetPlugin};
use crate::script::helper::RustylineHelper;
use crate::ui::event::Connect;
use crate::ui::graph::{ConnectedTo, GraphRef, GraphState};
use crate::OpName;
use crate::Sets::Params;

mod asset;
mod helper;

pub struct ScriptPlugin;

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ScriptAssetPlugin)
            .add_systems(First, clear_touched)
            .add_systems(Last, (drop_untouched_entity, clear_untouched_params))
            .add_systems(Startup, setup)
            .add_systems(Update, update.in_set(Params));
    }
}

fn clear_touched(mut commands: Commands, touched_q: Query<Entity, With<ScriptTouched>>) {
    for entity in touched_q.iter() {
        commands.entity(entity).remove::<ScriptTouched>();
    }
}

fn drop_untouched_entity(
    mut commands: Commands,
    mut index: ResMut<UniqueIndex<OpName>>,
    touched_q: Query<(Entity, &GraphRef, &OpName), Without<ScriptTouched>>,
) {
    for (entity, graph_ref, op_name) in touched_q.iter() {
        commands.entity(entity).despawn_recursive();
        commands.entity(**graph_ref).despawn_recursive();
        // TODO: remove from index, this shouldn't be necessary
        index.remove(op_name);
    }
}

fn clear_untouched_params(
    mut commands: Commands,
    mut params_q: Query<Entity, (With<ScriptedParam>, Without<ScriptTouched>)>,
) {
    for entity in params_q.iter() {
        commands.entity(entity)
            .remove::<ScriptedParam>()
            .remove::<ScriptedParamError>();
    }
}

#[derive(Component)]
struct ScriptTouched;

#[derive(Deref, DerefMut)]
struct ReadLineEditor(Receiver<String>);

impl Default for ReadLineEditor {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut editor = Editor::new().expect("Unable to instantiate the repl!");
            editor.set_helper(Some(RustylineHelper::new(
                MatchingBracketHighlighter::default(),
                MatchingBracketValidator::default(),
            )));
            loop {
                let line = editor.readline(">");
                match line {
                    Ok(line) => {
                        editor
                            .add_history_entry(line.as_str())
                            .expect("Unable to add history entry");
                        tx.send(line).unwrap();
                    }
                    Err(ReadlineError::Interrupted) => {
                        error!("CTRL-C");
                        break;
                    }
                    Err(ReadlineError::Eof) => {
                        error!("CTRL-D");
                        break;
                    }
                    Err(err) => {
                        error!("Error: {:?}", err);
                        break;
                    }
                }
            }
        });

        Self(rx)
    }
}

#[derive(Default, Deref, DerefMut)]
struct LispEngine(Rc<RefCell<Engine>>);

#[derive(Debug, Deref, DerefMut, Steel, PartialEq, Clone)]
pub struct EntityRef(Entity);

#[derive(Debug, Deref, DerefMut, Clone)]
struct WorldHolder<'w>(UnsafeWorldCell<'w>);

impl<'w> CustomReference for WorldHolder<'w> {}

steel::custom_reference!(WorldHolder<'a>);

fn setup(world: &mut World) {
    let mut engine = Engine::new();
    engine.register_value("*world*", SteelVal::Void);
    engine.register_value("*time*", SteelVal::Void);
    let editor = ReadLineEditor::default();
    let engine = Rc::new(RefCell::new(engine));
    world.insert_non_send_resource(editor);
    world.insert_non_send_resource(LispEngine(engine));
    let curr_time = world.resource::<Time<Virtual>>().elapsed_seconds();
    let world_cell = world.as_unsafe_world_cell();
    unsafe {
        world_cell
            .world_mut()
            .get_non_send_resource_mut::<LispEngine>()
            .unwrap()
            .borrow_mut()
            .with_mut_reference::<WorldHolder, WorldHolder>(&mut WorldHolder(world_cell))
            .consume(|engine, args| {
                let world = args[0].clone();
                engine
                    .update_value("*world*", world)
                    .expect("TODO: panic message");
                engine
                    .update_value("*time*", SteelVal::from(curr_time))
                    .expect("TODO: panic message");
                engine
                    .register_fn("-op", op)
                    .register_fn("-op!", op_bang)
                    .register_fn("-param", param)
                    .register_fn("-param!", param_bang)
                    .register_fn("-connect!", connect_bang)
                    .register_fn("rand", rand);
                let prog = engine
                    .emit_raw_program_no_path(
                        r#"
                        ; get an op
                        (define (op name)
                            (-op *world* name))
                        ; create an op
                        (define (op! type name)
                            (-op! *world* type name))
                        ; get a param
                        (define (param entity name)
                            (when entity
                                (-param *world* entity name)))
                        ; set a param
                        (define (param! entity name val)
                            (when entity
                                (-param! *world* entity name val)))
                        ; connect two ops
                        (define (connect! output input)
                            (-connect! *world* output input))
                    "#,
                    )
                    .unwrap();
                engine.run_raw_program(prog).unwrap();
            });
    }
}

pub fn update(world: &mut World) {
    let curr_time = world.resource::<Time<Virtual>>().elapsed_seconds();
    let world_cell = world.as_unsafe_world_cell();
    unsafe {
        let mut editor = world_cell
            .world_mut()
            .get_non_send_resource_mut::<ReadLineEditor>()
            .unwrap();
        let line = match editor.try_recv() {
            Ok(line) => Some(line),
            Err(err) => {
                if err != TryRecvError::Empty {
                    error!("Error: {:?}", err);
                    world.send_event(AppExit);
                    return;
                } else {
                    None
                }
            }
        };

        let mut scripts = vec![];
        {
            let mut query = world_cell.world_mut().query::<(&Handle<Script>)>();
            let programs = world_cell
                .world()
                .get_non_send_resource::<ProgramCache>()
                .unwrap();
            for x in query.iter(world_cell.world()) {
                let id = AssetId::from(x);
                let Some(script) = programs.get(&id) else {
                    continue;
                };
                scripts.push(script.clone());
            }
        }
        world_cell
            .world_mut()
            .get_non_send_resource_mut::<LispEngine>()
            .unwrap()
            .borrow_mut()
            .with_mut_reference::<WorldHolder, WorldHolder>(&mut WorldHolder(world_cell))
            .consume(move |engine, args| {
                let world = args[0].clone();
                engine
                    .update_value("*world*", world)
                    .expect("TODO: panic message");
                engine
                    .update_value("*time*", SteelVal::from(curr_time))
                    .expect("TODO: panic message");
                engine.register_fn("-op", op).register_fn("-param", param);

                if let Some(line) = &line {
                    let res = engine.compile_and_run_raw_program(line.clone());
                    match res {
                        Ok(r) => r.into_iter().for_each(|x| match x {
                            SteelVal::Void => {}
                            SteelVal::StringV(s) => {
                                println!("{} {:?}", "=>".bright_blue().bold(), s);
                            }
                            _ => {
                                print!("{} ", "=>".bright_blue().bold());
                                engine
                                    .call_function_by_name_with_args("displayln", vec![x])
                                    .unwrap();
                            }
                        }),
                        Err(e) => {
                            error!("Error: {:?}", e);
                            engine.raise_error(e);
                        }
                    }
                }

                for program in scripts.drain(..) {
                    let res = engine.run_raw_program(program);
                    if let Err(e) = res {
                        error!("Error: {:?}", e);
                    }
                }
            });
    }
}
fn op_bang(world: &mut WorldHolder, ty: String, name: String) -> Option<EntityRef> {
    let world = unsafe { world.world_mut() };

    // if the entity already exists, just touch it
    let index = world.get_resource::<UniqueIndex<OpName>>().unwrap();
    if let Some(entity) = index.get(&OpName(name.clone())) {
        let entity_ref = EntityRef(entity.clone());
        if let Some(mut entity) = world.get_entity_mut(*entity) {
            entity.insert(ScriptTouched);
        }
        return Some(entity_ref);
    }

    let name = OpName(name);
    let entity = match ty.as_str() {
        "ramp" => world.spawn((name, OpType::<TextureOpRamp>::default())),
        "composite" => world.spawn((name, OpType::<TextureOpComposite>::default())),
        "noise" => world.spawn((name, OpType::<TextureOpNoise>::default())),
        "window" => world.spawn((name, OpType::<ComponentOpWindow>::default())),
        "cuboid" => world.spawn((name, OpType::<MeshOpCuboid>::default())),
        "standard-material" => world.spawn((name, OpType::<MaterialOpStandard>::default())),
        _ => return None,
    };

    Some(EntityRef(entity.id()))
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

fn param_bang(world: &mut WorldHolder, entity: EntityRef, name: String, val: SteelVal) {
    let world = unsafe { world.world_mut() };

    let index = world
        .get_resource::<CompositeIndex2<OpRef, ParamName>>()
        .unwrap();
    let name = ParamName(name);
    if let Some(entity) = index.get(&(OpRef(*entity), name.clone())) {
        let entity = *entity;
        world.entity_mut(entity.clone())
            .insert(ScriptTouched)
            .insert(ScriptedParam);

        let mut param = world.get_mut::<ParamValue>(entity.clone()).unwrap();

        if let Err(e) = update_param(&mut param, val) {
            world
                .entity_mut(entity.clone())
                .insert(ScriptedParamError(e.to_string()));
        }
    } else {
    }
}

fn param(world: &mut WorldHolder, entity: EntityRef, name: String) -> SteelVal {
    let world = unsafe { world.world() };
    let index = world
        .get_resource::<CompositeIndex2<OpRef, ParamName>>()
        .unwrap();
    let name = index
        .get(&(OpRef(entity.0), ParamName(name)))
        .map_or(SteelVal::Void, |entity| {
            let value = world.get::<ParamValue>(*entity).unwrap();
            SteelVal::from(value.clone())
        });
    name
}

fn connect_bang(world: &mut WorldHolder, output: EntityRef, input: EntityRef) -> SteelVal {
    let mut world = unsafe { world.world_mut() };

    // TODO: run this as an event
    // let mut graph_state = world.get_resource_mut::<GraphState>().unwrap();

    // world.entity_mut(*output).insert(ConnectedTo(*input));
    // graph_state
    //     .graph
    //     .add_edge(to_graph_id.0, from_graph_id.0, (0, 0));
    // ev_connect.send(Connect {
    //     output: to_graph_ref.0,
    //     input: from_graph_ref.0,
    // });
    //
    SteelVal::Void
}

fn rand(min: f32, max: f32) -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..max)
}

#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
pub enum ScriptError {
    #[error("Could not convert value: {0}")]
    Conversion(SteelVal),
}

fn update_param(param_value: &mut ParamValue, steel_val: SteelVal) -> Result<(), ScriptError> {
    match param_value {
        ParamValue::None => {}
        ParamValue::F32(p) => match steel_val {
            SteelVal::NumV(n) => *p = n as f32,
            SteelVal::IntV(n) => *p = n as f32,
            _ => return Err(ScriptError::Conversion(steel_val)),
        },
        ParamValue::U32(p) => match steel_val {
            SteelVal::NumV(n) => *p = n as u32,
            SteelVal::IntV(n) => *p = n as u32,
            _ => return Err(ScriptError::Conversion(steel_val)),
        },
        ParamValue::Vec2(p) => match steel_val {
            SteelVal::ListV(ref v) => {
                let mut iter = v.into_iter();
                let x = iter.next().unwrap();
                let y = iter.next().unwrap();
                match (x, y) {
                    (SteelVal::NumV(x), SteelVal::NumV(y)) => {
                        p.x = *x as f32;
                        p.y = *y as f32;
                    }
                    (SteelVal::IntV(x), SteelVal::IntV(y)) => {
                        p.x = *x as f32;
                        p.y = *y as f32;
                    }
                    _ => return Err(ScriptError::Conversion(steel_val)),
                }
            }
            _ => return Err(ScriptError::Conversion(steel_val)),
        },
        ParamValue::Color(p) => match steel_val {
            SteelVal::ListV(ref v) => {
                let mut iter = v.into_iter();
                let r = iter.next().unwrap();
                let g = iter.next().unwrap();
                let b = iter.next().unwrap();
                let a = iter.next().unwrap();
                match (r, g, b, a) {
                    (
                        SteelVal::NumV(r),
                        SteelVal::NumV(g),
                        SteelVal::NumV(b),
                        SteelVal::NumV(a),
                    ) => {
                        p.x = *r as f32;
                        p.y = *g as f32;
                        p.z = *b as f32;
                        p.w = *a as f32;
                    }
                    (
                        SteelVal::IntV(r),
                        SteelVal::IntV(g),
                        SteelVal::IntV(b),
                        SteelVal::IntV(a),
                    ) => {
                        p.x = *r as f32;
                        p.y = *g as f32;
                        p.z = *b as f32;
                        p.w = *a as f32;
                    }
                    _ => return Err(ScriptError::Conversion(steel_val)),
                }
            }
            SteelVal::VectorV(ref v) => {
                let mut iter = v.iter();
                let r = iter.next().unwrap();
                let g = iter.next().unwrap();
                let b = iter.next().unwrap();
                let a = iter.next().unwrap();
                match (r, g, b, a) {
                    (
                        SteelVal::NumV(r),
                        SteelVal::NumV(g),
                        SteelVal::NumV(b),
                        SteelVal::NumV(a),
                    ) => {
                        p.x = *r as f32;
                        p.y = *g as f32;
                        p.z = *b as f32;
                        p.w = *a as f32;
                    }
                    (
                        SteelVal::IntV(r),
                        SteelVal::IntV(g),
                        SteelVal::IntV(b),
                        SteelVal::IntV(a),
                    ) => {
                        p.x = *r as f32;
                        p.y = *g as f32;
                        p.z = *b as f32;
                        p.w = *a as f32;
                    }
                    _ => return Err(ScriptError::Conversion(steel_val)),
                }
            }
            _ => return Err(ScriptError::Conversion(steel_val)),
        },
        ParamValue::Bool(p) => match steel_val {
            SteelVal::BoolV(b) => *p = b,
            _ => return Err(ScriptError::Conversion(steel_val)),
        },
        ParamValue::TextureOp(p) => match steel_val {
            SteelVal::Custom(mut c) => {
                let custom = c.borrow_mut();
                let entity = custom.as_any_ref().downcast_ref::<EntityRef>().unwrap();
                *p = Some(entity.0.clone());
            }
            _ => return Err(ScriptError::Conversion(steel_val)),
        },
        ParamValue::MeshOp(p) => match steel_val {
            SteelVal::Custom(mut c) => {
                let custom = c.borrow_mut();
                let entity = custom.as_any_ref().downcast_ref::<EntityRef>().unwrap();
                *p = Some(entity.0.clone());
            }
            _ => return Err(ScriptError::Conversion(steel_val)),
        },
    }

    Ok(())
}

impl From<ParamValue> for SteelVal {
    fn from(value: ParamValue) -> Self {
        match value {
            ParamValue::None => SteelVal::Void,
            ParamValue::F32(x) => SteelVal::from(x),
            ParamValue::U32(x) => SteelVal::from(x),
            ParamValue::Color(x) => {
                let (r, g, b, a) = x.into();
                vec![r, g, b, a].into_steelval().unwrap()
            }
            ParamValue::Vec2(v) => {
                let (x, y) = v.into();
                vec![x, y].into_steelval().unwrap()
            }
            ParamValue::Bool(x) => SteelVal::from(x),
            ParamValue::TextureOp(x) => match x {
                None => SteelVal::Void,
                Some(x) => EntityRef(x).into_steelval().unwrap(),
            },
            ParamValue::MeshOp(x) => match x {
                None => SteelVal::Void,
                Some(x) => EntityRef(x).into_steelval().unwrap(),
            },
        }
    }
}
