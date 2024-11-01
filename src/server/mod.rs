pub mod config;
mod errors;
mod forms;
mod routers;
mod swagger;

use crate::cache::CacheService;
use crate::crawler::CrawlerService;
use crate::publish::Publisher;

use axum::routing::{delete, get, post, put};
use axum::Router;
use getset::Getters;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

type JoinableWorkers = HashMap<String, RssWorker>;

#[derive(Getters)]
#[getset(get = "pub")]
pub struct RssWorker {
    source_name: String,
    source_url: String,
    worker: JoinHandle<Result<(), anyhow::Error>>,
}

impl RssWorker {
    pub fn new(name: String, url: String, worker: JoinHandle<Result<(), anyhow::Error>>) -> Self {
        RssWorker {
            source_name: name,
            source_url: url,
            worker,
        }
    }
}

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
        .route("/workers/all", get(routers::get_workers))
        .route("/workers/info", post(routers::get_worker_info))
        .route("/workers/create", put(routers::create_worker))
        .route("/workers/restart", post(routers::restart_worker))
        .route("/workers/delete", delete(routers::delete_worker))
        .route("/workers/terminate", post(routers::terminate_worker))
        .with_state(app_arc)
}
