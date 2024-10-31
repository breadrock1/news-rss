use crate::server::forms::*;
use crate::server::routers::*;

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

const SWAGGER_TARGET_URL: &str = "/swagger";
const SWAGGER_FILE_URL: &str = "/api-docs/openapi.json";

pub fn init_swagger() -> SwaggerUi {
    let api_doc = ApiDoc::openapi();
    SwaggerUi::new(SWAGGER_TARGET_URL).url(SWAGGER_FILE_URL, api_doc)
}

#[derive(OpenApi)]
#[openapi(
    info(
        description = "There is API routers of news-rss project."
    ),
    tags(
        (
            name = "news-rss project swagger", 
            description = "There are all available news-rss project routers.",
        ),
    ),
    paths(
        get_workers,
        get_worker_info,
        create_worker,
        restart_worker,
        terminate_worker,
    ),
    components(
        schemas(
            GetInfoForm,
            GetInfoResponse,
            CreateWorkerForm,
            TerminateWorkerForm,
        ),
    ),
)]
struct ApiDoc;

pub trait SwaggerExamples {
    type Example: serde::Serialize;

    fn example(value: Option<String>) -> Self::Example;
}
