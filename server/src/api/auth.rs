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

pub async fn register(
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<LoginResponse>, AppError> {
    let password_hash = hash(&req.password, DEFAULT_COST)?;

    // Use raw SQL to handle datetime properly
    let mut result = get_db()
        .query(r#"
            CREATE user CONTENT {
                email: $email,
                username: $username,
                password_hash: $password_hash,
                profile_picture: NONE,
                elo: 1200,
                is_admin: false,
                created_at: time::now(),
                updated_at: time::now()
            };
        "#)
        .bind(("email", req.email.clone()))
        .bind(("username", req.username.clone()))
        .bind(("password_hash", password_hash))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let created: Vec<User> = result
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;

    let created_user = created
        .into_iter()
        .next()
        .ok_or_else(|| AppError::Database("Failed to create user".to_string()))?;

    let token = create_jwt(&created_user)?;

    let profile = UserProfile {
        id: created_user.id.as_ref().unwrap().to_string(),
        email: created_user.email,
        username: created_user.username,
        profile_picture: created_user.profile_picture,
        elo: created_user.elo,
        games_played: 0,
        games_won: 0,
        win_rate: 0.0,
    };

    Ok(Json(LoginResponse { token, user: profile }))
}

pub async fn login(Json(req): Json<LoginRequest>) -> Result<Json<LoginResponse>, AppError> {
    // Find user by email for password verification
    let mut result = get_db()
        .query("SELECT * FROM user WHERE email = $email")
        .bind(("email", req.email.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let users: Vec<User> = result
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    let user = users.into_iter().next();
    
    let user = user.ok_or(AppError::InvalidCredentials)?;

    if !verify(&req.password, &user.password_hash)? {
        return Err(AppError::InvalidCredentials);
    }

    let token = create_jwt(&user)?;
    let user_id = user.id.as_ref().unwrap().to_string();
    let user_id_clean = user_id.split(':').last().unwrap_or(&user_id).to_string();
    
    // Get full profile with game statistics using SurrealQL - all logic inside SurrealQL
    let user_record_id = format!("user:{}", user_id_clean);
    
    // First, get the basic profile
    let mut result = get_db()
        .query(r#"
            SELECT 
                string::concat('user:', id.id) as id_str,
                email,
                username,
                profile_picture,
                elo
            FROM user
            WHERE id = type::thing($uid)
            LIMIT 1;
        "#)
        .bind(("uid", user_record_id.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    let basic_profiles: Vec<serde_json::Value> = result
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    if basic_profiles.is_empty() {
        return Err(AppError::Database("User not found".to_string()));
    }
    
    let basic_profile = &basic_profiles[0];
    
    // Get game statistics separately
    let mut result = get_db()
        .query(r#"
            SELECT 
                count() as total_games
            FROM game 
            WHERE status = 'completed' 
            AND type::thing($uid) IN [player1, player2];
        "#)
        .bind(("uid", user_record_id.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    let game_counts: Vec<serde_json::Value> = result
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    let total_games = game_counts.get(0)
        .and_then(|v| v.get("total_games"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
    
    // Get won games
    let mut result = get_db()
        .query(r#"
            SELECT 
                count() as won_games
            FROM game 
            WHERE status = 'completed' 
            AND winner = type::thing($uid);
        "#)
        .bind(("uid", user_record_id))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    let won_counts: Vec<serde_json::Value> = result
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    let won_games = won_counts.get(0)
        .and_then(|v| v.get("won_games"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
    
    let win_rate = if total_games > 0 {
        (won_games as f64 * 100.0 / total_games as f64).round()
    } else {
        0.0
    };
    
    // Construct the profile
    let profile = UserProfile {
        id: user_id.clone(),
        email: basic_profile.get("email")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        username: basic_profile.get("username")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        profile_picture: basic_profile.get("profile_picture")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        elo: basic_profile.get("elo")
            .and_then(|v| v.as_i64())
            .unwrap_or(1200) as i32,
        games_played: total_games,
        games_won: won_games,
        win_rate,
    };
    
    
    Ok(Json(LoginResponse { token, user: profile }))
}

pub async fn me(AuthUser(claims): AuthUser) -> Result<Json<UserProfile>, AppError> {
    let user_id = claims.user_id.split(':').last().unwrap_or(&claims.user_id).to_string();

    // Get full profile with stats using SurrealQL - all logic in SurrealQL
    let user_record_id = format!("user:{}", user_id);
    
    // First, get the basic profile
    let mut result = get_db()
        .query(r#"
            SELECT 
                string::concat('user:', id.id) as id_str,
                email,
                username,
                profile_picture,
                elo
            FROM user
            WHERE id = type::thing($uid)
            LIMIT 1;
        "#)
        .bind(("uid", user_record_id.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    let basic_profiles: Vec<serde_json::Value> = result
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    if basic_profiles.is_empty() {
        return Err(AppError::Database("User not found".to_string()));
    }
    
    let basic_profile = &basic_profiles[0];
    
    // Get game statistics separately
    let mut result = get_db()
        .query(r#"
            SELECT 
                count() as total_games
            FROM game 
            WHERE status = 'completed' 
            AND type::thing($uid) IN [player1, player2];
        "#)
        .bind(("uid", user_record_id.clone()))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    let game_counts: Vec<serde_json::Value> = result
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    let total_games = game_counts.get(0)
        .and_then(|v| v.get("total_games"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
    
    // Get won games
    let mut result = get_db()
        .query(r#"
            SELECT 
                count() as won_games
            FROM game 
            WHERE status = 'completed' 
            AND winner = type::thing($uid);
        "#)
        .bind(("uid", user_record_id))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    let won_counts: Vec<serde_json::Value> = result
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    let won_games = won_counts.get(0)
        .and_then(|v| v.get("won_games"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0) as i32;
    
    let win_rate = if total_games > 0 {
        (won_games as f64 * 100.0 / total_games as f64).round()
    } else {
        0.0
    };
    
    // Construct the profile
    let profile = UserProfile {
        id: claims.user_id.clone(),
        email: basic_profile.get("email")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        username: basic_profile.get("username")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        profile_picture: basic_profile.get("profile_picture")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        elo: basic_profile.get("elo")
            .and_then(|v| v.as_i64())
            .unwrap_or(1200) as i32,
        games_played: total_games,
        games_won: won_games,
        win_rate,
    };

    Ok(Json(profile))
}

#[derive(Debug)]
pub enum AppError {
    Database(String),
    InvalidCredentials,
    NotFound,
    #[allow(dead_code)]
    Bcrypt(bcrypt::BcryptError),
    #[allow(dead_code)]
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
