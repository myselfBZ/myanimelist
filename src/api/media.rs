use crate::objstore::PresignPutReq;
use serde::{Deserialize, Serialize};
use axum::extract::{State, Json};
use crate::{State as AppState, api::AppResponse, api::AppError};
use aide::{
    axum::{ApiRouter, routing::post_with}, transform::TransformOperation,
};
use axum::http;
use schemars::JsonSchema;

pub fn router(state: AppState) -> ApiRouter<AppState> {
    ApiRouter::new()
        .api_route("/", post_with(presign_put_image, presign_put_image_docs))
        .with_state(state)
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, JsonSchema)]
pub struct CreatePresignPutReq {
    content_type: String,
    filename: String,
    dir:      String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, JsonSchema)]
pub struct S3PresignedUrl {
    url: String
}

#[axum::debug_handler]
#[tracing::instrument]
pub async fn presign_put_image(
    State(state): State<AppState>,
    Json(payload): Json<CreatePresignPutReq>
    ) -> Result<AppResponse<S3PresignedUrl>, AppError> {

    tracing::info!("dalbayop?");
    let url = state
        .obj_store
        .presign_put(PresignPutReq{ 
            content_type: payload.content_type, 
            dir: "anime_images".to_string(), 
            filename:  payload.filename,
        })
        .await
        // TODO: fix the default error
        .map_err(|_| AppError {
            status_code: http::StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            message: "we fucked up, sorry".to_string()
        })?;
    Ok(AppResponse { 
        status_code: http::StatusCode::CREATED.as_u16(), 
        message: "here you go".to_string(), 
        data: Some(S3PresignedUrl { url: url })  
    })
}

pub fn presign_put_image_docs(op: TransformOperation) -> TransformOperation {
    op.tag("Media")
}


// Uploading directly to S3 bucket with a stream
// FIXME: axum::Multipart not supported by aide
// #[axum::debug_handler]
// #[tracing::instrument]
// pub async fn upload(
//     State(state): State<AppState>,
//     mut multipart: Multipart
// ) -> Result<AppResponse<S3Url>, AppError> {
//     let field = multipart.next_field()
//         .await
//         .unwrap()
//         .ok_or(AppError {
//             status_code: http::StatusCode::BAD_REQUEST.as_u16(),
//             message: "invalid content-type".to_string() 
//         })?;
//     let content_type = field
//         .content_type()
//         .map(|s| s.to_string())
//         .ok_or(AppError {
//             status_code: http::StatusCode::BAD_REQUEST.as_u16(),
//             message: "invalid content-type".to_string() 
//         })?;
//     let filename = field
//         .file_name()
//         .map(|s| s.to_string())
//         .ok_or(
//             AppError {
//                 status_code: http::StatusCode::BAD_REQUEST.as_u16(),
//                 message: "invalid content-type".to_string() 
//             }
//         )?;
//     let body_with_io_error = field.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
//     let body_reader = StreamReader::new(body_with_io_error);
//     pin_mut!(body_reader);
//     let upload_req = UploadReq {
//         reader: body_reader,
//         content_type: content_type,
//         dir: "anime_images".to_string(),
//         filename: filename,
//     };
//     let out = state.obj_store.put_file(upload_req)
//         .await
//         .map_err(|_| AppError { 
//             status_code: http::StatusCode::INTERNAL_SERVER_ERROR.as_u16(), 
//             message: "internal server error".to_string(), 
//         })?;
//     return Ok(AppResponse{
//         status_code: http::StatusCode::CREATED.as_u16(),
//         message: "created".to_string(),
//         data: Some(S3Url { url:out}),
//     });
// }

// pub fn upload_docs(op: TransformOperation) -> TransformOperation {
//     op.tag("Anime")
// }
