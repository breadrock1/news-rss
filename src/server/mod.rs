pub mod config;
mod errors;
mod forms;
mod routers;
mod swagger;

use crate::cache::CacheService;
use crate::crawler::CrawlerService;
use crate::publish::Publisher;

use axum::routing::post;
use axum::Router;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

type JoinableWorkers = HashMap<String, JoinHandle<Result<(), anyhow::Error>>>;

pub struct ServerApp<P, C, S>
where
    P: Publisher,
    C: CacheService,
    S: CrawlerService,
{
    workers: Arc<RwLock<JoinableWorkers>>,
    publish: Arc<P>,
    cache: Arc<C>,
    crawler: Arc<S>,
}

impl<P, C, S> ServerApp<P, C, S>
where
    P: Publisher,
    C: CacheService,
    S: CrawlerService,
{
    pub fn new(workers: JoinableWorkers, publish: Arc<P>, cache: Arc<C>, crawler: Arc<S>) -> Self {
        let workers_guard = Arc::new(RwLock::new(workers));
        ServerApp {
            workers: workers_guard,
            publish,
            cache,
            crawler,
        }
    }

    pub fn workers(&self) -> Arc<RwLock<JoinableWorkers>> {
        self.workers.clone()
    }
}

pub fn init_server<P, C, S>(app: ServerApp<P, C, S>) -> Router
where
    P: Publisher + Sync + Send + 'static,
    C: CacheService + Sync + Send + 'static,
    S: CrawlerService + Sync + Send + 'static,
{
    let app_arc = Arc::new(app);
    Router::new()
        .merge(swagger::init_swagger())
        .route("/workers/info", post(routers::get_worker_info))
        .route("/workers/create", post(routers::create_worker))
        .route("/workers/restart", post(routers::restart_worker))
        .route("/workers/terminate", post(routers::terminate_worker))
        .with_state(app_arc)
}
