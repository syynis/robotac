use std::ops::{BitOr, BitXor};

use rand::thread_rng;
use tac_types::{BitBoard, Card, Color, Home, Square, TacAction, TacMove};

use crate::{deck::Deck, hand::Hand};

#[derive(Clone)]
pub struct Board {
    // TODO dont hardcode 4
    balls: [BitBoard; 4],
    player_to_move: Color,
    homes: [Home; 4],
    outsides: [u8; 4],
    fresh: [bool; 4],
    discard_flag: bool,
    jester_flag: bool,
    devil_flag: bool,
    deck: Deck,
    discarded: Vec<Card>,
    past_moves: Vec<(TacMove, Option<Color>)>,
    hands: [Hand; 4],
    traded: [Option<Card>; 4],
}

impl Default for Board {
    fn default() -> Self {
        let mut rng = thread_rng();
        let mut deck = Deck::new(&mut rng);
        let dealt_cards = deck.deal(&mut rng);
        const EMPTY: Vec<Card> = Vec::new();
        let mut hands = [EMPTY; 4];
        for set in dealt_cards.chunks_exact(4) {
            for (cidx, card) in set.iter().enumerate() {
                hands[cidx].push(*card);
            }
        }

        let hands = hands.map(Hand::new);

        Self {
            balls: [BitBoard::EMPTY; 4],
            player_to_move: Color::Black,
            homes: [Home::EMPTY; 4],
            outsides: [4; 4],
            fresh: [true; 4],
            discard_flag: false,
            jester_flag: false,
            devil_flag: false,
            deck,
            discarded: Vec::new(),
            past_moves: Vec::new(),
            hands,
            traded: [None; 4],
        }
    }
}

impl Board {
    /// Put ball from given player onto the board.
    /// Captures any ball that was on the starting position.
    pub fn put_ball_in_play(&mut self, color: Color) {
        self.capture(color.home());
        self.xor(color.home(), color);
        // Don't need to check for underflow here
        self.outsides[color as usize] -= 1;
    }

    /// Move ball from `start` to `end`.
    /// Captures any ball that was on the `end`.
    pub fn move_ball(&mut self, start: Square, end: Square, color: Color) {
        self.capture(end);
        self.xor(start, color);
        self.xor(end, color);
    }

    /// Move ball from `start` to `goal_pos`.
    pub fn move_ball_to_goal(&mut self, start: Square, goal_pos: u8, color: Color) {
        self.xor(start, color);
        self.homes[color as usize].xor(goal_pos);
    }

    /// Move ball that is in it's home from `start` to `end`.
    pub fn move_ball_in_goal(&mut self, start: u8, end: u8, color: Color) {
        self.homes[color as usize].xor(start);
        self.homes[color as usize].xor(end);
    }

    /// Swaps the position of the balls on `sq1` and `sq2`.
    pub fn swap_balls(&mut self, sq1: Square, sq2: Square) {
        let c1 = self.color_on(sq1).expect("Square has ball");
        let c2 = self.color_on(sq2).expect("Square has ball");

        self.xor(sq1, c1);
        self.xor(sq1, c2);
        self.xor(sq2, c1);
        self.xor(sq2, c2);
    }

    /// Toggles the state of a square for a given player.
    fn xor(&mut self, square: Square, color: Color) {
        self.balls[color as usize] ^= square.bitboard();
    }

    /// Checks if there is a ball on `square` and returns it's color if there is any.
    pub fn color_on(&self, square: Square) -> Option<Color> {
        let colors = [Color::Black, Color::Blue, Color::Green, Color::Red];

        for color in colors.iter() {
            if self.balls[(*color) as usize].has(square) {
                return Some(*color);
            }
        }
        None
    }

    /// Try to remove target ball and return its color if there was any.
    pub fn capture(&mut self, target: Square) -> Option<Color> {
        let color = self.color_on(target)?;
        self.xor(target, color);
        self.outsides[color as usize] += 1;
        Some(color)
    }

    /// Advance to the next player according to turn order.
    pub fn next_player(&mut self) {
        self.player_to_move = self.player_to_move.next()
    }

    pub fn current_player(&self) -> Color {
        self.player_to_move
    }

    /// Returns a `BitBoard` representing every ball on the board.
    pub fn all_balls(&self) -> BitBoard {
        let colors = [Color::Black, Color::Blue, Color::Green, Color::Red];

        colors.iter().fold(BitBoard::EMPTY, |acc, color| {
            acc | self.balls[(*color) as usize]
        })
    }

    /// Returns a `BitBoard` representing the balls of a given player.
    pub fn balls_with(&self, color: Color) -> BitBoard {
        self.balls[color as usize]
    }

    /// Returns the amount of balls from a given player not in play.
    pub fn num_outside(&self, color: Color) -> u8 {
        self.outsides[color as usize]
    }

    /// Returns the `Home` of a given player.
    pub fn home(&self, color: Color) -> Home {
        self.homes[color as usize]
    }

    pub fn fresh(&self, color: Color) -> bool {
        self.fresh[color as usize]
    }

    pub fn hand(&self, color: Color) -> &Hand {
        &self.hands[color as usize]
    }

    /// Returns `true` if the previous player discarded a card.
    pub fn force_discard(&self) -> bool {
        self.discard_flag
    }

    /// Returns `true` if current player played jester and needs to play another card.
    pub fn jester_flag(&self) -> bool {
        self.jester_flag
    }

    /// Checks if a ball at a given position can reach its home with a given amount.
    /// Returns the position in the goal if able to.
    pub fn position_in_home(&self, start: Square, amount: u8, color: Color) -> Option<u8> {
        let min_needed = start.distance_to_home(color) + 1;
        let home_free = self.homes[color as usize].free();
        if (min_needed..min_needed + home_free).contains(&amount) {
            Some(amount - min_needed)
        } else {
            None
        }
    }

    /// Returns the last played card.
    /// Will return `None` if the current player is on the first move.
    pub fn last_played(&self) -> Option<Card> {
        self.discarded.iter().last().copied()
    }

    pub fn past_moves(&self) -> &Vec<(TacMove, Option<Color>)> {
        &self.past_moves
    }

    /// Returns `true` if the there is no ball between `start` and `goal`.
    /// Requires that `start` != `goal`
    pub fn can_move(&self, start: Square, goal: Square) -> bool {
        if start == goal {
            return false;
        }
        // TODO investigate if it is worth splitting the computation into two cases
        // Case 1: start < goal
        // Easy case where we just need to check bits between start and goal
        // Case 2: start > goal
        // Case we currently always do. Requires rotating bitboard instead of simply shifting

        // Get the distance of the start to the zero square
        let offset = start.0;
        // Convert to bitboard representation
        let start = start.bitboard();
        let goal = goal.bitboard();

        self.all_balls() // Need to check all balls for potential blockers
            .bitor(goal) // Set goal bit
            .bitxor(start) // Remove start bit
            .rotate_right(offset) // Rotate by distance start bit has to the 0th bit
            .next_square() // Get the next set bit. If we can_move this should be the goal bit
            .bitboard() // Convert back to bitboard
            .rotate_left(offset) // Rotate back
            .eq(&goal) // Bit has same position as goal bit
    }

    pub fn occupied(&self, square: Square) -> bool {
        self.all_balls().has(square)
    }

    /// Apply a `TacMove` to the current state
    pub fn play(&mut self, mv: TacMove) {
        self.jester_flag = false;
        self.devil_flag = false;
        let player = self.player_to_move;
        if matches!(mv.card, Card::Tac) && !self.discard_flag {
            self.tac_undo();
        }
        match mv.action {
            TacAction::Step { from, to } => self.move_ball(from, to, player),
            TacAction::StepHome { from, to } => self.move_ball_in_goal(from, to, player),
            TacAction::StepInHome { from, to } => self.move_ball_to_goal(from, to, player),
            TacAction::Switch { target1, target2 } => self.swap_balls(target1, target2),
            TacAction::Enter => self.put_ball_in_play(player),
            TacAction::Suspend => self.discard_flag = true,
            TacAction::Jester => {
                self.jester_flag = true;
                self.hands.rotate_right(1);
            }
            TacAction::Devil => self.devil_flag = true,
            TacAction::Warrior { from, to } => self.move_ball(from, to, player),
            TacAction::Discard => self.discard_flag = false,
            TacAction::AngelEnter => self.put_ball_in_play(player.next()),
        }
        self.discarded.push(mv.card);

        if !self.jester_flag {
            self.next_player();
        }
        self.past_moves.push((mv, None));

        if self.hands.iter().all(|h| h.is_empty()) {
            self.deal_new(self.current_player());
        }
    }

    /// Undo last move played according
    pub fn tac_undo(&mut self) {
        let (mv, captured) = self
            .past_moves
            .last()
            .expect("Undo only ever called with past_moves non-empty");
        let (mv, captured) = (*mv, *captured);
        let player = self.player_to_move.prev();
        match mv.action {
            TacAction::Step { from, to } => {
                self.xor(from, player);
                self.xor(to, player);
                if let Some(captured) = captured {
                    self.outsides[captured as usize] -= 1;
                    self.xor(to, captured);
                }
            }
            TacAction::StepHome { from, to } => self.move_ball_in_goal(to, from, player),
            TacAction::StepInHome { from, to } => {
                self.xor(from, player);
                self.home(player).xor(to);
            }
            TacAction::Switch { target1, target2 } => self.swap_balls(target1, target2),
            TacAction::Enter => {
                self.xor(player.home(), player);
                self.outsides[player as usize] += 1;
            }
            TacAction::Suspend => self.discard_flag = false,
            TacAction::Jester => {
                self.jester_flag = false;
                self.hands.rotate_left(1);
            }
            TacAction::Devil => self.devil_flag = false,
            TacAction::Warrior { from, to } => {
                let captured = captured.expect("Warrior always captures");
                self.outsides[captured as usize] -= 1;
                if from != to {
                    self.xor(from, player);
                }
                self.xor(to, captured);
            }
            TacAction::Discard => self.discard_flag = true,
            TacAction::AngelEnter => {
                let next = player.next();
                self.xor(next.home(), next);
                self.outsides[next as usize] += 1;
            }
        }
    }

    /// Set card to be traded
    pub fn trade(&mut self, card: Card, player: Color) {
        self.traded[player.next().next() as usize] = Some(card);
    }

    /// Returns the card that was traded if it exists
    pub fn take_traded(&mut self, player: Color) -> Option<Card> {
        self.traded[player as usize].take()
    }

    pub fn need_trade(&self) -> bool {
        self.traded.iter().all(|t| t.is_none())
    }

    pub fn deal_new(&mut self, dealer: Color) {
        let mut rng = thread_rng();
        let dealt_cards = self.deck.deal(&mut rng);
        const EMPTY: Vec<Card> = Vec::new();
        let mut new_hands = [EMPTY; 4];

        for set in dealt_cards.chunks_exact(4) {
            for (cidx, card) in set.iter().enumerate() {
                new_hands[cidx].push(*card);
            }
        }
        self.hands = new_hands.map(Hand::new);
        self.next_player();
    }
}

impl std::fmt::Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for team in self.balls {
            for square in team {
                write!(f, "{:?}", square)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn can_move() {
        let mut board = Board::default();
        board.xor(Square(10), Color::Black);
        for i in 1..64u8 {
            assert_eq!(true, board.can_move(Square(10), Square(10).add(i)));
        }
        board.xor(Square(12), Color::Blue);
        for i in 1..3u8 {
            assert_eq!(true, board.can_move(Square(10), Square(10).add(i)));
        }
        for i in 3..13u8 {
            assert_eq!(false, board.can_move(Square(10), Square(10).add(i)));
        }
    }
}
