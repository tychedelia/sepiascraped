mod asset;
mod helper;

use crate::index::{CompositeIndex2, UniqueIndex};
use crate::param::{ParamName, ParamValue, ScriptedParamError, ScriptedParamValue};
use crate::script::asset::{ProgramCache, Script, ScriptAssetPlugin};
use crate::script::helper::RustylineHelper;
use crate::ui::graph::OpRef;
use crate::OpName;
use crate::Sets::Params;
use bevy::app::AppExit;
use bevy::asset::AssetContainer;
use bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell;
use bevy::prelude::*;
use bevy::utils::HashMap;
use rustyline::error::ReadlineError;
use rustyline::highlight::MatchingBracketHighlighter;
use rustyline::validate::MatchingBracketValidator;
use rustyline::{DefaultEditor, Editor};
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, TryRecvError};
use colored::Colorize;
use steel::gc::unsafe_erased_pointers::CustomReference;
use steel::rvals::IntoSteelVal;
use steel::steel_vm::engine::Engine;
use steel::steel_vm::register_fn::RegisterFn;
use steel::SteelVal;
use steel_derive::Steel;

pub struct ScriptPlugin;

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ScriptAssetPlugin)
            .add_systems(Startup, setup)
            .add_systems(Update, (update.in_set(Params)));
    }
}

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

#[derive(Debug, Steel, PartialEq, Clone)]
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
            let programs = world_cell.world().get_non_send_resource::<ProgramCache>().unwrap();
            for x in query.iter(world_cell.world()) {
                let id = AssetId::from(x);
                let Some(script) = programs.get(&id) else {
                    continue;
                };
                scripts.push(script.clone());
            };
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
                                engine.call_function_by_name_with_args("displayln", vec![x])
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
        }
    }
}
