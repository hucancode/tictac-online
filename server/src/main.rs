mod game;
mod netcode;
mod protocol;
mod room;
mod db;
mod models;
mod auth;
mod elo;
mod api;
mod game_db;
use axum::routing::{get, post, put, delete};
use axum::Extension;
use axum::Router;
use netcode::handle_http;
use room::GameRooms;
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use std::env;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use tower_http::cors::{CorsLayer, AllowOrigin};
use http::header::{AUTHORIZATION, CONTENT_TYPE};
use http::{HeaderValue, Method};
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize database
    match db::init_db().await {
        Ok(_) => println!("Database initialized"),
        Err(e) => {
            eprintln!("Failed to initialize database: {}", e);
            return Err(e);
        }
    }

    let game_rooms: GameRooms = Arc::new(Mutex::new(HashMap::new()));
    let (tx, _) = broadcast::channel::<String>(1024);

    // API routes
    let api_routes = Router::new()
        // Health check
        .route("/health", get(|| async { "OK" }))
        // Auth routes
        .route("/auth/register", post(api::auth::register))
        .route("/auth/login", post(api::auth::login))
        .route("/auth/me", get(api::auth::me))
        // User routes
        .route("/users/{id}", get(api::users::get_user_profile))
        .route("/users/profile", put(api::users::update_profile))
        .route("/users/profile/picture", post(api::users::upload_profile_picture))
        // Leaderboard routes
        .route("/leaderboard", get(api::leaderboard::get_leaderboard))
        .route("/leaderboard/top", get(api::leaderboard::get_top_players))
        .route("/leaderboard/rank/{id}", get(api::leaderboard::get_player_rank))
        // Game routes
        .route("/games/history", get(api::games::get_match_history))
        .route("/games/{id}", get(api::games::get_game_details))
        // Admin routes
        .route("/admin/users", get(api::admin::list_users))
        .route("/admin/users/{id}", put(api::admin::update_user))
        .route("/admin/users/{id}", delete(api::admin::delete_user))
        .route("/admin/stats", get(api::admin::get_stats))
        // Debug routes
        .route("/debug/db", get(api::debug::get_database_info));

    let app = Router::new()
        .route("/ws/{room}", get(handle_http))
        .nest("/api", api_routes)
        .nest_service("/uploads", ServeDir::new("uploads"))
        .layer(Extension(game_rooms.clone()))
        .layer(Extension(tx.clone()))
        .layer({
            let mut cors_origins = Vec::new();
            
            // Get allowed origins from environment variable
            // Default to common development URLs if not specified
            let origins_str = env::var("CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000,http://localhost:5173,http://localhost:30030".to_string());
            
            for origin in origins_str.split(',') {
                let origin = origin.trim();
                if !origin.is_empty() {
                    if let Ok(header_value) = origin.parse::<HeaderValue>() {
                        cors_origins.push(header_value);
                        println!("Added CORS origin: {}", origin);
                    } else {
                        eprintln!("Invalid CORS origin: {}", origin);
                    }
                }
            }
            
            if cors_origins.is_empty() {
                panic!("No valid CORS origins configured!");
            }
            
            CorsLayer::new()
                .allow_origin(AllowOrigin::list(cors_origins))
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
                .allow_headers([AUTHORIZATION, CONTENT_TYPE])
                .allow_credentials(true)
        });

    let host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("{}:{}", host, port);
    
    let listener = TcpListener::bind(&addr).await?;
    println!("Server is running at {}", addr);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
