use crate::{
    api::auth::AppError,
    auth::AuthUser,
    db::get_db,
    models::{UpdateProfileRequest, User, UserProfile},
};
use axum::{
    extract::{Multipart, Path}, Json
};
use base64::{engine::general_purpose::STANDARD, Engine};
use image::{imageops::FilterType, DynamicImage};
use std::io::Cursor;
use surrealdb::{RecordId, Value};

pub async fn get_user_profile(Path(user_id): Path<String>) -> Result<Json<UserProfile>, AppError> {
    let user_id_clean = user_id.split(':').last().unwrap_or(&user_id).to_string();

    // Get full profile with stats using type-safe query
    let mut result = get_db()
        .query(r#"
            -- Define the user record type
            LET $uid = type::record('user', $user_id);

            -- Verify user exists
            IF NOT (SELECT * FROM $uid)[0] THEN
                THROW "User not found";
            END;

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
        .bind(("user_id", user_id_clean))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Use surrealdb::Value as intermediate type for RETURN statement
    let value: Value = result
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    // Convert Value to UserProfile
    let profile: UserProfile = serde_json::from_value(serde_json::to_value(value).map_err(|e| AppError::Database(e.to_string()))?)
        .map_err(|e| AppError::Database(format!("Failed to parse profile: {}", e)))?;

    Ok(Json(profile))
}

pub async fn update_profile(
    AuthUser(claims): AuthUser,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<UserProfile>, AppError> {
    let user_id = RecordId::from(("user", claims.user_id.as_str()));

    if let Some(username) = &req.username {
        let _: Option<User> = get_db()
            .update(user_id.clone())
            .merge(serde_json::json!({
                "username": username,
                "updated_at": chrono::Utc::now(),
            }))
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    let user_id_clean = claims.user_id.split(':').last().unwrap_or(&claims.user_id).to_string();

    // Get full profile with stats using type-safe query
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
        .bind(("user_id", user_id_clean))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Use surrealdb::Value as intermediate type for RETURN statement
    let value: Value = result
        .take(0)
        .map_err(|e| AppError::Database(e.to_string()))?;
    
    // Convert Value to UserProfile
    let profile: UserProfile = serde_json::from_value(serde_json::to_value(value).map_err(|e| AppError::Database(e.to_string()))?)
        .map_err(|e| AppError::Database(format!("Failed to parse profile: {}", e)))?;
    Ok(Json(profile))
}

pub async fn upload_profile_picture(
    AuthUser(claims): AuthUser,
    mut multipart: Multipart,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut image_data = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
    {
        if field.name() == Some("image") {
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;
            image_data = Some(data);
            break;
        }
    }

    let image_data = image_data.ok_or_else(|| AppError::Database("No image provided".to_string()))?;

    let img = image::load_from_memory(&image_data)
        .map_err(|e| AppError::Database(format!("Invalid image: {}", e)))?;

    let resized = resize_image(img, 200, 200);

    let mut buf = Vec::new();
    resized
        .write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Jpeg)
        .map_err(|e| AppError::Database(format!("Failed to encode image: {}", e)))?;

    let base64_image = format!("data:image/jpeg;base64,{}", STANDARD.encode(&buf));

    let user_id = RecordId::from(("user", claims.user_id.as_str()));

    let _: Option<User> = get_db()
        .update(user_id)
        .merge(serde_json::json!({
            "profile_picture": base64_image,
            "updated_at": chrono::Utc::now(),
        }))
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Profile picture updated successfully"
    })))
}

fn resize_image(img: DynamicImage, width: u32, height: u32) -> DynamicImage {
    img.resize(width, height, FilterType::Lanczos3)
}
