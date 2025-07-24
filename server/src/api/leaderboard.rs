use crate::{
    api::auth::AppError,
    db::get_db,
    models::LeaderboardEntry,
};
use axum::{
    extract::Query,
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LeaderboardQuery {
    #[serde(default = "default_page")]
    page: usize,
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_page() -> usize {
    1
}

fn default_limit() -> usize {
    20
}

#[derive(Serialize)]
pub struct LeaderboardResponse {
    pub entries: Vec<LeaderboardEntry>,
    pub total: usize,
    pub page: usize,
    pub limit: usize,
    pub total_pages: usize,
}

pub async fn get_leaderboard(
    Query(params): Query<LeaderboardQuery>,
) -> Result<Json<LeaderboardResponse>, AppError> {
    // Ensure page is at least 1
    let page = params.page.max(1);
    // Limit per page between 1 and 100
    let limit = params.limit.clamp(1, 100);
    let offset = (page - 1) * limit;
    
    // First, get all eligible players (those who have completed games)
    let mut count_result = get_db()
        .query(
            r#"
            SELECT DISTINCT player FROM (
                SELECT player1 as player FROM game WHERE status = 'completed'
                UNION
                SELECT player2 as player FROM game WHERE status = 'completed'
            )
            "#,
        )
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    let count_data: Vec<serde_json::Value> = count_result
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    let total = count_data.len();
    
    // Calculate total pages
    let total_pages = if total > 0 {
        (total + limit - 1) / limit
    } else {
        0
    };
    
    // Don't fetch if we're beyond the available pages
    if page > total_pages && total_pages > 0 {
        return Ok(Json(LeaderboardResponse {
            entries: vec![],
            total,
            page,
            limit,
            total_pages,
        }));
    }
    
    // Get users with game stats
    let mut result = get_db()
        .query(
            r#"
            SELECT 
                u.*,
                (SELECT count() FROM game WHERE (player1 = u.id OR player2 = u.id) AND status = 'completed') as games_count,
                (SELECT count() FROM game WHERE winner = u.id AND status = 'completed') as wins_count
            FROM user u
            WHERE u.id IN (
                SELECT DISTINCT player FROM (
                    SELECT player1 as player FROM game WHERE status = 'completed'
                    UNION
                    SELECT player2 as player FROM game WHERE status = 'completed'
                )
            )
            ORDER BY u.elo DESC 
            LIMIT $limit 
            START $offset
            "#,
        )
        .bind(("limit", limit))
        .bind(("offset", offset))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let data: Vec<serde_json::Value> = result
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;

    let entries: Vec<LeaderboardEntry> = data
        .into_iter()
        .enumerate()
        .filter_map(|(index, value)| {
            let games_played = value.get("games_count")?.as_i64()? as i32;
            let games_won = value.get("wins_count")?.as_i64()? as i32;
            
            let win_rate = if games_played > 0 {
                (games_won as f64 / games_played as f64) * 100.0
            } else {
                0.0
            };

            Some(LeaderboardEntry {
                rank: offset + index + 1,
                user_id: value.get("id")?.as_str()?.to_string(),
                username: value.get("username")?.as_str()?.to_string(),
                elo: value.get("elo")?.as_i64()? as i32,
                games_played,
                win_rate,
                profile_picture: value.get("profile_picture").and_then(|v| v.as_str()).map(String::from),
            })
        })
        .collect();

    Ok(Json(LeaderboardResponse {
        entries,
        total,
        page,
        limit,
        total_pages,
    }))
}

pub async fn get_top_players() -> Result<Json<Vec<LeaderboardEntry>>, AppError> {
    // For backward compatibility, return just the entries array
    let response = get_leaderboard(Query(LeaderboardQuery {
        page: 1,
        limit: 10,
    }))
    .await?;
    
    Ok(Json(response.0.entries))
}

// New endpoint to get a specific player's rank
pub async fn get_player_rank(
    axum::extract::Path(user_id): axum::extract::Path<String>
) -> Result<Json<serde_json::Value>, AppError> {
    let mut result = get_db()
        .query(
            r#"
            SELECT rank FROM (
                SELECT id, 
                       row_number() OVER (ORDER BY elo DESC) as rank
                FROM user 
                WHERE id IN (
                    SELECT DISTINCT player FROM (
                        SELECT player1 as player FROM game WHERE status = 'completed'
                        UNION
                        SELECT player2 as player FROM game WHERE status = 'completed'
                    )
                )
                ORDER BY elo DESC
                LIMIT 1000
            ) WHERE id = type::thing('user', $user_id)
            "#,
        )
        .bind(("user_id", user_id))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    let rank_data: Vec<serde_json::Value> = result
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    if let Some(data) = rank_data.first() {
        Ok(Json(data.clone()))
    } else {
        Ok(Json(serde_json::json!({ "rank": null, "message": "Player not in top 1000" })))
    }
}