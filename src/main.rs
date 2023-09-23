mod game;
mod room;
mod netcode;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use netcode::handle_connection;
use tokio::sync::broadcast;
use axum::routing::get;
use std::net::SocketAddr;
use axum::Extension;
use axum::Router;
use room::GameRooms;

#[tokio::main]
async fn main() {
    let game_rooms: GameRooms = Arc::new(Mutex::new(HashMap::new()));
    let (tx, _) = broadcast::channel::<String>(1024);

    let app = Router::new()
        .route("/ws/:room", get(handle_connection))
        .layer(Extension(game_rooms.clone()))
        .layer(Extension(tx.clone()));

    let addr = ([127, 0, 0, 1], 8080).into();
    let app = app.into_make_service_with_connect_info::<SocketAddr>(); 
    axum::Server::bind(&addr)
        .serve(app)
        .await
        .expect("Could not start server");
}