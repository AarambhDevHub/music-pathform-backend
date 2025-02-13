use std::sync::Arc;

use axum::{extract::DefaultBodyLimit, middleware, Extension, Router};
use tower_http::{trace::TraceLayer, services::ServeDir};

use crate::{auth::auth, handler::{auth::auth_handler, favorites::favorites_handler, getfile::get_file_handler, history::history_handler, playlists::playlist_hanlder, upload::upload_handler, users::users_handler}, AppState};

const MAX_FILE_SIZE: usize = 6 * 1024 * 1024; // 5 MB in bytes

pub fn create_router(app_state: Arc<AppState>) -> Router {
    let api_route = Router::new()
    .nest("/auth", auth_handler())
    .nest(
        "/users",
        users_handler()
            .layer(middleware::from_fn(auth))
    )
    .nest(
        "/upload", 
        upload_handler()
            .layer(middleware::from_fn(auth))
            .layer(DefaultBodyLimit::max(MAX_FILE_SIZE))
    )
    .nest(
        "/get", 
        get_file_handler()
            .layer(middleware::from_fn(auth))        
    )
    .nest(
        "/favorite", 
        favorites_handler()
            .layer(middleware::from_fn(auth))
    )
    .nest(
        "/playlist", 
        playlist_hanlder()
            .layer(middleware::from_fn(auth))
    )
    .nest(
        "/history",
        history_handler()
            .layer(middleware::from_fn(auth))
    )
    .nest_service("/assets", ServeDir::new("assets"))
    .layer(TraceLayer::new_for_http())
    .layer(Extension(app_state));

    Router::new().nest("/api", api_route)
}