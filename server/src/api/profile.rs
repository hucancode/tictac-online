use crate::{
    api::auth::AppError,
    db::get_db,
    models::UserProfile,
};
use surrealdb::Value;

/// Get user profile with game statistics using a single SurrealQL query
/// All logic is done in SurrealQL, returning only the final typed result
pub async fn get_user_profile_with_stats(user_id: &str) -> Result<UserProfile, AppError> {
    let mut result = get_db()
        .query(r#"
            -- Define the user record type
            LET $uid = type::record('user', $user_id);
            
            -- Get user data
            LET $user = (SELECT * FROM $uid)[0];
            
            -- Count games efficiently
            LET $games = SELECT 
                count() as total,
                count(winner = $uid) as won
            FROM game 
            WHERE (player1 = $uid OR player2 = $uid) 
            AND status = 'completed';
            
            -- Return strongly typed profile
            RETURN {
                id: string::concat('user:', $user_id),
                email: $user.email,
                username: $user.username,
                profile_picture: $user.profile_picture,
                elo: $user.elo,
                games_played: $games[0].total OR 0,
                games_won: $games[0].won OR 0,
                win_rate: IF $games[0].total > 0 THEN 
                    math::round($games[0].won * 100.0 / $games[0].total, 2) 
                ELSE 0.0 END
            };
        "#)
        .bind(("user_id", user_id))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Use surrealdb::Value as intermediate type for RETURN statement
    let value: Value = result
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    // Convert Value to UserProfile
    let profile: UserProfile = serde_json::from_value(serde_json::to_value(value).map_err(|e| AppError::Database(e.to_string()))?)
        .map_err(|e| AppError::Database(format!("Failed to parse profile: {}", e)))?;

    Ok(profile)
}