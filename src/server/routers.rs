use crate::cache::CacheService;
use crate::crawler::CrawlerService;
use crate::feeds::rss_feeds::RssFeeds;
use crate::feeds::FetchTopic;
use crate::publish::Publisher;
use crate::server::errors::ServerError;
use crate::server::errors::ServerResult;
use crate::server::errors::Success;
use crate::server::forms::*;
use crate::server::swagger::SwaggerExamples;
use crate::server::{RssWorker, ServerApp};
use crate::storage::pgsql::models::PgsqlTopicModel;
use crate::storage::LoadTopic;

use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use std::sync::Arc;

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
pub async fn get_workers<P, C, S, R>(
    State(state): State<Arc<ServerApp<P, C, S, R>>>,
) -> ServerResult<impl IntoResponse>
where
    P: Publisher + Sync + Send,
    C: CacheService + Sync + Send,
    S: CrawlerService + Sync + Send,
    R: LoadTopic + Sync + Send,
{
    let workers = state.workers();
    let workers_guard = workers.read().await;

    let response = workers_guard
        .values()
        .filter_map(|worker| {
            let is_launched = !worker.worker().is_finished();
            let worker_config = worker.config().as_ref();
            let config = RssConfigForm::from(worker_config);
            let response = GetInfoResponse::builder()
                .source_name(worker_config.source_name().to_owned())
                .source_url(worker_config.target_url().to_owned())
                .configuration(Some(config))
                .is_launched(is_launched)
                .build()
                .ok();

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
pub async fn get_worker_info<P, C, S, R>(
    State(state): State<Arc<ServerApp<P, C, S, R>>>,
    Json(form): Json<GetInfoForm>,
) -> ServerResult<impl IntoResponse>
where
    P: Publisher + Sync + Send,
    C: CacheService + Sync + Send,
    S: CrawlerService + Sync + Send,
    R: LoadTopic + Sync + Send,
{
    let workers = state.workers();
    let workers_guard = workers.read().await;

    let worker_name = form.source_url();
    let Some(worker) = workers_guard.get(worker_name) else {
        let msg = format!("there is no any worker with name: {worker_name}");
        tracing::warn!("{}", &msg);
        return Err(ServerError::NotFound(msg));
    };

    let is_launched = !worker.worker().is_finished();
    let worker_config = worker.config().as_ref();
    let config_form = RssConfigForm::from(worker_config);
    let response = GetInfoResponse::builder()
        .source_name(form.source_name().to_owned())
        .source_url(form.source_url().to_owned())
        .is_launched(is_launched)
        .configuration(Some(config_form))
        .build()
        .map_err(|err| ServerError::InternalError(err.to_string()))?;

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
pub async fn create_worker<P, C, S, R>(
    State(state): State<Arc<ServerApp<P, C, S, R>>>,
    Json(form): Json<CreateWorkerForm>,
) -> ServerResult<impl IntoResponse>
where
    P: Publisher + Sync + Send + 'static,
    C: CacheService + Sync + Send + 'static,
    S: CrawlerService + Sync + Send + 'static,
    R: LoadTopic + Sync + Send + 'static,
{
    let workers = state.workers();
    let mut workers_guard = workers.write().await;

    let worker_name = form.target_url();
    if let Some(worker) = workers_guard.get(worker_name) {
        tracing::info!(worker = worker_name, "worker already exists");
        if !worker.worker().is_finished() && !form.create_force() {
            let msg = format!("worker {worker_name} already launched");
            return Err(ServerError::Launched(msg));
        }
    }

    let config = form.to_rss_config();
    let cache = state.cache.clone();
    let crawler = state.crawler.clone();
    let publish = state.publish.clone();
    let feeds = RssFeeds::new(config.clone(), publish, cache, crawler).unwrap();

    let task = tokio::spawn(async move { feeds.launch_fetching().await });

    let rss_worker = RssWorker::new(Arc::new(config), task);
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
pub async fn restart_worker<P, C, S, R>(
    State(state): State<Arc<ServerApp<P, C, S, R>>>,
    Json(form): Json<CreateWorkerForm>,
) -> ServerResult<impl IntoResponse>
where
    P: Publisher + Sync + Send + 'static,
    C: CacheService + Sync + Send + 'static,
    S: CrawlerService + Sync + Send + 'static,
    R: LoadTopic + Sync + Send + 'static,
{
    let workers = state.workers();
    let mut workers_guard = workers.write().await;

    let worker_name = form.target_url();
    let Some(worker) = workers_guard.get(worker_name) else {
        let msg = format!("there is no any worker with name: {worker_name}");
        tracing::warn!("{}", &msg);
        return Err(ServerError::NotFound(msg));
    };

    worker.worker().abort();

    let config = form.to_rss_config();
    let cache = state.cache.clone();
    let crawler = state.crawler.clone();
    let publish = state.publish.clone();
    let feeds = RssFeeds::new(config.clone(), publish, cache, crawler).unwrap();

    let task = tokio::spawn(async move { feeds.launch_fetching().await });

    let rss_worker = RssWorker::new(Arc::new(config), task);
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
pub async fn terminate_worker<P, C, S, R>(
    State(state): State<Arc<ServerApp<P, C, S, R>>>,
    Json(form): Json<TerminateWorkerForm>,
) -> ServerResult<impl IntoResponse>
where
    P: Publisher + Sync + Send,
    C: CacheService + Sync + Send,
    S: CrawlerService + Sync + Send,
    R: LoadTopic + Sync + Send,
{
    let workers = state.workers();
    let workers_guard = workers.read().await;

    let worker_name = form.source_url();
    let Some(worker) = workers_guard.get(worker_name) else {
        let msg = format!("there is not worker with name: {worker_name}");
        tracing::warn!("{}", &msg);
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
pub async fn delete_worker<P, C, S, R>(
    State(state): State<Arc<ServerApp<P, C, S, R>>>,
    Json(form): Json<DeleteWorkerForm>,
) -> ServerResult<impl IntoResponse>
where
    P: Publisher + Sync + Send,
    C: CacheService + Sync + Send,
    S: CrawlerService + Sync + Send,
    R: LoadTopic + Sync + Send,
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

#[utoipa::path(
    get,
    path = "/sources/all",
    tag = "sources",
    responses(
        (
            status = 200,
            description = "Successful",
            body = Vec<GetSourcesResponse>,
            example = json!(vec![GetSourcesResponse::example(None)]),
        ),
        (
            status = 400,
            description = "Failed to load all source",
            body = ServerError,
            example = json!(ServerError::example(Some("failed to load all source".to_string()))),
        ),
        (
            status = 503,
            description = "Server does not available",
            body = ServerError,
            example = json!(ServerError::example(None)),
        ),
    )
)]
pub async fn all_sources<P, C, S, R>(
    State(state): State<Arc<ServerApp<P, C, S, R>>>,
) -> ServerResult<impl IntoResponse>
where
    P: Publisher + Sync + Send,
    C: CacheService + Sync + Send,
    S: CrawlerService + Sync + Send,
    R: LoadTopic<Topic = PgsqlTopicModel, Error = sqlx::Error> + Sync + Send,
{
    let storage = state.storage();
    let sources = storage
        .load_all()
        .await
        .map_err(|err| ServerError::InternalError(err.to_string()))?
        .into_iter()
        .map(GetSourcesResponse::from)
        .collect::<Vec<_>>();

    Ok(Json(sources))
}

#[utoipa::path(
    put,
    path = "/sources/add",
    tag = "sources",
    request_body(
        content = CreateSourceForm,
        example = json!(CreateSourceForm::example(None)),
    ),
    responses(
        (
            status = 200,
            description = "Successful",
            body = Vec<Success>,
            example = json!(Success::example(None)),
        ),
        (
            status = 400,
            description = "Failed to create source",
            body = ServerError,
            example = json!(ServerError::example(Some("failed to create source".to_string()))),
        ),
        (
            status = 503,
            description = "Server does not available",
            body = ServerError,
            example = json!(ServerError::example(None)),
        ),
    )
)]
pub async fn add_source<P, C, S, R>(
    State(state): State<Arc<ServerApp<P, C, S, R>>>,
    Json(form): Json<CreateSourceForm>,
) -> ServerResult<impl IntoResponse>
where
    P: Publisher + Sync + Send,
    C: CacheService + Sync + Send,
    S: CrawlerService + Sync + Send,
    R: LoadTopic<Topic = PgsqlTopicModel, Error = sqlx::Error> + Sync + Send,
{
    let storage = state.storage();
    storage
        .add_source(&form.into())
        .await
        .map_err(|err| ServerError::InternalError(err.to_string()))?;

    Ok(Json(Success::default()))
}

#[utoipa::path(
    delete,
    path = "/sources/{source_id}",
    tag = "sources",
    responses(
        (
            status = 200,
            description = "Successful",
            body = Vec<Success>,
            example = json!(Success::example(None)),
        ),
        (
            status = 400,
            description = "Failed to remove source",
            body = ServerError,
            example = json!(ServerError::example(Some("failed to remove source".to_string()))),
        ),
        (
            status = 503,
            description = "Server does not available",
            body = ServerError,
            example = json!(ServerError::example(None)),
        ),
    )
)]
pub async fn remove_source<P, C, S, R>(
    State(state): State<Arc<ServerApp<P, C, S, R>>>,
    Path(source_id): Path<i32>,
) -> ServerResult<impl IntoResponse>
where
    P: Publisher + Sync + Send,
    C: CacheService + Sync + Send,
    S: CrawlerService + Sync + Send,
    R: LoadTopic<TopicId = i32, Topic = PgsqlTopicModel, Error = sqlx::Error> + Sync + Send,
{
    let storage = state.storage();
    storage
        .remove_source(source_id)
        .await
        .map_err(|err| ServerError::InternalError(err.to_string()))?;

    Ok(Json(Success::default()))
}

#[utoipa::path(
    post,
    path = "/sources/search",
    tag = "sources",
    request_body(
        content = SearchSourcesForm,
        example = json!(SearchSourcesForm::example(None)),
    ),
    responses(
        (
            status = 200,
            description = "Successful",
            body = Vec<GetSourcesResponse>,
            example = json!(vec![GetSourcesResponse::example(None)]),
        ),
        (
            status = 400,
            description = "Failed to search source",
            body = ServerError,
            example = json!(ServerError::example(Some("failed to search source".to_string()))),
        ),
        (
            status = 503,
            description = "Server does not available",
            body = ServerError,
            example = json!(ServerError::example(None)),
        ),
    )
)]
pub async fn search_sources<P, C, S, R>(
    State(state): State<Arc<ServerApp<P, C, S, R>>>,
    Json(form): Json<SearchSourcesForm>,
) -> ServerResult<impl IntoResponse>
where
    P: Publisher + Sync + Send,
    C: CacheService + Sync + Send,
    S: CrawlerService + Sync + Send,
    R: LoadTopic<Topic = PgsqlTopicModel, Error = sqlx::Error> + Sync + Send,
{
    let storage = state.storage();
    let founded_topics = storage
        .search_source(form.query())
        .await
        .map_err(|err| ServerError::InternalError(err.to_string()))?
        .into_iter()
        .map(GetSourcesResponse::from)
        .collect::<Vec<_>>();

    Ok(Json(founded_topics))
}

#[utoipa::path(
    patch,
    path = "/sources/update",
    tag = "sources",
    request_body(
        content = CreateSourceForm,
        example = json!(CreateSourceForm::example(None)),
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
            description = "Failed to update source",
            body = ServerError,
            example = json!(ServerError::example(Some("failed to update source".to_string()))),
        ),
        (
            status = 503,
            description = "Server does not available",
            body = ServerError,
            example = json!(ServerError::example(None)),
        ),
    )
)]
pub async fn update_source<P, C, S, R>(
    State(state): State<Arc<ServerApp<P, C, S, R>>>,
    Json(form): Json<CreateSourceForm>,
) -> ServerResult<impl IntoResponse>
where
    P: Publisher + Sync + Send,
    C: CacheService + Sync + Send,
    S: CrawlerService + Sync + Send,
    R: LoadTopic<Topic = PgsqlTopicModel, Error = sqlx::Error> + Sync + Send,
{
    let storage = state.storage();
    storage
        .update_source(&form.into())
        .await
        .map_err(|err| ServerError::InternalError(err.to_string()))?;

    Ok(Json(Success::default()))
}
