use std::sync::Arc;

use axum::{response::IntoResponse, routing::{post,delete,get}, Extension, Json, Router};

use crate::{auth::JWTAuthMiddleware, database::favorites::FavoritesExt, dtos::{FilterTrackDto, Response, SaveFavoritesDto, TrackResponseDto}, error::HttpError, AppState};

pub fn favorites_handler() -> Router {
    Router::new()
        .route("/", post(save_favorite))
        .route("/", delete(delete_favorite))
        .route("/", get(get_user_favorite_tracks))
}


pub async fn save_favorite(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
    Json(body): Json<SaveFavoritesDto>
) -> Result<impl IntoResponse, HttpError> {
    let track_id = body.track_id;

    if track_id.is_nil() {
        return Err(HttpError::bad_request("Track ID is required".to_string()))?;
    }

    let user = &user.user;
    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();

    app_state.db_client
        .save_favorite(track_id.clone(), user_id.clone())
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let response = Response {
        message: "Favorites saved successfully".to_string(),
        status: "success",
    };

    Ok(Json(response))
}


pub async fn delete_favorite(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
    Json(body): Json<SaveFavoritesDto>
) -> Result<impl IntoResponse, HttpError> {
    let track_id = body.track_id;

    if track_id.is_nil() {
        return Err(HttpError::bad_request("Track ID is required".to_string()))?;
    }

    let user = &user.user;
    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();

    app_state.db_client
        .delete_favorite(track_id.clone(), user_id.clone())
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let response = Response {
        message: "Favorites remove successfully".to_string(),
        status: "success",
    };

    Ok(Json(response))
}

pub async fn get_user_favorite_tracks(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>
) -> Result<impl IntoResponse, HttpError> {
    let user = &user.user;

    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();

    let tracks = app_state.db_client
        .get_user_favorite_tracks(user_id.clone())
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let filter_tracks = FilterTrackDto::filter_tracks(&tracks);

    let response = TrackResponseDto {
        tracks: filter_tracks,
    };

    Ok(Json(response))
}