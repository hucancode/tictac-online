use super::game::Board;
use super::game::GameState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    GameState { board: Board, turn: usize },
    Chat { who: String, content: String },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    Place { x: usize, y: usize },
    Chat { content: String },
    Register { name: String },
    Unknown,
}

impl From<ServerMessage> for String {
    fn from(input: ServerMessage) -> Self {
        let res = serde_json::to_string(&input);
        res.unwrap_or(String::from(""))
    }
}

impl From<GameState> for ServerMessage {
    fn from(input: GameState) -> Self {
        Self::GameState {
            board: input.board,
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
