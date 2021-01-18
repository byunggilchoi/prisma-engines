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
fn engine_constructor(ctx: CallContext) -> napi::Result<JsUndefined> {
    let url = ctx.get::<JsString>(0)?.into_utf8()?;

    let mut this: JsObject = ctx.this_unchecked();
    let engine = QueryEngine::new(url.as_str()?)?;

    ctx.env.wrap(&mut this, engine)?;
    ctx.env.get_undefined()
}

#[js_function(1)]
fn engine_connect(ctx: CallContext) -> napi::Result<JsObject> {
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
fn engine_query(ctx: CallContext) -> napi::Result<JsObject> {
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

#[module_exports]
pub fn init(mut exports: JsObject, env: Env) -> napi::Result<()> {
    let query_engine = env.define_class(
        "QueryEngine",
        engine_constructor,
        &[
            Property::new(&env, "connect")?.with_method(engine_connect),
            Property::new(&env, "query")?.with_method(engine_query),
        ],
    )?;

    exports.set_named_property("QueryEngine", query_engine)?;

    Ok(())
}
