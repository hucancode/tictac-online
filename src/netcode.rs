use super::game::GameState;
use super::room::GameRoom;
use super::room::GameRooms;
use axum::debug_handler;
use axum::extract::{
    ws::{Message, WebSocket},
    Extension, Path, WebSocketUpgrade,
};
use axum::response::IntoResponse;
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
struct Position {
    pub x: usize,
    pub y: usize,
}

async fn handle_move(socket: WebSocket, game_room: GameRoom, tx: Sender<String>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = tx.subscribe();
    let player_id: usize;
    {
        let mut game_room = game_room.lock().await;
        player_id = game_room.add_player();
        if let Ok(message) = serde_json::to_string(&game_room.board) {
            if sender.send(Message::Text(message)).await.is_err() {
                eprintln!("can't response to client");
            }
        }
    }

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                eprintln!("can't response to client");
            }
        }
    });
    let game_room_a = game_room.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if let Err(e) = msg {
                eprintln!("Error receiving WebSocket message: {:?}", e);
                continue;
            }
            if let Message::Text(text) = msg.unwrap() {
                let res = serde_json::from_str::<Position>(&text);
                if let Err(e) = res {
                    eprintln!("bad request with {}, {}", text, e);
                    continue;
                }
                let pos = res.unwrap();
                eprintln!("making move at {:?}", pos);
                let mut game_room = game_room_a.lock().await;
                if game_room.place(pos.x, pos.y, player_id).is_err() {
                    eprintln!("bad request {}", text);
                    continue;
                }
                let res = serde_json::to_string(&game_room.board);
                if let Err(e) = res {
                    eprintln!("server error, can't serialize board! {}", e);
                    continue;
                }
                let message = res.unwrap();
                let _ = tx.send(message.clone());
            }
        }
    });
    // If any one of the tasks run to completion, we abort the other.
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
    println!("{} has left the room", player_id);
    // tx.send("player_id has left the room");
    let mut game_room = game_room.lock().await;
    game_room.remove_player(player_id);
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
