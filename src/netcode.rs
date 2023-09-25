use super::game::GameState;
use super::room::GameRoom;
use super::room::GameRooms;
use axum::debug_handler;
use axum::extract::{
    ws::{Message, WebSocket},
    Extension, Path, WebSocketUpgrade,
};
use axum::response::IntoResponse;
use futures::stream::StreamExt;
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use tokio::sync::Mutex;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Position {
    pub x: usize,
    pub y: usize,
}

async fn handle_move(mut socket: WebSocket, game_room: GameRoom, tx: Sender<String>) {
    let player_id: Option<usize>;
    {
        let mut game_room = game_room.lock().await;
        player_id = Some(game_room.add_player());
        if let Ok(message) = serde_json::to_string(&game_room.board) {
            if socket.send(Message::Text(message)).await.is_err() {
                eprintln!("can't response to client");
                return;
            }
        }
    }
    let player_id = player_id.unwrap();

    while let Some(msg) = socket.next().await {
        match msg {
            Ok(msg) => {
                if let Message::Text(text) = msg {
                    if let Ok(pos) = serde_json::from_str::<Position>(&text) {
                        eprintln!("making move at {:?}", pos);
                        let mut game_room = game_room.lock().await;
                        if !game_room.place(pos.x, pos.y, player_id).is_ok() {
                            eprintln!("bad request {}", text);
                            continue;
                        }
                        if let Ok(message) = serde_json::to_string(&game_room.board) {
                            let _ = tx.send(message.clone());
                            if socket.send(Message::Text(message)).await.is_err() {
                                eprintln!("can't response to client");
                            }
                        }
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
    {
        let mut game_room = game_room.lock().await;
        game_room.remove_player(player_id);
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
