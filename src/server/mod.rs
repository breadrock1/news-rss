pub mod config;
mod errors;
mod forms;
mod routers;
mod swagger;

use crate::cache::CacheService;
use crate::crawler::CrawlerService;
use crate::feeds::rss_feeds::config::RssConfig;
use crate::publish::Publisher;
use crate::storage::pgsql::models::PgsqlTopicModel;
use crate::storage::LoadTopic;

use axum::routing::{delete, get, patch, post, put};
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
    config: Arc<RssConfig>,
    worker: JoinHandle<Result<(), anyhow::Error>>,
}

impl RssWorker {
    pub fn new(config: Arc<RssConfig>, worker: JoinHandle<Result<(), anyhow::Error>>) -> Self {
        RssWorker { config, worker }
    }
}

pub struct ServerApp<P, C, S, R>
where
    P: Publisher,
    C: CacheService,
    S: CrawlerService,
    R: LoadTopic,
{
    workers: Arc<RwLock<JoinableWorkers>>,
    publish: Arc<P>,
    cache: Arc<C>,
    crawler: Arc<S>,
    storage: Arc<R>,
}

impl<P, C, S, R> ServerApp<P, C, S, R>
where
    P: Publisher,
    C: CacheService,
    S: CrawlerService,
    R: LoadTopic,
{
    pub fn new(
        workers: JoinableWorkers,
        publish: Arc<P>,
        cache: Arc<C>,
        crawler: Arc<S>,
        storage: Arc<R>,
    ) -> Self {
        let workers_guard = Arc::new(RwLock::new(workers));
        ServerApp {
            workers: workers_guard,
            publish,
            cache,
            crawler,
            storage,
        }
    }

    pub fn workers(&self) -> Arc<RwLock<JoinableWorkers>> {
        self.workers.clone()
    }

    pub fn storage(&self) -> Arc<R> {
        self.storage.clone()
    }
}

pub fn init_server<P, C, S, R>(app: ServerApp<P, C, S, R>) -> Router
where
    P: Publisher + Sync + Send + 'static,
    C: CacheService + Sync + Send + 'static,
    S: CrawlerService + Sync + Send + 'static,
    R: LoadTopic<TopicId = i32, Topic = PgsqlTopicModel, Error = sqlx::Error>
        + Sync
        + Send
        + 'static,
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
        .route("/sources/all", get(routers::all_sources))
        .route("/sources/add", put(routers::add_source))
        .route("/sources/search", post(routers::search_sources))
        .route("/sources/update", patch(routers::update_source))
        .route("/sources/:source_id", delete(routers::remove_source))
        .with_state(app_arc)
}
