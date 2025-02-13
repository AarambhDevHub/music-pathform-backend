use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket}, State, WebSocketUpgrade
    }, http::Version, response::IntoResponse, routing::{any, get}, Extension, Json, Router
};
use serde::Serialize;
use tokio::sync::broadcast;

use crate::{
    auth::JWTAuthMiddleware, database::history::HistoryExt, dtos::{FilterTrackDto, PlaybackMessageDto, TrackResponseDto}, error::HttpError, AppState
};

pub fn history_handler() -> Router {
    Router::new()
        .route("/", get(get_user_playback_history))
        .route("/add", any(add_history))
            .with_state(broadcast::channel::<String>(16).0)
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

pub async fn add_history(
    ws: WebSocketUpgrade,
    version: Version,
    State(tx): State<broadcast::Sender<String>>,
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>
) -> impl IntoResponse {
    println!("accepted a WebSocket using {version:?}");
    ws.on_upgrade(move |socket| handle_socket(socket, app_state, user, tx))
}

async fn handle_socket(
    mut socket: WebSocket,
    app_state: Arc<AppState>,
    user: JWTAuthMiddleware,
    tx: broadcast::Sender<String>,
) {
    let mut rx = tx.subscribe();
    let user = &user.user;
    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();

    println!("User '{}' connected", user_id.clone());

    loop {
        tokio::select! {
            Some(Ok(msg)) = socket.recv() => {
                if let Message::Text(text) = msg {
                   if let Ok(playback_msg) = serde_json::from_str::<PlaybackMessageDto>(&text) {

                        let track_id = playback_msg.track_id;
                        let duration_played = playback_msg.duration_played;

                        match app_state.db_client
                            .update_or_insert_playback_hisotry(track_id.clone(),user_id.clone(), duration_played)
                            .await {
                                Ok(_) => {
                                    println!("Playback history updated successfully.");
                                }
                                Err(e) => {
                                    println!("Error updating playback history: {}", e);
                                    let error_message = ErrorResponse {
                                        error: format!("Failed to update playback history: {}", e),
                                    };

                                    let _ = socket.send(Message::Text(serde_json::to_string(&error_message).unwrap()));
                                }
                            };

                   } else {
                    // If the message could not be parsed, send an error
                    let error_message = ErrorResponse {
                        error: "Invalid message format.".to_string(),
                    };
                    let _ = socket.send(Message::Text(serde_json::to_string(&error_message).unwrap())).await;
                }

                }
            },
            _  = rx.recv() => {

            }
        }
    }
}


async fn get_user_playback_history(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
) -> Result<impl IntoResponse, HttpError> {
    let user = &user.user;
    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();

    let tracks = app_state.db_client
        .get_user_playback_history(user_id.clone())
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let filter_tracks = FilterTrackDto::filter_tracks(&tracks);

    let response = TrackResponseDto {
        tracks: filter_tracks,
    };
    
    Ok(Json(response))
}