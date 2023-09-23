#[derive(Debug, Clone)]
pub struct GameState {
    pub board: [Option<char>; 9],
    pub current_turn: char,
}
impl GameState {
    pub fn new() -> Self {
        Self {
            board: [None; 9],
            current_turn: 'X',
        }
    }

    pub fn make_move(&mut self, position: usize, player: char) -> Result<(), &'static str> {
        if self.board[position].is_none() && player == self.current_turn {
            self.board[position] = Some(player);
            self.current_turn = if self.current_turn == 'X' { 'O' } else { 'X' };
            Ok(())
        } else {
            Err("Invalid move")
        }
    }
}