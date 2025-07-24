use crate::{
    auth::AuthUser,
    db::get_db,
    models::{GameRecord, User},
};
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use surrealdb::RecordId;

#[derive(Debug, Deserialize)]
pub struct MatchHistoryQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct MatchHistoryItem {
    pub id: String,
    pub opponent_id: String,
    pub opponent_name: String,
    pub opponent_elo: i32,
    pub result: String, // "win", "loss", "draw"
    pub my_elo_before: i32,
    pub my_elo_after: i32,
    pub opponent_elo_before: i32,
    pub opponent_elo_after: i32,
    pub elo_change: i32,
    pub created_at: String,
    pub ended_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct MatchHistoryResponse {
    pub matches: Vec<MatchHistoryItem>,
    pub total: usize,
    pub page: u32,
    pub limit: u32,
}

pub async fn get_match_history(
    auth: AuthUser,
    Query(query): Query<MatchHistoryQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let db = get_db();

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = ((page - 1) * limit) as usize;

    // Get the user's record ID
    let user_id = auth.0.user_id;
    let user_thing = RecordId::from(("user", user_id.as_str()));

    // Query games where the user is either player1 or player2
    let sql = r#"
        SELECT * FROM game 
        WHERE (player1 = $user OR player2 = $user) 
        AND status = 'completed'
        ORDER BY ended_at DESC
        LIMIT $limit
        START $offset
    "#;

    let mut result = db
        .query(sql)
        .bind(("user", user_thing.clone()))
        .bind(("limit", limit as i64))
        .bind(("offset", offset as i64))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to fetch games: {}", e)})),
            )
        })?;

    // Parse as JSON values first to handle datetime issues
    let games_json: Vec<serde_json::Value> = result.take(0).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to parse games: {}", e)})),
        )
    })?;

    // Count total games for pagination
    let count_sql = r#"
        SELECT count() AS total FROM game 
        WHERE (player1 = $user OR player2 = $user) 
        AND status = 'completed'
    "#;

    let mut count_result = db
        .query(count_sql)
        .bind(("user", user_thing.clone()))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to count games: {}", e)})),
            )
        })?;

    let count_data: Option<serde_json::Value> = count_result.take(0).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to parse count: {}", e)})),
        )
    })?;

    let total = count_data
        .and_then(|v| v.get("total").cloned())
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    // Fetch opponent details for all games
    let mut match_history = Vec::new();
    
    for game_json in games_json {
        // Extract game data from JSON
        let game_id = game_json.get("id")
            .and_then(|id| {
                if let Some(id_str) = id.as_str() {
                    Some(id_str.to_string())
                } else if let Some(id_obj) = id.as_object() {
                    id_obj.get("id").and_then(|i| i.as_str()).map(|s| s.to_string())
                } else {
                    None
                }
            })
            .unwrap_or_default();

        // Extract player IDs
        let player1_id = game_json.get("player1").and_then(|p| p.as_str()).unwrap_or("");
        let player2_id = game_json.get("player2").and_then(|p| p.as_str()).unwrap_or("");
        
        let is_player1 = player1_id.contains(&user_id) || player1_id == format!("user:{}", user_id);
        let opponent_id_str = if is_player1 { player2_id } else { player1_id };
        
        // Convert string ID to RecordId for fetching
        let opponent_id = if opponent_id_str.starts_with("user:") {
            RecordId::from(("user", &opponent_id_str[5..]))
        } else {
            RecordId::from(("user", opponent_id_str))
        };
        
        // Fetch opponent user data
        let opponent: Option<User> = db
            .select(&opponent_id)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({"error": format!("Failed to fetch opponent: {}", e)})),
                    )
                })?;

            if let Some(opponent_user) = opponent {
                // Extract ELO values
                let player1_elo_before = game_json.get("player1_elo_before")
                    .and_then(|e| e.as_i64())
                    .unwrap_or(1200) as i32;
                let player2_elo_before = game_json.get("player2_elo_before")
                    .and_then(|e| e.as_i64())
                    .unwrap_or(1200) as i32;
                let player1_elo_after = game_json.get("player1_elo_after")
                    .and_then(|e| e.as_i64())
                    .map(|e| e as i32)
                    .unwrap_or(player1_elo_before);
                let player2_elo_after = game_json.get("player2_elo_after")
                    .and_then(|e| e.as_i64())
                    .map(|e| e as i32)
                    .unwrap_or(player2_elo_before);

                let (my_elo_before, my_elo_after, opponent_elo_before, opponent_elo_after) = 
                    if is_player1 {
                        (player1_elo_before, player1_elo_after, player2_elo_before, player2_elo_after)
                    } else {
                        (player2_elo_before, player2_elo_after, player1_elo_before, player1_elo_after)
                    };

                // Determine result
                let winner_id = game_json.get("winner").and_then(|w| w.as_str()).unwrap_or("");
                let result = if winner_id.is_empty() {
                    "draw"
                } else if winner_id.contains(&user_id) {
                    "win"
                } else {
                    "loss"
                };

                // Extract datetime strings
                let created_at = game_json.get("started_at")
                    .and_then(|dt| dt.as_str())
                    .unwrap_or("")
                    .to_string();
                let ended_at = game_json.get("ended_at")
                    .and_then(|dt| dt.as_str())
                    .map(|s| s.to_string());

                match_history.push(MatchHistoryItem {
                    id: game_id,
                    opponent_id: opponent_id_str.to_string(),
                    opponent_name: opponent_user.username.clone(),
                    opponent_elo: opponent_user.elo,
                    result: result.to_string(),
                    my_elo_before,
                    my_elo_after,
                    opponent_elo_before,
                    opponent_elo_after,
                    elo_change: my_elo_after - my_elo_before,
                    created_at,
                    ended_at,
                });
            }
    }

    let response = MatchHistoryResponse {
        matches: match_history,
        total,
        page,
        limit,
    };

    Ok(Json(response))
}

pub async fn get_game_details(
    _auth: AuthUser,
    Path(game_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let db = get_db();

    let game: Option<GameRecord> = db
        .select(("game", game_id.as_str()))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to fetch game: {}", e)})),
            )
        })?;

    let game = game.ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Game not found"})),
        )
    })?;

    // Fetch player details - game.player1 and player2 are now RecordIds
    let player1_id = game.player1.clone();
    let player2_id = game.player2.clone();
    
    let player1: Option<User> = db.select(player1_id).await.ok().flatten();
    let player2: Option<User> = db.select(player2_id).await.ok().flatten();

    let response = json!({
        "id": game.id.as_ref().map(|id| id.to_string()).unwrap_or_default(),
        "status": game.status,
        "board": game.board,
        "player1": player1.map(|p| json!({
            "id": p.id.as_ref().unwrap().to_string(),
            "username": p.username,
            "elo_before": game.player1_elo_before,
            "elo_after": game.player1_elo_after,
        })),
        "player2": player2.map(|p| json!({
            "id": p.id.as_ref().unwrap().to_string(),
            "username": p.username,
            "elo_before": game.player2_elo_before,
            "elo_after": game.player2_elo_after,
        })),
        "winner": game.winner,
        "started_at": game.started_at.to_rfc3339(),
        "ended_at": game.ended_at.map(|dt| dt.to_rfc3339()),
    });

    Ok(Json(response))
}