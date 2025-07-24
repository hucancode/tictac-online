use crate::{
    api::auth::AppError,
    auth::AdminUser,
    db::get_db,
};
use axum::Json;
use serde_json::json;

pub async fn get_database_info(
    _admin: AdminUser,
) -> Result<Json<serde_json::Value>, AppError> {
    let db = get_db();
    
    // Count users
    let mut users_count_result = db
        .query("SELECT count() as total FROM user")
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    let users_count_vec: Vec<serde_json::Value> = users_count_result
        .take(0)
        .unwrap_or_default();
    
    let users_count = users_count_vec
        .first()
        .and_then(|v| v.get("total"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    
    // Count total games
    let mut total_games_result = db
        .query("SELECT count() as total FROM game")
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    let total_games_vec: Vec<serde_json::Value> = total_games_result
        .take(0)
        .unwrap_or_default();
    
    let total_games = total_games_vec
        .first()
        .and_then(|v| v.get("total"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    
    // Count completed games
    let mut completed_games_result = db
        .query("SELECT count() as total FROM game WHERE status = 'completed'")
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    let completed_games_vec: Vec<serde_json::Value> = completed_games_result
        .take(0)
        .unwrap_or_default();
    
    let completed_games = completed_games_vec
        .first()
        .and_then(|v| v.get("total"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    
    // Get recent games for debugging
    let mut recent_games_result = db
        .query("SELECT id, status, started_at FROM game ORDER BY started_at DESC LIMIT 5")
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    let recent_games: Vec<serde_json::Value> = recent_games_result
        .take(0)
        .unwrap_or_default();
    
    Ok(Json(json!({
        "users": {
            "total": users_count
        },
        "games": {
            "total": total_games,
            "completed": completed_games,
            "recent": recent_games
        }
    })))
}