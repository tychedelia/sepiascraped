mod asset;
mod helper;

use crate::index::{CompositeIndex2, UniqueIndex};
use crate::param::{ParamName, ParamValue, ScriptedParamError};
use crate::script::asset::{ProgramCache, Script, ScriptAssetPlugin};
use crate::script::helper::RustylineHelper;
use crate::op::texture::types::composite::TextureOpComposite;
use crate::op::texture::types::noise::TextureOpNoise;
use crate::op::texture::types::ramp::TextureOpRamp;
use crate::op::texture::{TextureOp, TextureOpType};
use crate::ui::graph::{GraphRef, OpRef};
use crate::OpName;
use crate::Sets::Params;
use bevy::app::AppExit;
use bevy::asset::AssetContainer;
use bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell;
use bevy::prelude::*;
use bevy::utils::HashMap;
use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::highlight::MatchingBracketHighlighter;
use rustyline::validate::MatchingBracketValidator;
use rustyline::{DefaultEditor, Editor};
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, TryRecvError};
use rand::Rng;
use steel::gc::unsafe_erased_pointers::CustomReference;
use steel::rvals::{CustomType, IntoSteelVal};
use steel::steel_vm::engine::Engine;
use steel::steel_vm::register_fn::RegisterFn;
use steel::SteelVal;
use steel_derive::Steel;
use crate::op::component::{ComponentOp, ComponentOpType};
use crate::op::component::types::window::ComponentOpWindow;

pub struct ScriptPlugin;

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ScriptAssetPlugin)
            .add_systems(First, clear_touched)
            .add_systems(Last, drop_untouched)
            .add_systems(Startup, setup)
            .add_systems(Update, (update.in_set(Params)));
    }
}

fn clear_touched(mut commands: Commands, touched_q: Query<Entity, With<ScriptTouched>>) {
    for entity in touched_q.iter() {
        commands.entity(entity).remove::<ScriptTouched>();
    }
}

fn drop_untouched(
    mut commands: Commands,
    mut index: ResMut<UniqueIndex<OpName>>,
    touched_q: Query<(Entity, &GraphRef, &OpName), (With<TextureOp>, Without<ScriptTouched>)>,
) {
    for (entity, graph_ref, op_name) in touched_q.iter() {
        commands.entity(entity).despawn_recursive();
        commands.entity(**graph_ref).despawn_recursive();
        // TODO: remove from index, this shouldn't be necessary
        index.remove(op_name);
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
    let editor = ReadLineEditor::default();
    let engine = Rc::new(RefCell::new(engine));
    world.insert_non_send_resource(editor);
    world.insert_non_send_resource(LispEngine(engine));
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
                    .register_fn("-op", op)
                    .register_fn("-op!", op_bang)
                    .register_fn("-param", param)
                    .register_fn("-param!", set_bang)
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
        world.entity_mut(*entity).insert(ScriptTouched);
        return Some(entity_ref);
    }

    let name = OpName(name);
    let entity = match ty.as_str() {
        "ramp" => world.spawn((name, TextureOp, TextureOpType::<TextureOpRamp>::default())),
        "composite" => world.spawn((
            name,
            TextureOp,
            TextureOpType::<TextureOpComposite>::default(),
        )),
        "noise" => world.spawn((name, TextureOp, TextureOpType::<TextureOpNoise>::default())),
        "window" => world.spawn((name, ComponentOp, ComponentOpType::<ComponentOpWindow>::default())),
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

fn set_bang(world: &mut WorldHolder, entity: EntityRef, name: String, val: SteelVal) {
    let world = unsafe { world.world_mut() };
    let index = world.get_resource::<CompositeIndex2<OpRef, ParamName>>().unwrap();
    let name = ParamName(name);
    if let Some(entity) = index.get(&(OpRef(*entity), name.clone())) {
        let mut param = world.get_mut::<ParamValue>(*entity).unwrap();
        update_param(&mut param, val);
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

fn rand(min: f32, max: f32) -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..max)
}

fn update_param(param_value: &mut ParamValue, steel_val: SteelVal) {
    match param_value {
        ParamValue::None => {}
        ParamValue::F32(p) => {
            match steel_val {
                SteelVal::NumV(n) => *p = n as f32,
                SteelVal::IntV(n) => *p = n as f32,
                _ => warn!("Mismatched type"),
            }
        }
        ParamValue::U32(p) => {
            match steel_val {
                SteelVal::NumV(n) => *p = n as u32,
                SteelVal::IntV(n) => *p = n as u32,
                _ => warn!("Mismatched type"),
            }
        }
        ParamValue::Vec2(p) => {
            match steel_val {
                SteelVal::ListV(v) => {
                    let mut iter = v.into_iter();
                    let x = iter.next().unwrap();
                    let y = iter.next().unwrap();
                    match (x, y) {
                        (SteelVal::NumV(x), SteelVal::NumV(y)) => {
                            p.x = x as f32;
                            p.y = y as f32;
                        }
                        (SteelVal::IntV(x), SteelVal::IntV(y)) => {
                            p.x = x as f32;
                            p.y = y as f32;
                        }
                        _ => warn!("Mismatched type"),
                    }
                }
                _ => warn!("Mismatched type"),
            }
        }
        ParamValue::Color(p) => {
            match steel_val {
                SteelVal::ListV(v) => {
                    let mut iter = v.into_iter();
                    let r = iter.next().unwrap();
                    let g = iter.next().unwrap();
                    let b = iter.next().unwrap();
                    let a = iter.next().unwrap();
                    match (r, g, b, a) {
                        (SteelVal::NumV(r), SteelVal::NumV(g), SteelVal::NumV(b), SteelVal::NumV(a)) => {
                            p.x = r as f32;
                            p.y = g as f32;
                            p.z = b as f32;
                            p.w = a as f32;
                        }
                        (SteelVal::IntV(r), SteelVal::IntV(g), SteelVal::IntV(b), SteelVal::IntV(a)) => {
                            p.x = r as f32;
                            p.y = g as f32;
                            p.z = b as f32;
                            p.w = a as f32;
                        }
                        _ => warn!("Mismatched type"),
                    }
                }
                SteelVal::VectorV(v) => {
                    let mut iter = v.iter();
                    let r = iter.next().unwrap();
                    let g = iter.next().unwrap();
                    let b = iter.next().unwrap();
                    let a = iter.next().unwrap();
                    match (r, g, b, a) {
                        (SteelVal::NumV(r), SteelVal::NumV(g), SteelVal::NumV(b), SteelVal::NumV(a)) => {
                            p.x = *r as f32;
                            p.y = *g as f32;
                            p.z = *b as f32;
                            p.w = *a as f32;
                        }
                        (SteelVal::IntV(r), SteelVal::IntV(g), SteelVal::IntV(b), SteelVal::IntV(a)) => {
                            p.x = *r as f32;
                            p.y = *g as f32;
                            p.z = *b as f32;
                            p.w = *a as f32;
                        }
                        _ => warn!("Mismatched type"),
                    }
                }
                _ => warn!("Mismatched type"),
            }
        }
        ParamValue::Bool(p) => {
            match steel_val {
                SteelVal::BoolV(b) => *p = b,
                _ => warn!("Mismatched type"),
            }
        }
        ParamValue::TextureOp(p) => {
            match steel_val {
                SteelVal::Custom(mut c) => {
                    let custom = c.get_mut().unwrap().borrow_mut();
                    let entity = custom.as_any_ref().downcast_ref::<EntityRef>().unwrap();
                    *p = entity.0.clone();
                }
                _ => {}
            }
        }
    }
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
            ParamValue::TextureOp(x) => {
                EntityRef(x).into_steelval().unwrap()
            }
        }
    }
}
