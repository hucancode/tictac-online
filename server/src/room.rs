use super::game::GameState;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
pub type GameRoom = Arc<Mutex<GameState>>;
pub type GameRooms = Arc<Mutex<HashMap<String, GameRoom>>>;
