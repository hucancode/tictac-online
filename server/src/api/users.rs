use crate::{
    api::auth::AppError,
    auth::AuthUser,
    db::get_db,
    models::{UpdateProfileRequest, User, UserProfile},
};
use axum::{
    extract::{Multipart, Path},
    Json,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use image::{imageops::FilterType, DynamicImage};
use std::io::Cursor;
use surrealdb::RecordId;

pub async fn get_user_profile(Path(user_id): Path<String>) -> Result<Json<UserProfile>, AppError> {
    let thing = RecordId::from(("user", user_id.as_str()));
    
    let user: Option<User> = get_db()
        .select(thing)
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