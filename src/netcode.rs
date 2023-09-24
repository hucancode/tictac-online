use std::sync::Arc;
use tokio::sync::Mutex;
use axum::debug_handler;
use axum::response::IntoResponse;
use tokio::sync::broadcast::Sender;
use std::net::SocketAddr;
use axum::extract::{
    Path, 
    Extension, 
    WebSocketUpgrade,
    connect_info::ConnectInfo,
    ws::{WebSocket, Message},
};
use futures::stream::StreamExt;
use super::room::GameRoom;
use super::room::GameRooms;
use super::game::GameState;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Position {
    pub x: usize,
    pub y: usize,
}

async fn handle_move(
    mut socket: WebSocket, 
    who: SocketAddr,
    game_room: GameRoom, 
    tx: Sender<String>) {
    let mut player_id: Option<usize> = None;
    let message = {
        let mut game_room = game_room.lock().await;
        player_id = Some(game_room.join());
        serde_json::to_string(&game_room.board).unwrap()
    };
    let player_id = player_id.expect("Failed to join game");
    socket.send(Message::Text(message))
        .await
        .expect("WebSocket send error");
    while let Some(msg) = socket.next().await {
        match msg {
            Ok(msg) => {
                if let Message::Text(text) = msg {
                    if let Ok(pos) = serde_json::from_str::<Position>(&text) {
                        println!("making move at {:?}", pos);
                        let mut game_room = game_room.lock().await;
                        if let Err(_) = game_room.place(pos.x,pos.y,player_id) {
                            println!("cant make a move");
                            continue;
                        }
                        let message = serde_json::to_string(&game_room.board).unwrap();
                        let _ = tx.send(message.clone());
                        socket
                            .send(Message::Text(message))
                            .await
                            .expect("WebSocket send error");
                    } else {
                        println!("bad request {}", text);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error processing WebSocket message: {:?}", e);
            }
        }
    }
}

#[debug_handler]
pub async fn handle_connection(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Path(room_name): Path<String>,
    Extension(state): Extension<GameRooms>,
    Extension(tx): Extension<Sender<String>>,
) -> impl IntoResponse {
    let game_rooms = state.clone();
    let tx = tx.clone();
    let mut game_rooms = game_rooms.lock().await;
    let game_room = game_rooms
        .entry(room_name.clone())
        .or_insert_with(|| Arc::new(Mutex::new(GameState::new())))
        .clone();
    ws.on_upgrade(move |websocket| handle_move(websocket, addr, game_room, tx))
}


