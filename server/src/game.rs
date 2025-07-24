
const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 10;
const WINNING_TRAIL: usize = 5;
const ACTING_PLAYER: usize = 2;

pub type Board = [[Option<usize>; BOARD_HEIGHT]; BOARD_WIDTH];
#[derive(Debug, Clone)]
pub struct GameState {
    pub board: Board,
    pub current_turn: usize,
    pub room_creator: Option<String>,
    pub members: Vec<String>,  // All people in room
    pub player_queue: Vec<String>,  // People who stepped up to play
    pub active_players: Vec<String>,  // Current 2 players in game
    pub phase: GamePhase,
    pub game_id: Option<String>,  // Database game record ID
}

#[derive(Debug, Clone)]
pub enum GamePhase {
    Ready,
    Action,
    Scoreboard,
}

#[derive(Debug)]
pub enum MoveResult {
    Ok,
    Err,
    Win,
    Draw,
}
impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
impl GameState {
    pub fn new() -> Self {
        Self {
            board: [[None; BOARD_HEIGHT]; BOARD_WIDTH],
            current_turn: usize::MAX,
            room_creator: None,
            members: Vec::new(),
            player_queue: Vec::new(),
            active_players: Vec::new(),
            phase: GamePhase::Ready,
            game_id: None,
        }
    }

    pub fn add_member(&mut self, member: String) -> usize {
        let id = self.members.len();
        
        // First member becomes room creator
        if self.room_creator.is_none() {
            self.room_creator = Some(member.clone());
        }
        
        self.members.push(member);
        id
    }

    pub fn remove_member(&mut self, member: String) {
        // Remove from all lists
        self.members.retain(|m| m != &member);
        self.player_queue.retain(|m| m != &member);
        self.active_players.retain(|m| m != &member);
        
        // Transfer room creator if needed
        if self.room_creator.as_ref() == Some(&member) {
            self.room_creator = self.members.first().cloned();
        }
    }

    fn count_trail(&self, x: usize, y: usize) -> usize {
        if self.board[x][y].is_none() {
            return 0;
        }
        let traverse = |dx, dy, v| {
            let mut ret = 0;
            let mut i = x as i32 + dx;
            let mut j = y as i32 + dy;
            while i >= 0
                && i < self.board.len() as i32
                && j >= 0
                && j < self.board[0].len() as i32
                && self.board[i as usize][j as usize].is_some()
                && self.board[i as usize][j as usize].unwrap() == v
            {
                ret += 1;
                i += dx;
                j += dy;
            }
            ret
        };
        let v = self.board[x][y].unwrap();
        const MOVES: [(i32, i32); 4] = [
            (1, 1),  // diagonal 1
            (1, -1), // diagonal 2
            (0, 1),  // vertical
            (1, 0),  // horizontal
        ];
        MOVES
            .into_iter()
            .map(|(dx, dy)| traverse(dx, dy, v) + traverse(-dx, -dy, v) + 1)
            .max()
            .unwrap_or(1)
    }

    fn is_board_full(&self) -> bool {
        for row in &self.board {
            for cell in row {
                if cell.is_none() {
                    return false;
                }
            }
        }
        true
    }

    pub fn place(&mut self, x: usize, y: usize, member_id: usize) -> MoveResult {
        if member_id >= self.members.len() {
            eprintln!("Invalid member id {}", member_id);
            return MoveResult::Err;
        }
        
        let member = &self.members[member_id];
        
        // Check if member is an active player
        let player_index = match self.active_players.iter().position(|p| p == member) {
            Some(idx) => idx,
            None => {
                eprintln!("Member {} is not an active player", member);
                return MoveResult::Err;
            }
        };
        
        if self.board[x][y].is_some() {
            eprintln!("Invalid move - position occupied");
            return MoveResult::Err;
        }
        
        if player_index != self.current_turn {
            eprintln!("Not this player's turn: player_index={}, current_turn={}, member={}", 
                player_index, self.current_turn, member);
            return MoveResult::Err;
        }
        
        self.board[x][y] = Some(player_index);
        
        // Check for win
        if self.count_trail(x, y) >= WINNING_TRAIL {
            self.current_turn = usize::MAX;
            self.phase = GamePhase::Scoreboard;
            return MoveResult::Win;
        }
        
        // Check for draw
        if self.is_board_full() {
            self.current_turn = usize::MAX;
            self.phase = GamePhase::Scoreboard;
            return MoveResult::Draw;
        }
        
        self.current_turn = (self.current_turn + 1) % ACTING_PLAYER;
        MoveResult::Ok
    }

    pub fn get_acting_players(&mut self) -> Vec<usize> {
        (0..ACTING_PLAYER).collect()
    }

    pub fn step_up(&mut self, member_id: usize) -> bool {
        if member_id >= self.members.len() {
            eprintln!("Invalid member id {}", member_id);
            return false;
        }
        
        let member = self.members[member_id].clone();
        
        // Check if already in queue
        if self.player_queue.contains(&member) {
            return false;
        }
        
        self.player_queue.push(member);
        true
    }
    
    pub fn step_down(&mut self, member_id: usize) -> bool {
        if member_id >= self.members.len() {
            return false;
        }
        
        let member = &self.members[member_id];
        self.player_queue.retain(|m| m != member);
        true
    }
    
    pub fn start_game(&mut self, member_id: usize) -> bool {
        if member_id >= self.members.len() {
            return false;
        }
        
        // Only room creator can start game
        if self.room_creator.as_ref() != Some(&self.members[member_id]) {
            eprintln!("Only room creator can start the game");
            return false;
        }
        
        // Need at least 2 players in queue
        if self.player_queue.len() < 2 {
            eprintln!("Need at least 2 players in queue");
            return false;
        }
        
        // Take first 2 from queue as active players
        self.active_players = self.player_queue.drain(..2).collect();
        eprintln!("Game starting with players: {:?}", self.active_players);
        self.reset();
        self.phase = GamePhase::Action;
        true
    }
    
    pub fn is_room_creator(&self, member_id: usize) -> bool {
        member_id < self.members.len() && self.room_creator.as_ref() == Some(&self.members[member_id])
    }
    
    pub fn is_active_player(&self, member_id: usize) -> bool {
        member_id < self.members.len() && self.active_players.contains(&self.members[member_id])
    }
    
    pub fn get_member_indices(&self, names: &[String]) -> Vec<usize> {
        names.iter()
            .filter_map(|name| self.members.iter().position(|m| m == name))
            .collect()
    }

    fn reset(&mut self) {
        for i in 0..BOARD_WIDTH {
            for j in 0..BOARD_HEIGHT {
                self.board[i][j] = None;
            }
        }
        self.current_turn = 0;  // First player's turn
    }
}
