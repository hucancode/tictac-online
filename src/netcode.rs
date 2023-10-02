use super::game::GameState;
use super::game::MoveResult;
use super::protocol::ClientMessage;
use super::protocol::ServerMessage;
use super::room::GameRoom;
use super::room::GameRooms;
use axum::debug_handler;
use axum::extract::{
    ws::{Message, WebSocket},
    Extension, Path, WebSocketUpgrade,
};
use axum::response::IntoResponse;
use futures::{
    sink::SinkExt,
    stream::{SplitSink, SplitStream, StreamExt},
};
use std::sync::Arc;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

const DEFAULT_PLAYER_NAME: &str = "someone";
async fn enter_room(
    game_room: GameRoom,
    sender: &mut SplitSink<WebSocket, Message>,
) -> Option<usize> {
    let mut game_room = game_room.lock().await;
    let id = game_room.add_player();
    let message = String::from(ServerMessage::JoinedRoom { your_id: id });
    if !message.is_empty() && sender.send(Message::Text(message.clone())).await.is_err() {
        eprintln!("can't response to client with {}", message);
        return None;
    }
    return Some(id);
}

fn handle_send(
    mut sender: SplitSink<WebSocket, Message>,
    mut rx: Receiver<String>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.clone())).await.is_err() {
                eprintln!("can't response to client with {}", msg);
            }
        }
    })
}

fn handle_receive(
    mut receiver: SplitStream<WebSocket>,
    tx: Sender<String>,
    game_room: GameRoom,
    player_id: usize,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let mut player_name = DEFAULT_PLAYER_NAME.to_string();
        while let Some(msg) = receiver.next().await {
            if let Err(e) = msg {
                eprintln!("Error receiving WebSocket message: {:?}", e);
                continue;
            }
            if let Message::Text(text) = msg.unwrap() {
                let message = ClientMessage::from(text);
                println!("Server received: {:?}", message);
                match message {
                    ClientMessage::Place { x, y } => {
                        let mut game_room = game_room.lock().await;
                        match game_room.place(x, y, player_id) {
                            MoveResult::Ok => {
                                if let Err(e) =
                                    tx.send(String::from(ServerMessage::from(game_room.clone())))
                                {
                                    eprintln!("Server error while sending message: {}", e);
                                }
                            }
                            MoveResult::Win => {
                                if let Err(e) =
                                    tx.send(String::from(ServerMessage::from(game_room.clone())))
                                {
                                    eprintln!("Server error while sending message: {}", e);
                                }
                                if let Err(e) = tx.send(String::from(ServerMessage::GameEnd {
                                    winner_x: x,
                                    winner_y: y,
                                })) {
                                    eprintln!("Server error while sending message: {}", e);
                                }
                            }
                            _ => {}
                        }
                    }
                    ClientMessage::ReadyVote { accept } => {
                        let mut game_room = game_room.lock().await;
                        if game_room.ready_vote(player_id, accept) {
                            if let Err(e) = tx.send(String::from(ServerMessage::GameStarted {
                                players: game_room.get_acting_players()
                            })) {
                                eprintln!("Server error while sending message: {}", e);
                            }
                            if let Err(e) =
                                tx.send(String::from(ServerMessage::from(game_room.clone())))
                            {
                                eprintln!("Server error while sending message: {}", e);
                            }
                        }
                    }
                    ClientMessage::Chat { content } => {
                        if let Err(e) = tx.send(String::from(ServerMessage::Chat {
                            who: player_name.clone(),
                            content,
                        })) {
                            eprintln!("Server error while sending chat message: {}", e);
                        }
                    }
                    ClientMessage::Register { name } => {
                        player_name = name;
                        println!("player {} registered with name {}", player_id, player_name);
                    }
                    _ => {}
                };
            }
        }
        let _ = tx.send(String::from(ServerMessage::Chat {
            who: "system: ".to_string(),
            content: format!("{} has left the room", player_id),
        }));
    })
}

async fn handle_ws(socket: WebSocket, game_room: GameRoom, tx: Sender<String>) {
    let (mut sender, receiver) = socket.split();
    let rx = tx.subscribe();
    let player_id = enter_room(game_room.clone(), &mut sender).await;
    if player_id.is_none() {
        return;
    }
    let player_id = player_id.unwrap();
    let mut send_task = handle_send(sender, rx);
    let mut recv_task = handle_receive(receiver, tx, game_room.clone(), player_id);
    // If any one of the tasks run to completion, we abort the other.
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
    let mut game_room = game_room.lock().await;
    game_room.remove_player(player_id);
}

#[debug_handler]
pub async fn handle_http(
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
    ws.on_upgrade(move |ws| handle_ws(ws, game_room, tx))
}
