use std::collections::HashSet;

const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 10;
const WINNING_TRAIL: usize = 5;
const ACTING_PLAYER: usize = 2;

pub type Board = [[Option<usize>; BOARD_HEIGHT]; BOARD_WIDTH];
#[derive(Debug, Clone)]
pub struct GameState {
    pub board: Board,
    pub current_turn: usize,
    players: Vec<String>,
    ready_status: Vec<bool>,
    phase: GamePhase,
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
            players: Vec::new(),
            ready_status: Vec::new(),
            phase: GamePhase::Ready,
        }
    }

    pub fn add_player(&mut self, player: String) -> usize {
        let ret = self.players.len();
        self.players.push(player);
        self.ready_status.push(false);
        ret
    }

    pub fn remove_player(&mut self, player: String) {
        if let Some(i) = self.players.iter().position(|p| p == &player) {
            self.players.remove(i);
            self.ready_status.remove(i);
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

    pub fn place(&mut self, x: usize, y: usize, player_id: usize) -> MoveResult {
        if player_id >= self.players.len() {
            eprintln!("Invalid player id {}", player_id);
            return MoveResult::Err;
        }
        if self.board[x][y].is_some() {
            eprintln!("Invalid move");
            return MoveResult::Err;
        }
        if player_id != self.current_turn {
            eprintln!(
                "Can't accept action from player {} at turn {}",
                self.players[player_id], self.current_turn
            );
            return MoveResult::Err;
        }
        self.board[x][y] = Some(player_id);
        if self.count_trail(x, y) >= WINNING_TRAIL {
            self.current_turn = usize::MAX;
            self.phase = GamePhase::Scoreboard;
            return MoveResult::Win;
        }
        self.current_turn = (self.current_turn + 1) % ACTING_PLAYER;
        MoveResult::Ok
    }

    pub fn get_acting_players(&mut self) -> Vec<usize> {
        (0..ACTING_PLAYER).collect()
    }

    pub fn ready_vote(&mut self, player_id: usize, ready: bool) -> bool {
        if player_id >= self.players.len() {
            eprintln!("Invalid player id {}", player_id);
            return false;
        }
        self.ready_status[player_id] = ready;
        if self.ready_status.iter().take(ACTING_PLAYER).all(|&r| r) {
            self.reset();
            self.phase = GamePhase::Action;
            true
        } else {
            false
        }
    }

    fn reset(&mut self) {
        for i in 0..BOARD_WIDTH {
            for j in 0..BOARD_HEIGHT {
                self.board[i][j] = None;
            }
        }
        self.current_turn = 0;
    }
}
