use std::cell::OnceCell;
use std::sync::{LazyLock, Mutex};
use bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell;

use bevy::prelude::*;
use boa_engine::{
    class::{Class, ClassBuilder},
    Context,
    error::JsNativeError,
    js_string,
    JsData,
    JsResult, JsValue, native_function::NativeFunction, property::Attribute, Source,
};
use boa_gc::{Finalize, Trace};
use boa_runtime::Console;

pub static mut WORLD: LazyLock<Option<UnsafeWorldCell>> = LazyLock::new(|| None);

pub static mut RUNTIME: LazyLock<Mutex<Context>> = LazyLock::new(|| {
    let mut ctx = Context::default();
    add_runtime(&mut ctx);
    Mutex::new(ctx)
});

pub struct ScriptPlugin;

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, startup)
            .add_systems(Update, (update, print_counts));
    }
}

fn startup(world: &mut World) {
    unsafe {
        WORLD.replace(world.as_unsafe_world_cell());
    }
    unsafe {
        RUNTIME
            .lock()
            .unwrap()
            .eval(Source::from_bytes(
                r"
                let foo = new Foo();
                console.log(Object.keys(foo));
    ",
            ))
            .unwrap();
    }
}

fn update(world: &mut World) {
    unsafe {
        WORLD.replace(world.as_unsafe_world_cell());
    }
    unsafe {
        RUNTIME
            .lock()
            .unwrap()
            .eval(Source::from_bytes(
                r"
                console.log('Setting count from JS');
		        foo.set_count(Math.floor(Math.random() * 100));
    ",
            ))
            .unwrap();
    }
}

fn print_counts(count_q: Query<&Counter>) {
    for count in count_q.iter() {
        println!("Count: {}", count.count);
    }
}

#[derive(Component)]
pub struct Counter {
    count: u32,
}

#[derive(Debug, Trace, Finalize, JsData)]
struct CounterClass {
    #[unsafe_ignore_trace]
    pub entity: Entity,
    pub count: u32,
}

impl CounterClass {
    fn set_count(this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
        if let Some(object) = this.as_object() {
            if let Some(Foo) = object.downcast_ref::<CounterClass>() {
                unsafe {
                    let world = WORLD.unwrap().world_mut();
                    let mut entity = world.get_entity_mut(Foo.entity).unwrap();
                    let mut counter = entity.get_mut::<Counter>().unwrap();
                    counter.count = args.first().unwrap().to_u32(ctx).unwrap();
                    Foo.count = counter.count;
                }
            }

            return Ok(JsValue::undefined());
        }
        Err(JsNativeError::typ()
            .with_message("'this' is not a Foo object")
            .into())
    }
}

impl Class for CounterClass {
    const NAME: &'static str = "Foo";
    const LENGTH: usize = 0;

    fn data_constructor(
        _this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<Self> {
        let entity = unsafe {
            let world = WORLD.unwrap().world_mut();
            world.spawn(Counter { count: 0 })
        };
        let entity = entity.id();
        let Foo = CounterClass {
            entity,
            count: 0,
        };
        Ok(Foo)
    }

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        class.method(
            js_string!("set_count"),
            0,
            NativeFunction::from_fn_ptr(Self::set_count),
        );

        class.accessor("count", None, Some(NativeFunction::from_fn_ptr(Self::set_count)), Attribute::all());
        Ok(())
    }
}

fn add_runtime(context: &mut Context) {
    let console = Console::init(context);
    context
        .register_global_property(js_string!(Console::NAME), console, Attribute::all())
        .expect("the console builtin shouldn't exist");

    // Then we need to register the global class `Person` inside `context`.
    context
        .register_global_class::<CounterClass>()
        .expect("the Person builtin shouldn't exist");
}
