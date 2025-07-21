use super::game::Board;
use super::game::GameState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    JoinedRoom { 
        your_id: usize, 
        is_room_creator: bool,
        room_creator: String,
        members: Vec<String>,
        player_queue: Vec<String>,
    },
    GameStarted { players: Vec<String> },
    GameState { board: Box<Board>, turn: usize },
    GameEnd { winner: String, winner_x: usize, winner_y: usize },
    RoomStateUpdate {
        members: Vec<String>,
        player_queue: Vec<String>,
        room_creator: String,
    },
    Chat { who: String, content: String },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    StepUp,
    StepDown,
    StartGame,
    Place { x: usize, y: usize },
    Chat { content: String },
    KickMember { member_id: usize },
    Register { name: String },
    Unknown,
}

impl From<ServerMessage> for String {
    fn from(input: ServerMessage) -> Self {
        let res = serde_json::to_string(&input);
        res.unwrap_or(String::new())
    }
}

impl From<GameState> for ServerMessage {
    fn from(input: GameState) -> Self {
        Self::GameState {
            board: Box::new(input.board),
            turn: input.current_turn,
        }
    }
}

impl From<String> for ClientMessage {
    fn from(input: String) -> Self {
        let res = serde_json::from_str::<ClientMessage>(&input);
        res.unwrap_or(ClientMessage::Unknown)
    }
}
