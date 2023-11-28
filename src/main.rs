mod game;
mod netcode;
mod protocol;
mod room;
use axum::routing::get;
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let game_rooms: GameRooms = Arc::new(Mutex::new(HashMap::new()));
    let (tx, _) = broadcast::channel::<String>(1024);

    let app = Router::new()
        .route("/ws/:room", get(handle_http))
        .layer(Extension(game_rooms.clone()))
        .layer(Extension(tx.clone()));

    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server is running at {:?}", listener.local_addr()?);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
