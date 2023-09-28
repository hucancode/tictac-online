mod game;
mod netcode;
mod room;
use axum::routing::get;
use axum::Extension;
use axum::Router;
use netcode::handle_connection;
use room::GameRooms;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::Mutex;

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
    println!("Server is running at {:?}", addr);
    axum::Server::bind(&addr)
        .serve(app)
        .await
        .expect("Could not start server");
}
