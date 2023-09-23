use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use super::game::GameState;
pub type GameRoom = Arc<Mutex<GameState>>;
pub type GameRooms = Arc<Mutex<HashMap<String, GameRoom>>>;
