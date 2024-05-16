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
    players: Vec<usize>,
    next_id: usize,
    ready_players: HashSet<usize>,
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
            next_id: 0,
            ready_players: HashSet::new(),
            phase: GamePhase::Ready,
        }
    }

    pub fn add_player(&mut self) -> usize {
        self.players.push(self.next_id);
        self.next_id += 1;
        self.next_id - 1
    }

    pub fn remove_player(&mut self, player: usize) {
        if let Ok(i) = self.players.binary_search(&player) {
            self.players.swap_remove(i);
        }
        self.ready_players.remove(&player);
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

    pub fn place(&mut self, x: usize, y: usize, player: usize) -> MoveResult {
        if self.board[x][y].is_some() {
            eprintln!("Invalid move");
            return MoveResult::Err;
        }
        if player != self.current_turn {
            eprintln!(
                "Can't accept action from player {} at turn {}",
                player, self.current_turn
            );
            return MoveResult::Err;
        }
        self.board[x][y] = Some(player);
        if self.count_trail(x, y) >= WINNING_TRAIL {
            self.current_turn = self.next_id;
            self.phase = GamePhase::Scoreboard;
            return MoveResult::Win;
        }
        if let Ok(i) = self.players.binary_search(&self.current_turn) {
            let i = (i + 1) % self.players.len();
            self.current_turn = self.players[i];
            MoveResult::Ok
        } else {
            eprintln!(
                "Internal server error, player {} doesn't exist in player list {:?}",
                self.current_turn, self.players
            );
            MoveResult::Err
        }
    }

    pub fn get_acting_players(&mut self) -> Vec<usize> {
        self.players[0..ACTING_PLAYER].to_vec()
    }

    pub fn ready_vote(&mut self, player_id: usize, ready: bool) -> bool {
        if let Ok(i) = self.players.binary_search(&player_id) {
            if i < ACTING_PLAYER {
                if ready {
                    self.ready_players.insert(player_id);
                    if self.ready_players.len() >= ACTING_PLAYER {
                        self.reset();
                        self.phase = GamePhase::Action;
                        return true;
                    }
                } else {
                    self.ready_players.remove(&player_id);
                }
            } else {
                println!("Player {} is not allowed to play, only the first 2 players can play {:?}", player_id, self.players);
            }
        } else {
            eprintln!("Player {} doesn't exist in player list {:?}", player_id, self.players);
        }
        false
    }

    fn reset(&mut self) {
        for i in 0..BOARD_WIDTH {
            for j in 0..BOARD_HEIGHT {
                self.board[i][j] = None;
            }
        }
        self.current_turn = *self.players.first().unwrap_or(&self.next_id);
        self.ready_players.clear();
    }
}
