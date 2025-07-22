use super::game::GameState;
use super::game::MoveResult;
use super::protocol::ClientMessage;
use super::protocol::ServerMessage;
use super::room::GameRoom;
use super::room::GameRooms;
use crate::auth::{DECODING_KEY};
use crate::models::Claims;
use crate::game_db;
use axum::debug_handler;
use axum::extract::{
    ws::{Message, WebSocket},
    Extension, Path, WebSocketUpgrade,
};
use axum::response::IntoResponse;
use axum::http::StatusCode;
use jsonwebtoken::{decode, Validation};
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
    player: String,
    sender: &mut SplitSink<WebSocket, Message>,
    tx: &Sender<String>,
) -> Option<usize> {
    let mut game_room = game_room.lock().await;
    let id = game_room.add_member(player);
    let is_room_creator = game_room.is_room_creator(id);
    
    let message = String::from(ServerMessage::JoinedRoom { 
        your_id: id,
        is_room_creator,
        room_creator: game_room.room_creator.clone().unwrap_or_default(),
        members: game_room.members.clone(),
        player_queue: game_room.player_queue.clone(),
    });
    
    if !message.is_empty() && sender.send(Message::Text(message.clone())).await.is_err() {
        eprintln!("can't response to client with {}", message);
        None
    } else {
        // Broadcast room state update to others
        let _ = tx.send(String::from(ServerMessage::RoomStateUpdate {
            members: game_room.members.clone(),
            player_queue: game_room.player_queue.clone(),
            room_creator: game_room.room_creator.clone().unwrap_or_default(),
        }));
        Some(id)
    }
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
                                let winner_name = game_room.members[player_id].clone();
                                
                                // Get winner's email from active players
                                let winner_email = if player_id < game_room.active_players.len() {
                                    game_room.active_players[player_id].clone()
                                } else {
                                    winner_name.clone() // Fallback, though this shouldn't happen
                                };
                                
                                // Update game in database
                                if let Some(game_id) = &game_room.game_id {
                                    // Convert board to database format
                                    let board: Vec<Vec<Option<i32>>> = game_room.board
                                        .iter()
                                        .map(|row| row.iter().map(|&cell| cell.map(|p| p as i32)).collect())
                                        .collect();
                                    
                                    if let Err(e) = game_db::update_game_board(game_id, board).await {
                                        eprintln!("Failed to update game board: {}", e);
                                    }
                                    
                                    if let Err(e) = game_db::end_game(game_id, Some(&winner_email)).await {
                                        eprintln!("Failed to end game in database: {}", e);
                                    }
                                }
                                
                                if let Err(e) = tx.send(String::from(ServerMessage::GameEnd {
                                    winner: winner_email.clone(),
                                    winner_x: x,
                                    winner_y: y,
                                })) {
                                    eprintln!("Server error while sending message: {}", e);
                                }
                                // Move back to preparation phase
                                game_room.phase = super::game::GamePhase::Ready;
                                game_room.active_players.clear();
                                game_room.game_id = None;
                                
                                // Send room state update
                                let _ = tx.send(String::from(ServerMessage::RoomStateUpdate {
                                    members: game_room.members.clone(),
                                    player_queue: game_room.player_queue.clone(),
                                    room_creator: game_room.room_creator.clone().unwrap_or_default(),
                                }));
                            }
                            _ => {}
                        }
                    }
                    ClientMessage::StepUp => {
                        let mut game_room = game_room.lock().await;
                        if game_room.step_up(player_id) {
                            let _ = tx.send(String::from(ServerMessage::RoomStateUpdate {
                                members: game_room.members.clone(),
                                player_queue: game_room.player_queue.clone(),
                                room_creator: game_room.room_creator.clone().unwrap_or_default(),
                            }));
                        }
                    }
                    ClientMessage::StepDown => {
                        let mut game_room = game_room.lock().await;
                        if game_room.step_down(player_id) {
                            let _ = tx.send(String::from(ServerMessage::RoomStateUpdate {
                                members: game_room.members.clone(),
                                player_queue: game_room.player_queue.clone(),
                                room_creator: game_room.room_creator.clone().unwrap_or_default(),
                            }));
                        }
                    }
                    ClientMessage::StartGame => {
                        let mut game_room = game_room.lock().await;
                        if game_room.start_game(player_id) {
                            // Create game in database
                            if game_room.active_players.len() == 2 {
                                match game_db::create_game(&game_room.active_players[0], &game_room.active_players[1]).await {
                                    Ok(game_id) => {
                                        game_room.game_id = Some(game_id);
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to create game in database: {}", e);
                                    }
                                }
                            }
                            
                            if let Err(e) = tx.send(String::from(ServerMessage::GameStarted {
                                players: game_room.active_players.clone(),
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
                    ClientMessage::KickMember { member_id } => {
                        let mut game_room = game_room.lock().await;
                        if game_room.is_room_creator(player_id) && member_id < game_room.members.len() {
                            let kicked_member = game_room.members[member_id].clone();
                            game_room.remove_member(kicked_member.clone());
                            let _ = tx.send(String::from(ServerMessage::Chat {
                                who: "system".to_string(),
                                content: format!("{} was kicked from the room", kicked_member),
                            }));
                            let _ = tx.send(String::from(ServerMessage::RoomStateUpdate {
                                members: game_room.members.clone(),
                                player_queue: game_room.player_queue.clone(),
                                room_creator: game_room.room_creator.clone().unwrap_or_default(),
                            }));
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

async fn handle_ws(socket: WebSocket, player: String, game_room: GameRoom, tx: Sender<String>) {
    let (mut sender, receiver) = socket.split();
    let player_id = match enter_room(game_room.clone(), player.clone(), &mut sender, &tx).await {
        Some(id) => id,
        None => return,
    };
    let rx = tx.subscribe();
    let mut send_task = handle_send(sender, rx);
    let mut recv_task = handle_receive(receiver, tx.clone(), game_room.clone(), player_id);
    // If any one of the tasks run to completion, we abort the other.
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
    
    // Handle disconnection
    let mut game_room = game_room.lock().await;
    let member_name = game_room.members.get(player_id).cloned().unwrap_or_default();
    
    // Check if disconnected player was in an active game
    if game_room.is_active_player(player_id) && matches!(game_room.phase, super::game::GamePhase::Action) {
        // Find the other player's email
        let disconnected_email = game_room.active_players.get(player_id).cloned();
        let winner_email = game_room.active_players.iter()
            .find(|&p| disconnected_email.as_ref().map_or(true, |de| p != de))
            .cloned();
        
        if let Some(winner) = winner_email {
            // Update game in database
            if let Some(game_id) = &game_room.game_id {
                // Convert board to database format
                let board: Vec<Vec<Option<i32>>> = game_room.board
                    .iter()
                    .map(|row| row.iter().map(|&cell| cell.map(|p| p as i32)).collect())
                    .collect();
                
                if let Err(e) = game_db::update_game_board(game_id, board).await {
                    eprintln!("Failed to update game board: {}", e);
                }
                
                if let Err(e) = game_db::end_game(game_id, Some(&winner)).await {
                    eprintln!("Failed to end game in database: {}", e);
                }
            }
            
            let _ = tx.send(String::from(ServerMessage::GameEnd {
                winner: winner.clone(),
                winner_x: 0,
                winner_y: 0,
            }));
            let _ = tx.send(String::from(ServerMessage::Chat {
                who: "system".to_string(),
                content: format!("{} wins by default - opponent disconnected", winner),
            }));
            game_room.phase = super::game::GamePhase::Ready;
            game_room.active_players.clear();
            game_room.game_id = None;
            
            // Send room state update after game ends by disconnect
            let _ = tx.send(String::from(ServerMessage::RoomStateUpdate {
                members: game_room.members.clone(),
                player_queue: game_room.player_queue.clone(),
                room_creator: game_room.room_creator.clone().unwrap_or_default(),
            }));
        }
    }
    
    game_room.remove_member(player);
    
    // Broadcast updated room state after member removal
    let _ = tx.send(String::from(ServerMessage::RoomStateUpdate {
        members: game_room.members.clone(),
        player_queue: game_room.player_queue.clone(),
        room_creator: game_room.room_creator.clone().unwrap_or_default(),
    }));
    
    let _ = tx.send(String::from(ServerMessage::Chat {
        who: "system".to_string(),
        content: format!("{} has left the room", member_name),
    }));
}

#[derive(serde::Deserialize)]
pub struct EnterRoomRequest {
    user: Option<String>,
    token: Option<String>,
}
#[debug_handler]
pub async fn handle_http(
    ws: WebSocketUpgrade,
    Path(room_name): Path<String>,
    Extension(state): Extension<GameRooms>,
    Extension(tx): Extension<Sender<String>>,
    axum::extract::Query(params): axum::extract::Query<EnterRoomRequest>,
) -> impl IntoResponse {
    // Verify JWT token if provided
    let user = if let Some(token) = params.token {
        match decode::<Claims>(&token, &DECODING_KEY, &Validation::default()) {
            Ok(token_data) => token_data.claims.email,
            Err(_) => return StatusCode::UNAUTHORIZED.into_response(),
        }
    } else if let Some(user) = params.user {
        user
    } else {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        format!("Guest_{}", timestamp % 10000)
    };
    
    let game_rooms = state.clone();
    let tx = tx.clone();
    let mut game_rooms = game_rooms.lock().await;
    let game_room = game_rooms
        .entry(room_name.clone())
        .or_insert_with(|| Arc::new(Mutex::new(GameState::new())))
        .clone();
    
    ws.on_upgrade(move |ws| handle_ws(ws, user, game_room, tx))
}
