use std::sync::Arc;

use axum::{
    response::IntoResponse, 
    routing::{get, put}, 
    Extension, 
    Json, 
    Router
};
use validator::Validate;

use crate::{auth::JWTAuthMiddleware, database::users::UserExt, dtos::{FilterUserDto, NameUpdateDto, Response, UserData, UserPasswordUpdateDto, UserResponseDto}, error::{ErrorMessage, HttpError}, utils::password, AppState};

pub fn users_handler() -> Router {
    Router::new()
        .route(
            "/me", 
            get(get_me)
    )
    .route("/name", put(update_user_name))
    .route("/password", put(update_user_password))
}

pub async fn get_me(
    Extension(_app_state): Extension<Arc<AppState>>, // Extract app state
    Extension(user): Extension<JWTAuthMiddleware>,
) -> Result<impl IntoResponse, HttpError> {
    // Filter user data
    let filtered_user = FilterUserDto::filter_user(&user.user);

    // Prepare response data
    let response_data = UserResponseDto {
        status: "success".to_string(),
        data: UserData {
            user: filtered_user,
        },
    };

    // Return JSON response
    Ok(Json(response_data))
}

pub async fn update_user_name(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
    body: Json<NameUpdateDto>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
       .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let user = &user.user;

    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();

    let result = app_state.db_client.update_username(user_id.clone(), &body.name)
                    .await
                    .map_err(|e| HttpError::server_error(e.to_string()))?;
    
    let filtered_user = FilterUserDto::filter_user(&result);

    let response = UserResponseDto {
        data: UserData {
            user: filtered_user,
        },
        status: "success".to_string()
    };

    Ok(Json(response))
}

pub async fn update_user_password(
    Extension(app_state): Extension<Arc<AppState>>,
    Extension(user): Extension<JWTAuthMiddleware>,
    body: Json<UserPasswordUpdateDto>,
) -> Result<impl IntoResponse, HttpError> {
    body.validate()
       .map_err(|e| HttpError::bad_request(e.to_string()))?;

    let user = &user.user;

    let user_id = uuid::Uuid::parse_str(&user.id.to_string()).unwrap();

    let result = app_state.db_client
            .get_user(Some(user_id.clone()), None, None)
            .await
            .map_err(|e| HttpError::server_error(e.to_string()))?;

    let user = result.ok_or(HttpError::unauthorized(ErrorMessage::InvalidToken.to_string()))?;

    let password_match = password::compare(&body.old_password, &user.password_hash)
                        .map_err(|e| HttpError::bad_request(e.to_string()))?;
    
    if !password_match {
        return Err(HttpError::bad_request("Old password is incorrect".to_string()))?;
    }

    let hashed_password = password::hash(&body.new_password)
            .map_err(|e| HttpError::server_error(e.to_string()))?;

    app_state.db_client
        .update_user_password_hash(user_id.clone(), hashed_password.clone())
        .await
        .map_err(|e| HttpError::server_error(e.to_string()))?;

    let response = Response {
        message: "Password updated successfull".to_string(),
        status: "success",
    };

    Ok(Json(response))
}