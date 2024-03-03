pub mod repl;

use crate::index::{CompositeIndex2, UniqueIndex};
use crate::param::{ParamName, ParamValue, ScriptedParamError, ScriptedParamValue};
use crate::ui::graph::OpRef;
use crate::OpName;
use crate::Sets::Params;
use bevy::asset::AssetContainer;
use bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell;
use bevy::prelude::*;
use boa_engine::object::builtins::JsArray;
use boa_engine::object::Object;
use boa_engine::{
    class::{Class, ClassBuilder},
    error::JsNativeError,
    js_string,
    native_function::NativeFunction,
    property::Attribute,
    Context, JsData, JsObject, JsResult, JsValue, Source,
};
use boa_gc::{Finalize, Trace};
use boa_runtime::Console;
use crate::script::repl::ScriptReplPlugin;

pub struct ScriptPlugin;

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(ScriptReplPlugin)
            .add_systems(Startup, setup)
            .add_systems(Update, (update.in_set(Params)));
    }
}

#[derive(Default, Deref, DerefMut)]
struct JsContext(Context);
#[derive(Debug, JsData, PartialEq, Clone)]
pub struct EntityRef(Entity);

trait WorldScope {
    fn with_world_scope<F, R>(&mut self, world: UnsafeWorldCell<'_>, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R;
}

impl WorldScope for JsContext {
    fn with_world_scope<F, R>(&mut self, world: UnsafeWorldCell<'_>, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let curr_time = unsafe { world.world() }.resource::<Time>().elapsed_seconds();
        let holder = WorldHolder(unsafe { world.world_mut() });
        self.realm().host_defined_mut().insert(holder);
        let result = f(self);
        self.realm().host_defined_mut().remove::<WorldHolder>();
        result
    }
}

#[derive(Debug, Trace, Finalize, JsData)]
struct WorldHolder(#[unsafe_ignore_trace] *mut World);

impl WorldHolder {
    fn world(&self) -> &World {
        unsafe { &*self.0 }
    }

    fn world_mut(&mut self) -> &mut World {
        unsafe { &mut *self.0 }
    }
}

fn setup(world: &mut World) {
    let mut ctx = Context::default();
    add_runtime(&mut ctx);
    world.insert_non_send_resource(JsContext(ctx));
    let world_cell = world.as_unsafe_world_cell();
    unsafe {
        world_cell
            .world_mut()
            .get_non_send_resource_mut::<JsContext>()
            .unwrap()
            .with_world_scope(world_cell, |ctx| {
                ctx
                    .register_global_property(
                        js_string!("time"),
                        0.0,
                        Attribute::all(),
                    )
                    .expect("property shouldn't exist");
            });
    }
}

fn update(world: &mut World) {
    unsafe {
        let mut world_cell = world.as_unsafe_world_cell();
        let mut query = world_cell.world_mut().query::<(
            Entity,
            &mut ParamValue,
            &ScriptedParamValue,
            Option<&ScriptedParamError>,
        )>();
        let params = query.iter_mut(world_cell.world_mut()).collect::<Vec<_>>();
        for (param, mut param_value, script, script_error) in params {
            let script = script.0.clone();
            world_cell
                .world_mut()
                .get_non_send_resource_mut::<JsContext>()
                .unwrap()
                .with_world_scope(world_cell, |ctx| {
                    let elapsed_seconds = world_cell.world().resource::<Time>().elapsed_seconds();
                    let time = ctx.global_object().set(js_string!("time"), elapsed_seconds, true, ctx).unwrap();

                    let res = ctx.eval(Source::from_bytes(&script)).unwrap_or_else(|e| {
                        warn!("Error in script eval: {:?}", e);
                        world_cell
                            .world_mut()
                            .entity_mut(param)
                            .insert(ScriptedParamError(e.to_string()));
                        JsValue::Null
                    });

                    if JsValue::Null != res {
                        // clear the error if there is one
                        if let Some(err) = script_error {
                            world_cell
                                .world_mut()
                                .entity_mut(param)
                                .remove::<ScriptedParamError>();
                        }
                        update_param(&mut param_value, res, ctx);
                    }
                });
        }
    }
}

trait IntoJsValue {
    fn into_js_value(self, context: &mut Context) -> JsValue;
}

impl IntoJsValue for ParamValue {
    fn into_js_value(self, context: &mut Context) -> JsValue {
        match self {
            ParamValue::None => JsValue::Null,
            ParamValue::F32(x) => JsValue::from(x),
            ParamValue::U32(x) => JsValue::from(x),
            ParamValue::Color(x) => {
                let array = JsArray::from_iter(
                    [
                        JsValue::from(x.x),
                        JsValue::from(x.y),
                        JsValue::from(x.z),
                        JsValue::from(x.w),
                    ],
                    context,
                );
                JsValue::Object(array.into())
            }
            ParamValue::Vec2(x) => {
                let array = JsArray::from_iter([JsValue::from(x.x), JsValue::from(x.y)], context);
                JsValue::Object(array.into())
            }
            _ => unimplemented!(),
        }
    }
}

fn update_param(param: &mut ParamValue, js_value: JsValue, context: &mut Context) {
    match param {
        ParamValue::None => {}
        ParamValue::F32(x) => {
            *x = js_value.to_number(context).unwrap() as f32;
        }
        ParamValue::U32(x) => {
            *x = js_value.to_number(context).unwrap() as u32;
        }
        ParamValue::Color(x) => {
            if let Some(array) = js_value.as_object() {
                let r = array.get(0, context).unwrap().to_number(context).unwrap() as f32;
                let g = array.get(1, context).unwrap().to_number(context).unwrap() as f32;
                let b = array.get(2, context).unwrap().to_number(context).unwrap() as f32;
                let a = array.get(3, context).unwrap().to_number(context).unwrap() as f32;
                x.as_mut().copy_from_slice(&[r, g, b, a])
            } else {
                warn!("Expected an array for color");
            }
        }
        ParamValue::Vec2(v) => {
            if let Some(array) = js_value.as_object() {
                let x = array.get(0, context).unwrap().to_number(context).unwrap() as f32;
                let y = array.get(1, context).unwrap().to_number(context).unwrap() as f32;
                v.as_mut().copy_from_slice(&[x, y])
            } else {
                warn!("Expected an array for vec2");
            }
        }
    }
}

fn add_runtime(context: &mut Context) {
    let console = Console::init(context);
    context
        .register_global_property(js_string!(Console::NAME), console, Attribute::all())
        .expect("the console builtin shouldn't exist");
}
