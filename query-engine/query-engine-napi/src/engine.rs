use crate::error::ApiError;
use datamodel::{Configuration, Datamodel};
use prisma_models::DatamodelConverter;
use query_core::{schema_builder, BuildMode, QueryExecutor, QuerySchema};
use request_handlers::{GraphQlBody, GraphQlHandler, PrismaResponse};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct QueryEngine {
    inner: Arc<RwLock<Inner>>,
}

pub enum Inner {
    Builder(EngineBuilder),
    Connected(ConnectedEngine),
}

pub struct EngineBuilder {
    datamodel: Datamodel,
    config: Configuration,
}

pub struct ConnectedEngine {
    query_schema: Arc<QuerySchema>,
    executor: crate::Executor,
}

impl ConnectedEngine {
    pub fn query_schema(&self) -> &Arc<QuerySchema> {
        &self.query_schema
    }

    pub fn executor(&self) -> &(dyn QueryExecutor + Send + Sync) {
        &*self.executor
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectParams {
    enable_raw_queries: bool,
}

impl QueryEngine {
    pub fn new(datamodel_str: &str) -> crate::Result<Self> {
        let config = datamodel::parse_configuration(datamodel_str)
            .map_err(|errors| ApiError::conversion(errors, datamodel_str))?
            .subject
            .validate_that_one_datasource_is_provided()
            .map_err(|errors| ApiError::conversion(errors, datamodel_str))?;

        let datamodel = datamodel::parse_datamodel(datamodel_str)
            .map_err(|errors| ApiError::conversion(errors, datamodel_str))?
            .subject;

        let flags: Vec<_> = config.preview_features().map(|s| s.to_string()).collect();

        if let Err(_) = feature_flags::initialize(&flags) {
            panic!("How feature flags are currently implemented, you must start a new node process to re-initialize a new Query Engine. Sorry Tim!");
        };

        let builder = EngineBuilder { config, datamodel };

        Ok(Self {
            inner: Arc::new(RwLock::new(Inner::Builder(builder))),
        })
    }

    pub async fn connect(&self, params: ConnectParams) -> crate::Result<()> {
        let mut inner = self.inner.write().await;

        match *inner {
            Inner::Builder(ref builder) => {
                let template = DatamodelConverter::convert(&builder.datamodel);

                // We only support one data source at the moment, so take the first one (default not exposed yet).
                let data_source = builder
                    .config
                    .datasources
                    .first()
                    .ok_or_else(|| ApiError::configuration("No valid data source found"))?;

                let (db_name, executor) = crate::exec_loader::load(&data_source).await?;
                let connector = executor.primary_connector();
                connector.get_connection().await?;

                // Build internal data model
                let internal_data_model = template.build(db_name);

                let query_schema = schema_builder::build(
                    internal_data_model,
                    BuildMode::Modern,
                    params.enable_raw_queries,
                    data_source.capabilities(),
                );

                let engine = ConnectedEngine {
                    query_schema: Arc::new(query_schema),
                    executor,
                };

                *inner = Inner::Connected(engine);

                Ok(())
            }
            Inner::Connected(_) => Err(ApiError::AlreadyConnected),
        }
    }

    pub async fn query(&self, query: GraphQlBody) -> crate::Result<PrismaResponse> {
        match *self.inner.read().await {
            Inner::Connected(ref engine) => {
                let handler = GraphQlHandler::new(engine.executor(), engine.query_schema());

                Ok(handler.handle(query).await)
            }
            Inner::Builder(_) => Err(ApiError::NotConnected),
        }
    }
}
