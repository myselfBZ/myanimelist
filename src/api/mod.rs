pub mod anime;
pub mod media;
pub mod anime_genres;
pub mod extractor;
pub mod openapi;
use aide::{ OperationIo, axum::{IntoApiResponse, routing::{get}}, openapi::OpenApi, swagger::Swagger, };
use axum::{Extension, Json, Router, http::StatusCode, response::IntoResponse};
use schemars::JsonSchema;
use sea_orm::DbErr;
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;
use tower_http::cors;

use crate::State;

pub async fn create_app(state: State) -> anyhow::Result<Router> {
    aide::generate::extract_schemas(true);

    let mut api = OpenApi::default();

    let app = aide::axum::ApiRouter::new()
        .route("/", get(dummy))
        .nest("/animes", anime::router(state.clone()))
        .nest("/anime-genres", anime_genres::router(state.clone()))
        .nest("/upload-media", media::router(state.clone()))
        .route("/api.json", get(serve_docs))
        .route(
            "/swagger",
            get(Swagger::new("/api.json")
                .with_title("myAnimelist")
                .axum_handler()),
        )
        .finish_api_with(&mut api, |op| {
            op.security_scheme(openapi::SECURITY_SCHEME_KEY, openapi::security_scheme())
        })
        .layer(Extension(Arc::new(api)))
        .layer(
            cors::CorsLayer::new()
                .allow_origin(cors::Any)
                .allow_headers(cors::Any),
        )
        .with_state(state);
    Ok(app)
}

async fn dummy() -> &'static str {
    "Hello World"
}

async fn serve_docs(Extension(api): Extension<Arc<OpenApi>>) -> impl IntoApiResponse {
    Json(api).into_response()
}

#[derive(Serialize, Debug, OperationIo)]
pub struct AppResponse<T: Serialize> {
    status_code: u16,
    message: String,
    data: Option<T>,
}

impl<D> IntoResponse for AppResponse<D>
where
    D: Serialize,
{
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::from_u16(self.status_code).unwrap(),
            axum::Json(json!({
                "status_code": self.status_code,
                "message": self.message,
                "data": self.data
            })),
        )
            .into_response()
    }
}

#[derive(Serialize, Debug, OperationIo, JsonSchema)]
pub struct AppError {
    status_code: u16,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::from_u16(self.status_code).unwrap(),
            axum::Json(json!({
                "status_code": self.status_code,
                "message": self.message,
            })),
        )
            .into_response()
    }
}

pub fn consruct_response<T: Serialize>(
    value: Result<T, DbErr>,
    message: String,
    error_message: String,
) -> Result<AppResponse<T>, AppError> {
    match value {
        Ok(anime) => Ok(AppResponse {
            status_code: StatusCode::OK.into(),
            message,
            data: Some(anime),
        }),
        Err(err) => Err(AppError {
            status_code: StatusCode::INTERNAL_SERVER_ERROR.into(),
            message: format!("{:?}: {:?}", error_message, err),
        }),
    }
}
