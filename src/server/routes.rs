use super::{
    config::{ServerConfig, MULTIPART_OVERHEAD_BYTES},
    conversion::run_conversion,
    error::ApiError,
    request::parse_multipart,
    response::header_name,
    state::AppState,
};
use axum::{
    extract::{DefaultBodyLimit, Multipart, State},
    http::{
        header::{CONTENT_DISPOSITION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    response::Response,
    routing::{get, post},
    Router,
};
use tower_http::cors::CorsLayer;

pub(super) fn build_router(config: ServerConfig, allowed_origin: HeaderValue) -> Router {
    let exposed_headers = [
        CONTENT_DISPOSITION,
        header_name("x-input-size"),
        header_name("x-output-size"),
        header_name("x-input-width"),
        header_name("x-input-height"),
        header_name("x-output-width"),
        header_name("x-output-height"),
    ];
    let cors = CorsLayer::new()
        .allow_origin(allowed_origin)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([CONTENT_TYPE])
        .expose_headers(exposed_headers);

    Router::new()
        .route("/healthz", get(healthz))
        .route(
            "/v1/convert",
            post(convert).layer(DefaultBodyLimit::max(
                config.max_upload_bytes + MULTIPART_OVERHEAD_BYTES,
            )),
        )
        .layer(cors)
        .with_state(AppState::new(config))
}

async fn healthz() -> &'static str {
    "ok"
}

async fn convert(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<Response, ApiError> {
    let request = parse_multipart(multipart, state.config.max_upload_bytes).await?;
    let converted = run_conversion(state, request).await?;
    Ok(converted.into_response())
}
