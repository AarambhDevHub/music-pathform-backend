use std::{
    fs::{self, File},
    io::Write,
    sync::Arc,
};

use axum::{
    extract::{Multipart, Path}, response::IntoResponse, routing::{get, post}, Extension, Json, Router
};

use crate::{
    auth::JWTAuthMiddleware,
    database::playlists::PlaylistsExt,
    dtos::{AddTrackPlaylist, FilterTrackDto, PlayListResponse, Response, TrackResponseDto},
    error::HttpError,
    AppState,
};

pub fn playlist_hanlder() -> Router {
    Router::new()
        .route("/", post(create_playlist))
        .route("/add", post(add_track_to_playlist))
        .route("/", get(get_user_playlists))
        .route("/:playlist_id", get(get_playlists_tracks))
}

pub async fn create_playlist(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, HttpError> {
    let user = &user.user;

    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();

    let mut title = String::new();
    let mut thumbnail_name = String::new();
    let mut thumbnail_data = Vec::new();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let field_name = field.name().unwrap_or_default().to_string();
        match field_name.as_str() {
            "title" => {
                title = field.text().await.unwrap();
            }
            "thumbnail" => {
                thumbnail_name = field.file_name().unwrap_or_default().to_string();
                match field.bytes().await {
                    Ok(bytes) => thumbnail_data = bytes.to_vec(),
                    Err(err) => {
                        eprintln!("Error reading chunk data: {:?}", err);
                        return Err(HttpError::bad_request("File Upload failed"));
                    }
                }
            }
            _ => return Err(HttpError::bad_request("File Upload failed")),
        }
    }

    if thumbnail_name.is_empty() || thumbnail_data.is_empty() {
        return Err(HttpError::bad_request("Thumbnail is missing"));
    }

    let thumbnail_dir = format!("assets/playlist/");
    if let Err(_err) = fs::create_dir_all(&thumbnail_dir) {
        return Err(HttpError::server_error("Createing failed"));
    }

    let thumbnail_file_path = format!("{}/{}", &thumbnail_dir, &thumbnail_name);
    let mut file = match File::create(&thumbnail_file_path) {
        Ok(f) => f,
        Err(err) => {
            return Err(HttpError::server_error(err.to_string()));
        }
    };

    if let Err(_err) = file.write_all(&thumbnail_data) {
        return Err(HttpError::server_error("Createing failed"));
    }

    app_state
        .db_client
        .create_playlist(user_id.clone(), title, thumbnail_name)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let response = Response {
        status: "success",
        message: "Playlist created successfull!".to_string(),
    };

    Ok(Json(response))
}

pub async fn add_track_to_playlist(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(_user): Extension<JWTAuthMiddleware>,
    Json(body): Json<AddTrackPlaylist>,
) -> Result<impl IntoResponse, HttpError> {
    let playlist_id = body.playlist_id;
    let track_id = body.track_id;

    if playlist_id.is_nil() || track_id.is_nil() {
        return Err(HttpError::bad_request(
            "Playlist id and track id are required",
        ))?;
    }

    let track_order = app_state
        .db_client
        .get_last_track_order(playlist_id.clone())
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    app_state
        .db_client
        .add_track_to_playlist(playlist_id.clone(), track_id.clone(), track_order + 1)
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let response = Response {
        status: "success",
        message: "Track added to playlist successfully!".to_string(),
    };

    Ok(Json(response))
}

pub async fn get_user_playlists(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
) -> Result<impl IntoResponse, HttpError> {
    let user_id = uuid::Uuid::parse_str(&user.user.id.to_string()).unwrap();

    let playlists = app_state
        .db_client
        .get_user_playlists(user_id.clone())
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let response = PlayListResponse { playlists };

    Ok(Json(response))
}

pub async fn get_playlists_tracks(
    Path(playlist_id): Path<uuid::Uuid>,
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
) -> Result<impl IntoResponse, HttpError> {
    let user_id = uuid::Uuid::parse_str(&user.user.id.to_string()).unwrap();

    let tracks = app_state
        .db_client
        .get_playlists_tracks(playlist_id.clone(), user_id.clone())
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let filter_tracks = FilterTrackDto::filter_tracks(&tracks);

    let response = TrackResponseDto {
        tracks: filter_tracks,
    };

    Ok(Json(response))
}
