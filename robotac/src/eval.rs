use crate::board::Board;

impl Board {
    pub fn eval(&self) -> i64 {
        // Absolute most basic eval for now. Just count the number of balls we have in home
        self.home(self.current_player()).amount() as i64
            + self.home(self.current_player().partner()).amount() as i64
    }
}
