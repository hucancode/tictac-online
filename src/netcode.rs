use std::sync::Arc;
use tokio::sync::Mutex;
use axum::debug_handler;
use axum::extract::ws::{WebSocket, Message};
use axum::response::IntoResponse;
use tokio::sync::broadcast::Sender;
use axum::extract::{Path, Extension, WebSocketUpgrade};
use futures::stream::StreamExt;
use super::room::GameRoom;
use super::room::GameRooms;
use super::game::GameState;

async fn handle_move(mut socket: WebSocket, game_room: GameRoom, tx: Sender<String>) {
    let message = {
        let game_room = game_room.lock().await;
        serde_json::to_string(&game_room.board).unwrap()
    };
    socket.send(Message::Text(message))
        .await
        .expect("WebSocket send error");
    while let Some(msg) = socket.next().await {
        match msg {
            Ok(msg) => {
                if let Message::Text(text) = msg {
                    if let Ok(position) = text.parse::<usize>() {
                        let mut game_room = game_room.lock().await;
                        if let Err(_) = game_room.make_move(position, 'X') {
                            continue;
                        }
                        let message = serde_json::to_string(&game_room.board).unwrap();
                        let _ = tx.send(message.clone());
                        socket
                            .send(Message::Text(message))
                            .await
                            .expect("WebSocket send error");
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
    ws.on_upgrade(move |websocket| handle_move(websocket, game_room, tx))
}


