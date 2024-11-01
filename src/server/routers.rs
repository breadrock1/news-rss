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
use crate::server::forms::TerminateWorkerForm;
use crate::server::swagger::SwaggerExamples;
use crate::server::{RssWorker, ServerApp};

use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use std::sync::Arc;

use super::forms::DeleteWorkerForm;

#[utoipa::path(
    get,
    path = "/workers/all",
    tag = "workers",
    responses(
        (
            status = 200,
            description = "Successful",
            body = Vec<GetInfoResponse>,
            example = json!(vec![GetInfoResponse::example(None)]),
        ),
        (
            status = 400,
            description = "Failed to get all workers info",
            body = ServerError,
            example = json!(ServerError::example(Some("failed to get all workers info".to_string()))),
        ),
        (
            status = 503,
            description = "Server does not available",
            body = ServerError,
            example = json!(ServerError::example(None)),
        ),
    )
)]
pub async fn get_workers<P, C, S>(
    State(state): State<Arc<ServerApp<P, C, S>>>,
) -> ServerResult<impl IntoResponse>
where
    P: Publisher + Sync + Send,
    C: CacheService + Sync + Send,
    S: CrawlerService + Sync + Send,
{
    let workers = state.workers();
    let workers_guard = workers.read().await;

    let response = workers_guard
        .values()
        .map(|worker| {
            let is_launched = !worker.worker().is_finished();
            let response = GetInfoResponse::builder()
                .source_name(worker.source_name().to_owned())
                .source_url(worker.source_url().to_owned())
                .is_launched(is_launched)
                .build()
                .unwrap();

            response
        })
        .collect::<Vec<GetInfoResponse>>();

    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/workers/info",
    tag = "workers",
    request_body(
        content = GetInfoForm,
        example = json!(GetInfoForm::example(None)),
    ),
    responses(
        (
            status = 200,
            description = "Successful",
            body = GetInfoResponse,
            example = json!(GetInfoResponse::example(None)),
        ),
        (
            status = 400,
            description = "Failed to get worker info",
            body = ServerError,
            example = json!(ServerError::example(Some("failed to get worker info".to_string()))),
        ),
        (
            status = 503,
            description = "Server does not available",
            body = ServerError,
            example = json!(ServerError::example(None)),
        ),
    )
)]
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

    let is_launched = !worker.worker().is_finished();
    let response = GetInfoResponse::builder()
        .source_name(form.source_name().to_owned())
        .source_url(form.source_url().to_owned())
        .is_launched(is_launched)
        .build()
        .unwrap();

    Ok(Json(response))
}

#[utoipa::path(
    put,
    path = "/workers/create",
    tag = "workers",
    request_body(
        content = CreateWorkerForm,
        example = json!(CreateWorkerForm::example(None)),
    ),
    responses(
        (
            status = 200,
            description = "Successful",
            body = Success,
            example = json!(Success::example(None)),
        ),
        (
            status = 400,
            description = "Failed to create worker",
            body = ServerError,
            example = json!(ServerError::example(Some("failed to create worker".to_string()))),
        ),
        (
            status = 503,
            description = "Server does not available",
            body = ServerError,
            example = json!(ServerError::example(None)),
        ),
    )
)]
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
        if !worker.worker().is_finished() && !form.create_force() {
            let msg = format!("worker {worker_name} already launched");
            return Err(ServerError::Launched(msg));
        }
    }

    let config = RssConfig::from(&form);
    let cache = state.cache.clone();
    let crawler = state.crawler.clone();
    let publish = state.publish.clone();
    let feeds = RssFeeds::new(config, publish, cache, crawler).unwrap();

    let task = tokio::spawn(async move { feeds.launch_fetching().await });

    let source = form.source_name().to_owned();
    let target = form.target_url().to_owned();
    let rss_worker = RssWorker::new(source, target, task);
    workers_guard.insert(worker_name.to_owned(), rss_worker);

    Ok(Json(Success::default()))
}

#[utoipa::path(
    post,
    path = "/workers/restart",
    tag = "workers",
    request_body(
        content = CreateWorkerForm,
        example = json!(CreateWorkerForm::example(None)),
    ),
    responses(
        (
            status = 200,
            description = "Successful",
            body = Success,
            example = json!(Success::example(None)),
        ),
        (
            status = 400,
            description = "Failed to restart worker",
            body = ServerError,
            example = json!(ServerError::example(Some("failed to restart worker".to_string()))),
        ),
        (
            status = 503,
            description = "Server does not available",
            body = ServerError,
            example = json!(ServerError::example(None)),
        ),
    )
)]
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

    worker.worker().abort();

    let config = RssConfig::from(&form);
    let cache = state.cache.clone();
    let crawler = state.crawler.clone();
    let publish = state.publish.clone();
    let feeds = RssFeeds::new(config, publish, cache, crawler).unwrap();

    let task = tokio::spawn(async move { feeds.launch_fetching().await });

    let source = form.source_name().to_owned();
    let target = form.target_url().to_owned();
    let rss_worker = RssWorker::new(source, target, task);
    workers_guard.insert(worker_name.to_owned(), rss_worker);

    Ok(Json(Success::default()))
}

#[utoipa::path(
    post,
    path = "/workers/terminate",
    tag = "workers",
    request_body(
        content = TerminateWorkerForm,
        example = json!(TerminateWorkerForm::example(None)),
    ),
    responses(
        (
            status = 200,
            description = "Successful",
            body = Success,
            example = json!(Success::example(None)),
        ),
        (
            status = 400,
            description = "Failed to terminate worker",
            body = ServerError,
            example = json!(ServerError::example(Some("failed to terminate worker".to_string()))),
        ),
        (
            status = 503,
            description = "Server does not available",
            body = ServerError,
            example = json!(ServerError::example(None)),
        ),
    )
)]
pub async fn terminate_worker<P, C, S>(
    State(state): State<Arc<ServerApp<P, C, S>>>,
    Json(form): Json<TerminateWorkerForm>,
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

    worker.worker().abort();
    Ok(Json(Success::default()))
}

#[utoipa::path(
    delete,
    path = "/workers/delete",
    tag = "workers",
    request_body(
        content = DeleteWorkerForm,
        example = json!(DeleteWorkerForm::example(None)),
    ),
    responses(
        (
            status = 200,
            description = "Successful",
            body = Success,
            example = json!(Success::example(None)),
        ),
        (
            status = 400,
            description = "Failed to delete worker",
            body = ServerError,
            example = json!(ServerError::example(Some("failed to delete worker".to_string()))),
        ),
        (
            status = 503,
            description = "Server does not available",
            body = ServerError,
            example = json!(ServerError::example(None)),
        ),
    )
)]
pub async fn delete_worker<P, C, S>(
    State(state): State<Arc<ServerApp<P, C, S>>>,
    Json(form): Json<DeleteWorkerForm>,
) -> ServerResult<impl IntoResponse>
where
    P: Publisher + Sync + Send,
    C: CacheService + Sync + Send,
    S: CrawlerService + Sync + Send,
{
    let workers = state.workers();
    let mut workers_guard = workers.write().await;

    let worker_name = form.target_url();
    let Some(worker) = workers_guard.get(worker_name) else {
        let msg = format!("worker {worker_name} does not exist");
        tracing::warn!("{}", &msg);
        return Err(ServerError::NotFound(msg));
    };

    if worker.worker().is_finished() {
        let _ = workers_guard.remove(worker_name);
        return Ok(Json(Success::default()));
    }

    if form.is_force() {
        worker.worker().abort();
        let _ = workers_guard.remove(worker_name);
        return Ok(Json(Success::default()));
    }

    let msg = format!("worker {worker_name} is launched. Use force flag to terminate");
    tracing::warn!("{}", &msg);
    Err(ServerError::Launched(msg))
}
