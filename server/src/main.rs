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
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::sync::Mutex;
use tower_http::cors::{CorsLayer, Any};
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
        // Auth routes
        .route("/auth/register", post(api::auth::register))
        .route("/auth/login", post(api::auth::login))
        .route("/auth/me", get(api::auth::me))
        // User routes
        .route("/users/:id", get(api::users::get_user_profile))
        .route("/users/profile", put(api::users::update_profile))
        .route("/users/profile/picture", post(api::users::upload_profile_picture))
        // Leaderboard routes
        .route("/leaderboard", get(api::leaderboard::get_leaderboard))
        .route("/leaderboard/top", get(api::leaderboard::get_top_players))
        // Admin routes
        .route("/admin/users", get(api::admin::list_users))
        .route("/admin/users/:id", put(api::admin::update_user))
        .route("/admin/users/:id", delete(api::admin::delete_user))
        .route("/admin/stats", get(api::admin::get_stats));

    let app = Router::new()
        .route("/ws/:room", get(handle_http))
        .nest("/api", api_routes)
        .nest_service("/uploads", ServeDir::new("uploads"))
        .layer(Extension(game_rooms.clone()))
        .layer(Extension(tx.clone()))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server is running at {:?}", listener.local_addr()?);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
