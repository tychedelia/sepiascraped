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

pub struct ScriptPlugin;

impl Plugin for ScriptPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup)
            .add_systems(Update, (update, print_counts));
    }
}

#[derive(Default, Deref, DerefMut)]
struct JsContext(Context);

#[derive(Component)]
pub struct Counter {
    count: u32,
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

trait WorldScope {
    fn with_world_scope<'w, F, R>(&mut self, world: UnsafeWorldCell<'w>, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R;
}

impl WorldScope for Context {
    fn with_world_scope<'w, F, R>(&mut self, world: UnsafeWorldCell<'w>, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        let holder = WorldHolder(unsafe { world.world_mut() });
        self.realm().host_defined_mut().insert(holder);
        let result = f(self);
        self.realm().host_defined_mut().remove::<WorldHolder>();
        result
    }
}

fn startup(world: &mut World) {
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
                ctx.eval(Source::from_bytes(
                    r"
                let counter = new Counter();
    ",
                ))
                .unwrap();
            });
    }
}

fn update(world: &mut World) {
let world_cell = world.as_unsafe_world_cell();
    unsafe {
        world_cell
            .world_mut()
            .get_non_send_resource_mut::<JsContext>()
            .unwrap()
            .with_world_scope(world_cell, |ctx| {
                ctx.eval(Source::from_bytes(
                    r"
                console.log('Setting count from JS');
		        counter.set_count(Math.floor(Math.random() * 100));
    ",
                ))
                    .unwrap();
            });
    }
}

fn print_counts(count_q: Query<&Counter>) {
    for count in count_q.iter() {
        println!("Count: {}", count.count);
    }
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
            if let Some(Counter) = object.downcast_ref::<CounterClass>() {
                unsafe {
                    let realm = ctx.realm().clone();
                    let mut host_defined = realm.host_defined_mut();
                    let world_holder = host_defined.get_mut::<WorldHolder>().unwrap();
                    let world = world_holder.world_mut();
                    let mut entity = world.get_entity_mut(Counter.entity).unwrap();
                    let mut counter = entity.get_mut::<Counter>().unwrap();
                    counter.count = args.first().unwrap().to_u32(ctx).unwrap();
                }
            }

            return Ok(JsValue::undefined());
        }
        Err(JsNativeError::typ()
            .with_message("'this' is not a Counter object")
            .into())
    }
}

impl Class for CounterClass {
    const NAME: &'static str = "Counter";
    const LENGTH: usize = 0;

    fn data_constructor(_this: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<Self> {
        let realm = ctx.realm().clone();
        let mut host_defined = realm.host_defined_mut();
        let world_holder = host_defined.get_mut::<WorldHolder>().unwrap();
        let world = world_holder.world_mut();
        let entity = world.spawn(Counter { count: 0 });
        let entity = entity.id();
        let Counter = CounterClass { entity, count: 0 };
        Ok(Counter)
    }

    fn init(class: &mut ClassBuilder<'_>) -> JsResult<()> {
        class.method(
            js_string!("set_count"),
            0,
            NativeFunction::from_fn_ptr(Self::set_count),
        );

        Ok(())
    }
}

fn add_runtime(context: &mut Context) {
    let console = Console::init(context);
    context
        .register_global_property(js_string!(Console::NAME), console, Attribute::all())
        .expect("the console builtin shouldn't exist");

    context
        .register_global_class::<CounterClass>()
        .expect("the Counter builtin shouldn't exist");
}
