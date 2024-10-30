use crate::cache::CacheService;
use crate::crawler::CrawlerService;
use crate::feeds::rss_feeds::config::RssConfig;
use crate::feeds::rss_feeds::RssFeeds;
use crate::feeds::FetchTopic;
use crate::publish::Publisher;
use crate::server::errors::ServerError;
use crate::server::errors::ServerResult;
use crate::server::errors::Success;
use crate::server::forms::CreateWorkerForm;
use crate::server::forms::GetInfoForm;
use crate::server::forms::GetInfoResponse;
use crate::server::forms::TerminateForm;
use crate::server::ServerApp;

use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use std::sync::Arc;

pub async fn get_worker_info<P, C, S>(
    State(state): State<Arc<ServerApp<P, C, S>>>,
    Json(form): Json<GetInfoForm>,
) -> ServerResult<impl IntoResponse>
where
    P: Publisher + Sync + Send,
    C: CacheService + Sync + Send,
    S: CrawlerService + Sync + Send,
{
    let workers = state.workers();
    let workers_guard = workers.read().await;

    let worker_name = form.source_url();

    let Some(worker) = workers_guard.get(worker_name) else {
        tracing::warn!("there is not worker with name: {worker_name}");
        let msg = format!("worker with name {worker_name}");
        return Err(ServerError::NotFound(msg));
    };

    let is_launched = !worker.is_finished();
    let response = GetInfoResponse::builder()
        .source_name(form.source_name().to_owned())
        .source_url(form.source_url().to_owned())
        .is_launched(is_launched)
        .build()
        .unwrap();

    Ok(Json(response))
}

pub async fn create_worker<P, C, S>(
    State(state): State<Arc<ServerApp<P, C, S>>>,
    Json(form): Json<CreateWorkerForm>,
) -> ServerResult<impl IntoResponse>
where
    P: Publisher + Sync + Send + 'static,
    C: CacheService + Sync + Send + 'static,
    S: CrawlerService + Sync + Send + 'static,
{
    let workers = state.workers();
    let mut workers_guard = workers.write().await;

    let worker_name = form.target_url();
    if let Some(worker) = workers_guard.get(worker_name) {
        tracing::info!("worker {worker_name} already exists");
        if !worker.is_finished() && !form.create_force() {
            let msg = format!("worker {worker_name} already launched");
            return Err(ServerError::AlreadyLaunched(msg));
        }
    }

    let config = RssConfig::from(&form);
    let cache = state.cache.clone();
    let crawler = state.crawler.clone();
    let publish = state.publish.clone();
    let feeds = RssFeeds::new(&config, publish, cache, crawler).unwrap();

    let task = tokio::spawn(async move { feeds.launch_fetching().await });
    workers_guard.insert(worker_name.to_owned(), task);

    Ok(Json(Success::default()))
}

pub async fn restart_worker<P, C, S>(
    State(state): State<Arc<ServerApp<P, C, S>>>,
    Json(form): Json<CreateWorkerForm>,
) -> ServerResult<impl IntoResponse>
where
    P: Publisher + Sync + Send + 'static,
    C: CacheService + Sync + Send + 'static,
    S: CrawlerService + Sync + Send + 'static,
{
    let workers = state.workers();
    let mut workers_guard = workers.write().await;

    let worker_name = form.target_url();
    let Some(worker) = workers_guard.get(worker_name) else {
        tracing::warn!("there is not worker with name: {worker_name}");
        let msg = format!("worker with name {worker_name}");
        return Err(ServerError::NotFound(msg));
    };

    worker.abort();

    let config = RssConfig::from(&form);
    let cache = state.cache.clone();
    let crawler = state.crawler.clone();
    let publish = state.publish.clone();
    let feeds = RssFeeds::new(&config, publish, cache, crawler).unwrap();

    let task = tokio::spawn(async move { feeds.launch_fetching().await });
    workers_guard.insert(worker_name.to_owned(), task);

    Ok(Json(Success::default()))
}

pub async fn terminate_worker<P, C, S>(
    State(state): State<Arc<ServerApp<P, C, S>>>,
    Json(form): Json<TerminateForm>,
) -> ServerResult<impl IntoResponse>
where
    P: Publisher + Sync + Send,
    C: CacheService + Sync + Send,
    S: CrawlerService + Sync + Send,
{
    let workers = state.workers();
    let workers_guard = workers.read().await;

    let worker_name = form.source_url();
    let Some(worker) = workers_guard.get(worker_name) else {
        tracing::warn!("there is not worker with name: {worker_name}");
        let msg = format!("worker with name {worker_name}");
        return Err(ServerError::NotFound(msg));
    };

    worker.abort();
    Ok(Json(Success::default()))
}
