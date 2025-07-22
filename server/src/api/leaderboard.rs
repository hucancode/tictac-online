use crate::{
    api::auth::AppError,
    db::get_db,
    models::{LeaderboardEntry, User},
};
use axum::{
    extract::Query,
    Json,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LeaderboardQuery {
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(default)]
    offset: usize,
}

fn default_limit() -> usize {
    20
}

pub async fn get_leaderboard(
    Query(params): Query<LeaderboardQuery>,
) -> Result<Json<Vec<LeaderboardEntry>>, AppError> {
    let mut result = get_db()
        .query(
            r#"
            SELECT * FROM user 
            WHERE games_played > 0 
            ORDER BY elo DESC 
            LIMIT $limit 
            START $offset
            "#,
        )
        .bind(("limit", params.limit))
        .bind(("offset", params.offset))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let users: Vec<User> = result
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;

    let leaderboard: Vec<LeaderboardEntry> = users
        .into_iter()
        .enumerate()
        .map(|(index, user)| {
            let win_rate = if user.games_played > 0 {
                (user.games_won as f64 / user.games_played as f64) * 100.0
            } else {
                0.0
            };

            LeaderboardEntry {
                rank: params.offset + index + 1,
                username: user.username,
                elo: user.elo,
                games_played: user.games_played,
                win_rate,
                profile_picture: user.profile_picture,
            }
        })
        .collect();

    Ok(Json(leaderboard))
}

pub async fn get_top_players() -> Result<Json<Vec<LeaderboardEntry>>, AppError> {
    get_leaderboard(Query(LeaderboardQuery {
        limit: 10,
        offset: 0,
    }))
    .await
}