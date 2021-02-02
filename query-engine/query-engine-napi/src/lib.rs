use engine::{ConnectParams, QueryEngine};
use napi::{CallContext, Env, JsObject, JsString, JsUndefined, JsUnknown, Property};
use napi_derive::{js_function, module_exports};
use query_core::QueryExecutor;

mod engine;
mod error;
mod exec_loader;

pub(crate) type Result<T> = std::result::Result<T, error::ApiError>;
pub(crate) type Executor = Box<dyn QueryExecutor + Send + Sync>;

#[js_function(1)]
fn constructor(ctx: CallContext) -> napi::Result<JsUndefined> {
    let url = ctx.get::<JsString>(0)?.into_utf8()?;

    let mut this: JsObject = ctx.this_unchecked();
    let engine = QueryEngine::new(url.as_str()?)?;

    ctx.env.wrap(&mut this, engine)?;
    ctx.env.get_undefined()
}

#[js_function(1)]
fn connect(ctx: CallContext) -> napi::Result<JsObject> {
    let this: JsObject = ctx.this_unchecked();
    let engine: &QueryEngine = ctx.env.unwrap(&this)?;
    let engine: QueryEngine = engine.clone();

    let arg0 = ctx.get::<JsUnknown>(0)?;
    let params: ConnectParams = ctx.env.from_js_value(arg0)?;

    ctx.env
        .execute_tokio_future(async move { Ok(engine.connect(params).await?) }, |&mut env, ()| {
            env.get_undefined()
        })
}

#[js_function(1)]
fn query(ctx: CallContext) -> napi::Result<JsObject> {
    let this: JsObject = ctx.this_unchecked();
    let engine: &QueryEngine = ctx.env.unwrap(&this)?;
    let engine: QueryEngine = engine.clone();

    let query = ctx.get::<JsObject>(0)?;
    let body = ctx.env.from_js_value(query)?;

    ctx.env
        .execute_tokio_future(async move { Ok(engine.query(body).await?) }, |&mut env, response| {
            env.to_js_value(&response)
        })
}

#[js_function(0)]
fn sdl_schema(ctx: CallContext) -> napi::Result<JsObject> {
    let this: JsObject = ctx.this_unchecked();
    let engine: &QueryEngine = ctx.env.unwrap(&this)?;
    let engine: QueryEngine = engine.clone();

    ctx.env
        .execute_tokio_future(async move { Ok(engine.sdl_schema().await?) }, |&mut env, schema| {
            env.create_string(&schema)
        })
}

#[js_function(0)]
fn dmmf(ctx: CallContext) -> napi::Result<JsObject> {
    let this: JsObject = ctx.this_unchecked();
    let engine: &QueryEngine = ctx.env.unwrap(&this)?;
    let engine: QueryEngine = engine.clone();

    ctx.env
        .execute_tokio_future(async move { Ok(engine.dmmf().await?) }, |&mut env, dmmf| {
            env.to_js_value(&dmmf)
        })
}

#[js_function(0)]
fn server_info(ctx: CallContext) -> napi::Result<JsObject> {
    let this: JsObject = ctx.this_unchecked();
    let engine: &QueryEngine = ctx.env.unwrap(&this)?;
    let engine: QueryEngine = engine.clone();

    ctx.env.execute_tokio_future(
        async move { Ok(engine.server_info().await?) },
        |&mut env, server_info| env.to_js_value(&server_info),
    )
}

#[module_exports]
pub fn init(mut exports: JsObject, env: Env) -> napi::Result<()> {
    let query_engine = env.define_class(
        "QueryEngine",
        constructor,
        &[
            Property::new(&env, "connect")?.with_method(connect),
            Property::new(&env, "query")?.with_method(query),
            Property::new(&env, "sdlSchema")?.with_method(sdl_schema),
            Property::new(&env, "dmmf")?.with_method(dmmf),
            Property::new(&env, "serverInfo")?.with_method(server_info),
        ],
    )?;

    exports.set_named_property("QueryEngine", query_engine)?;

    Ok(())
}
