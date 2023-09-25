const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 10;

#[derive(Debug, Clone)]
pub struct GameState {
    pub board: [[Option<usize>; BOARD_HEIGHT]; BOARD_WIDTH],
    pub current_turn: usize,
    player_count: usize,
}
impl GameState {
    pub fn new() -> Self {
        Self {
            board: [[None; BOARD_HEIGHT]; BOARD_WIDTH],
            current_turn: 0,
            player_count: 1,
        }
    }

    pub fn add_player(&mut self) -> usize {
        self.player_count += 1;
        self.player_count - 1
    }

    pub fn remove_player(&mut self, player: usize) {
        if player < self.current_turn {
            self.current_turn -= 1;
        }
        self.player_count -= 1;
    }

    pub fn place(&mut self, x: usize, y: usize, player: usize) -> Result<(), &'static str> {
        if self.board[x][y].is_none() && player == self.current_turn {
            self.board[x][y] = Some(player);
            self.current_turn += 1;
            self.current_turn %= self.player_count;
            Ok(())
        } else {
            Err("Invalid move")
        }
    }
}
