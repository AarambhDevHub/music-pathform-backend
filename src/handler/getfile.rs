use std::{path::PathBuf, sync::Arc};

use axum::{body::Body, extract::{Path, Request}, http::{header, Response}, response::IntoResponse, routing::get, Extension, Json, Router};

use crate::{
    auth::JWTAuthMiddleware, 
    database::{track::TrackExt, upload::UploadExt}, 
    dtos::{FilterTrackDto, IncompleteTrackInfoResponse, TrackResponseDto}, 
    error::HttpError, 
    AppState
};

use tokio::fs;

pub fn get_file_handler() -> Router {
    Router::new()
        .route("/incomplete", get(get_incomplete_uploads_handler))
        .route("/track", get(get_random_tracks_handler))
        .route("/play/:file_name", get(stream_audio))
}

pub async fn get_incomplete_uploads_handler(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
) -> Result<impl IntoResponse, HttpError> {

    let user = &user.user;
    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();

    let uploads = app_state.db_client
        .get_incomplete_uploads(user_id.clone())
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let rsponse = IncompleteTrackInfoResponse {
        incomplete_track_info: uploads,
    };

    Ok(Json(rsponse))
}

pub async fn get_random_tracks_handler(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>
) -> Result<impl IntoResponse, HttpError> {
    let user = &user.user;

    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();

    let tracks = app_state.db_client.get_random_tracks(user_id.clone())
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let filter_tracks = FilterTrackDto::filter_tracks(&tracks);

    let response = TrackResponseDto {
        tracks: filter_tracks,
    };

    Ok(Json(response))
}

async fn stream_audio(
    Path(file_name): Path<String>,
    Extension(_app_state): Extension<Arc<AppState>>,
    Extension(_user): Extension<JWTAuthMiddleware>,
    req: Request<Body>,
) -> Result<impl IntoResponse, HttpError> {
    let file_path = PathBuf::from("uploads/").join(file_name);

    let headers = req.headers();

    match fs::read(&file_path).await {
        Ok(content) => {
            // Log or process headers if needed
            if let Some(range) = headers.get("Range") {
                println!("Range header: {:?}", range);
                // Implement range handling logic here if needed
                let range_str = range.to_str().unwrap_or("");
                let parts: Vec<&str> = range_str.split('=').collect();
                if parts.len() == 2 && parts[0] == "bytes" {
                    let ranges: Vec<&str> = parts[1].split('-').collect();
                    if let Some(start) = ranges.get(0).and_then(|s| s.parse::<u64>().ok()) {
                        let end = if let Some(end_str) = ranges.get(1) {
                            end_str.parse::<u64>().ok().unwrap_or_else(|| content.len() as u64 - 1)
                        } else {
                            content.len() as u64 - 1
                        };

                        // Create a Vec<u8> for the body
                        let body = content[start as usize..=end as usize].to_vec();
                        let response: Response<Body> = Response::builder()
                            .status(206) // Partial content status
                            .header(header::CONTENT_TYPE, "audio/mpeg")
                            .header(header::CONTENT_RANGE, format!("bytes {}-{}/{}", start, end, content.len()))
                            .body(Body::from(body))
                            .unwrap();

                        return Ok(response);
                    }
                }
            }

            // If no Range header, return the whole content
            let response: Response<Body> = Response::builder()
                .header(header::CONTENT_TYPE, "audio/mpeg")
                .header(header::CONTENT_DISPOSITION, "inline")
                .header(header::ACCEPT_RANGES, "bytes") // Allow range requests
                .body(Body::from(content)) // Take ownership of content
                .unwrap();

            Ok(response)
        }
        Err(_) => Err(HttpError::server_error("File Not Found")),
    }

}
