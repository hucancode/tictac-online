use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::RecordId;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: Option<RecordId>,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub profile_picture: Option<String>,
    pub elo: i32,
    pub games_played: i32,
    pub games_won: i32,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub email: String,
    pub username: String,
    pub profile_picture: Option<String>,
    pub elo: i32,
    pub games_played: i32,
    pub games_won: i32,
    pub win_rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserProfile,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub user_id: String,
    pub email: String,
    pub is_admin: bool,
    pub exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub rank: usize,
    pub user_id: String,
    pub username: String,
    pub elo: i32,
    pub games_played: i32,
    pub win_rate: f64,
    pub profile_picture: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameRecord {
    pub id: Option<RecordId>,
    pub player1: RecordId,
    pub player2: RecordId,
    pub winner: Option<RecordId>,
    pub board: Vec<Vec<Option<i32>>>,
    pub status: String,
    pub player1_elo_before: i32,
    pub player2_elo_before: i32,
    pub player1_elo_after: Option<i32>,
    pub player2_elo_after: Option<i32>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateProfileRequest {
    pub username: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminUpdateUserRequest {
    pub email: Option<String>,
    pub username: Option<String>,
    pub elo: Option<i32>,
    pub is_admin: Option<bool>,
}