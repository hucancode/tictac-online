use crate::{
    api::auth::AppError,
    auth::AdminUser,
    db::get_db,
    models::{AdminUpdateUserRequest, User, UserProfile},
};
use axum::{
    extract::{Path, Query},
    Json,
};
use serde::Deserialize;
use surrealdb::RecordId;

#[derive(Deserialize)]
pub struct UserListQuery {
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(default)]
    offset: usize,
    #[serde(default)]
    search: Option<String>,
}

fn default_limit() -> usize {
    50
}

pub async fn list_users(
    _admin: AdminUser,
    Query(params): Query<UserListQuery>,
) -> Result<Json<Vec<UserProfile>>, AppError> {
    let mut result = if let Some(ref search) = params.search {
        get_db()
            .query(
                r#"
                SELECT * FROM user 
                WHERE email ~ $search OR username ~ $search
                ORDER BY created_at DESC 
                LIMIT $limit 
                START $offset
                "#,
            )
            .bind(("search", search.clone()))
            .bind(("limit", params.limit))
            .bind(("offset", params.offset))
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
    } else {
        get_db()
            .query(
                r#"
                SELECT * FROM user 
                ORDER BY created_at DESC 
                LIMIT $limit 
                START $offset
                "#,
            )
            .bind(("limit", params.limit))
            .bind(("offset", params.offset))
            .await
            .map_err(|e| AppError::Database(e.to_string()))?
    };

    let users: Vec<User> = result
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;

    let profiles: Vec<UserProfile> = users
        .into_iter()
        .map(|user| {
            let win_rate = if user.games_played > 0 {
                (user.games_won as f64 / user.games_played as f64) * 100.0
            } else {
                0.0
            };

            UserProfile {
                id: user.id.as_ref().unwrap().to_string(),
                email: user.email,
                username: user.username,
                profile_picture: user.profile_picture,
                elo: user.elo,
                games_played: user.games_played,
                games_won: user.games_won,
                win_rate,
            }
        })
        .collect();

    Ok(Json(profiles))
}

pub async fn update_user(
    _admin: AdminUser,
    Path(user_id): Path<String>,
    Json(req): Json<AdminUpdateUserRequest>,
) -> Result<Json<UserProfile>, AppError> {
    let user_id = RecordId::from(("user", user_id.as_str()));
    
    let mut update_data = serde_json::json!({
        "updated_at": chrono::Utc::now(),
    });

    if let Some(email) = req.email {
        update_data["email"] = serde_json::json!(email);
    }
    if let Some(username) = req.username {
        update_data["username"] = serde_json::json!(username);
    }
    if let Some(elo) = req.elo {
        update_data["elo"] = serde_json::json!(elo);
    }
    if let Some(is_admin) = req.is_admin {
        update_data["is_admin"] = serde_json::json!(is_admin);
    }

    let _: Option<User> = get_db()
        .update(user_id.clone())
        .merge(update_data)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

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

pub async fn delete_user(
    _admin: AdminUser,
    Path(user_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = RecordId::from(("user", user_id.as_str()));
    
    let _: Option<User> = get_db()
        .delete(user_id)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "User deleted successfully"
    })))
}

pub async fn get_stats(_admin: AdminUser) -> Result<Json<serde_json::Value>, AppError> {
    let mut result = get_db()
        .query(
            r#"
            SELECT 
                count() as total_users,
                math::sum(games_played) as total_games,
                math::avg(elo) as average_elo
            FROM user
            "#,
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let stats: Option<serde_json::Value> = result
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(stats.unwrap_or_else(|| serde_json::json!({
        "total_users": 0,
        "total_games": 0,
        "average_elo": 1200
    }))))
}