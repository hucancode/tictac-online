use crate::{
    auth::{create_jwt, AuthUser},
    db::get_db,
    models::{CreateUserRequest, LoginRequest, LoginResponse, User, UserProfile},
};
use axum::{
    extract::Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bcrypt::{hash, verify, DEFAULT_COST};
use serde_json::json;
use surrealdb::RecordId;

pub async fn register(
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let password_hash = hash(&req.password, DEFAULT_COST)?;

    let created: Option<User> = get_db()
        .create("user")
        .content(serde_json::json!({
            "email": req.email.clone(),
            "username": req.username.clone(),
            "password_hash": password_hash,
            "profile_picture": null,
            "elo": 1200,
            "games_played": 0,
            "games_won": 0,
            "is_admin": false
        }))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let created_user = created
        .ok_or_else(|| AppError::Database("Failed to create user".to_string()))?;

    let token = create_jwt(&created_user)?;

    let profile = UserProfile {
        id: created_user.id.as_ref().unwrap().to_string(),
        email: created_user.email,
        username: created_user.username,
        profile_picture: created_user.profile_picture,
        elo: created_user.elo,
        games_played: created_user.games_played,
        games_won: created_user.games_won,
        win_rate: 0.0,
    };

    Ok(Json(LoginResponse { token, user: profile }))
}

pub async fn login(Json(req): Json<LoginRequest>) -> Result<Json<LoginResponse>, AppError> {
    let mut result = get_db()
        .query("SELECT * FROM user WHERE email = $email")
        .bind(("email", req.email.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let users: Vec<User> = result
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;

    let user = users
        .into_iter()
        .next()
        .ok_or_else(|| AppError::InvalidCredentials)?;

    if !verify(&req.password, &user.password_hash)? {
        return Err(AppError::InvalidCredentials);
    }

    let token = create_jwt(&user)?;

    let win_rate = if user.games_played > 0 {
        (user.games_won as f64 / user.games_played as f64) * 100.0
    } else {
        0.0
    };

    let profile = UserProfile {
        id: user.id.as_ref().unwrap().to_string(),
        email: user.email,
        username: user.username,
        profile_picture: user.profile_picture,
        elo: user.elo,
        games_played: user.games_played,
        games_won: user.games_won,
        win_rate,
    };

    Ok(Json(LoginResponse { token, user: profile }))
}

pub async fn me(AuthUser(claims): AuthUser) -> Result<Json<UserProfile>, AppError> {
    let user_id = RecordId::from(("user", claims.user_id.as_str()));
    
    let user: Option<User> = get_db()
        .select(user_id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let user = user.ok_or_else(|| AppError::NotFound)?;

    let win_rate = if user.games_played > 0 {
        (user.games_won as f64 / user.games_played as f64) * 100.0
    } else {
        0.0
    };

    let profile = UserProfile {
        id: user.id.as_ref().unwrap().to_string(),
        email: user.email,
        username: user.username,
        profile_picture: user.profile_picture,
        elo: user.elo,
        games_played: user.games_played,
        games_won: user.games_won,
        win_rate,
    };

    Ok(Json(profile))
}

#[derive(Debug)]
pub enum AppError {
    Database(String),
    InvalidCredentials,
    NotFound,
    Bcrypt(bcrypt::BcryptError),
    Jwt(jsonwebtoken::errors::Error),
}

impl From<bcrypt::BcryptError> for AppError {
    fn from(err: bcrypt::BcryptError) -> Self {
        AppError::Bcrypt(err)
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AppError::Jwt(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Database(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            AppError::Bcrypt(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Password error".to_string()),
            AppError::Jwt(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Token error".to_string()),
        };

        let body = Json(json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}