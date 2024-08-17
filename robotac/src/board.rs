use std::ops::{BitOr, BitXor};

use itertools::Itertools;
use rand::{rngs::StdRng, thread_rng, SeedableRng};
use tac_types::{
    BitBoard, Card, Color, Home, Square, TacAction, TacMove, TacMoveResult, ALL_COLORS,
};

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
    trade_flag: bool,
    deck: Deck,
    discarded: Vec<Card>,
    past_moves: Vec<(TacMove, Option<TacMoveResult>)>,
    hands: [Hand; 4],
    traded: [Option<Card>; 4],
    one_or_thirteen: [bool; 4],
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    pub fn new() -> Self {
        let mut rng = thread_rng();
        const EMPTY: Vec<Card> = Vec::new();

        let mut s = Self {
            balls: [BitBoard::EMPTY; 4],
            player_to_move: Color::Black,
            homes: [Home::EMPTY; 4],
            outsides: [4; 4],
            fresh: [true; 4],
            discard_flag: false,
            jester_flag: false,
            devil_flag: false,
            trade_flag: false,
            deck: Deck::new(&mut rng),
            discarded: Vec::new(),
            past_moves: Vec::new(),
            hands: [EMPTY; 4].map(Hand::new),
            traded: [None; 4],
            one_or_thirteen: [false; 4],
        };

        s.deal_new();
        s
    }
    pub fn new_with_seed(seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        const EMPTY: Vec<Card> = Vec::new();

        let mut s = Self {
            balls: [BitBoard::EMPTY; 4],
            player_to_move: Color::Black,
            homes: [Home::EMPTY; 4],
            outsides: [4; 4],
            fresh: [true; 4],
            discard_flag: false,
            jester_flag: false,
            devil_flag: false,
            trade_flag: false,
            deck: Deck::new(&mut rng),
            discarded: Vec::new(),
            past_moves: Vec::new(),
            hands: [EMPTY; 4].map(Hand::new),
            traded: [None; 4],
            one_or_thirteen: [false; 4],
        };

        s.deal_new();
        s
    }
    /// Put ball from given player onto the board.
    /// Captures any ball that was on the starting position.
    pub fn put_ball_in_play(&mut self, color: Color) -> Option<Color> {
        let capture = self.capture(color.home());
        self.xor(color.home(), color);
        // Don't need to check for underflow here
        self.outsides[color as usize] -= 1;
        capture
    }

    /// Move ball from `start` to `end`.
    /// Captures any ball that was on the `end`.
    pub fn move_ball(&mut self, start: Square, end: Square, color: Color) -> Option<Color> {
        // Need to capture first in case color on start and end is the same
        let capture = self.capture(end);
        self.xor(start, color);
        self.xor(end, color);
        capture
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
    pub(crate) fn xor(&mut self, square: Square, color: Color) {
        self.balls[color as usize] ^= square.bitboard();
    }

    /// Checks if there is a ball on `square` and returns it's color if there is any.
    pub fn color_on(&self, square: Square) -> Option<Color> {
        for color in ALL_COLORS.iter() {
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

    /// Returns true if player has no ball on home square
    /// or if it is on the home square but hasn't been moved yet.
    pub fn fresh(&self, color: Color) -> bool {
        self.fresh[color as usize]
    }

    /// Returns players hand
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

    /// Returns past moves
    pub fn past_moves(&self) -> &Vec<(TacMove, Option<TacMoveResult>)> {
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

    /// Returns true if square is occupied
    pub fn occupied(&self, square: Square) -> bool {
        self.all_balls().has(square)
    }

    /// Apply a `TacMove` to the current state
    pub fn play(&mut self, mv: TacMove) {
        self.jester_flag = false;
        self.devil_flag = false;
        let player = self.player_to_move;
        if matches!(mv.card, Card::Tac)
            && !matches!(mv.action, TacAction::Discard | TacAction::Trade)
        {
            self.tac_undo();
        }
        if matches!(mv.action, TacAction::Trade) {
            self.trade(mv.card, player);
            if self.traded.iter().all(|t| t.is_some()) {
                self.take_traded();
            };
            self.next_player();
        } else {
            let captured = self.apply_action(mv.action.clone(), player);

            self.hands[player as usize].remove(mv.card);
            self.discarded.push(mv.card);
            if !self.jester_flag {
                self.next_player();
            }
            self.past_moves.push((mv.clone(), captured));

            if self.hands.iter().all(|h| h.is_empty()) {
                self.deal_new();
            }
        }
    }

    pub fn apply_action(&mut self, action: TacAction, player: Color) -> Option<TacMoveResult> {
        match action {
            TacAction::Step { from, to } => {
                return self.move_ball(from, to, player).map(TacMoveResult::Capture)
            }
            TacAction::StepHome { from, to } => self.move_ball_in_goal(from, to, player),
            TacAction::StepInHome { from, to } => self.move_ball_to_goal(from, to, player),
            TacAction::Switch { target1, target2 } => self.swap_balls(target1, target2),
            TacAction::Enter => return self.put_ball_in_play(player).map(TacMoveResult::Capture),
            TacAction::Suspend => self.discard_flag = true,
            TacAction::Jester => {
                self.jester_flag = true;
                self.hands.rotate_right(1);
            }
            TacAction::Devil => self.devil_flag = true,
            TacAction::Warrior { from, to } => {
                return self.move_ball(from, to, player).map(TacMoveResult::Capture)
            }
            TacAction::Discard => self.discard_flag = false,
            TacAction::AngelEnter => {
                return self
                    .put_ball_in_play(player.next())
                    .map(TacMoveResult::Capture)
            }
            TacAction::SevenSteps { steps } => {
                todo!();
                steps
                    .iter()
                    .sorted_by_key(|s| match s {
                        TacAction::Step { from, to } => to.0,
                        TacAction::StepHome { from, to } => 64 + from,
                        TacAction::StepInHome { from, to } => {
                            from.0 + from.distance_to_home(player)
                        }
                        _ => unreachable!(""),
                    })
                    .rev();
            }
            TacAction::Trade => {}
        }
        None
    }

    pub fn undo_action(
        &mut self,
        action: TacAction,
        player: Color,
        captured: Option<TacMoveResult>,
    ) {
        match action {
            TacAction::Step { from, to } => {
                self.xor(from, player);
                self.xor(to, player);
                if let Some(TacMoveResult::Capture(captured)) = captured {
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
                if let Some(TacMoveResult::Capture(captured)) = captured {
                    self.outsides[captured as usize] -= 1;
                    self.xor(player.home(), captured);
                }
            }
            TacAction::Suspend => self.discard_flag = false,
            TacAction::Jester => {}
            TacAction::Devil => {}
            TacAction::Warrior { from, to } => {
                if let TacMoveResult::Capture(captured) = captured.expect("Warrior always captures")
                {
                    self.outsides[captured as usize] -= 1;
                    if from != to {
                        self.xor(from, player);
                    }
                    self.xor(to, captured);
                }
            }
            TacAction::Discard => {}
            TacAction::AngelEnter => {
                let next = player.next();
                self.xor(next.home(), next);
                self.outsides[next as usize] += 1;
                if let Some(TacMoveResult::Capture(captured)) = captured {
                    self.outsides[captured as usize] -= 1;
                    self.xor(next.home(), captured);
                }
            }
            TacAction::Trade => unreachable!("Can't undo trading"),
            TacAction::SevenSteps { steps } => todo!(),
        }
    }

    /// Undo last move played according
    pub fn tac_undo(&mut self) {
        let (mv, captured) = self
            .past_moves
            .pop() // Pop here so recursive tac works
            .expect("Undo only ever called with past_moves non-empty");
        let player = self.player_to_move.prev();
        self.undo_action(mv.action.clone(), player, captured.clone());
        if matches!(mv.card, Card::Tac) {
            self.tac_undo_recursive(true, player);
        }
        // Push back when we are done
        self.past_moves.push((mv, captured));
    }

    fn tac_undo_recursive(&mut self, redo: bool, player: Color) {
        let (mv, captured) = self
            .past_moves
            .pop() // Pop here so recursive tac works
            .expect("Undo only ever called with past_moves non-empty");
        let player = player.prev();
        if redo {
            self.apply_action(mv.action.clone(), player);
        } else {
            self.undo_action(mv.action.clone(), player, captured.clone());
        }

        if matches!(mv.card, Card::Tac) {
            self.tac_undo_recursive(!redo, player);
        }
        // Push back when we are done
        self.past_moves.push((mv, captured));
    }

    /// Set card to be traded
    pub fn trade(&mut self, card: Card, player: Color) {
        self.hands[player as usize].remove(card);
        self.traded[player.partner() as usize] = Some(card);
    }

    /// Put each traded card into the hand they belong to
    pub fn take_traded(&mut self) {
        self.trade_flag = false;
        for player in ALL_COLORS.iter() {
            self.hands[*player as usize].push(
                self.traded[*player as usize]
                    .take()
                    .expect("Every player put up a card for trade"),
            );
        }
    }

    /// Returns true if we are in trade phase
    pub fn need_trade(&self) -> bool {
        self.trade_flag && self.traded[self.player_to_move.partner() as usize].is_none()
    }

    /// Begin trade phase
    pub fn begin_trade(&mut self) {
        self.trade_flag = true;
    }

    /// Deal a new set of hands to each player
    pub fn deal_new(&mut self) {
        assert!(self.hands.iter().all(|h| h.is_empty()));
        let mut rng = thread_rng();
        let dealt_cards = self.deck.deal(&mut rng);

        for set in dealt_cards.chunks_exact(4) {
            for (cidx, card) in set.iter().enumerate() {
                self.hands[cidx].push(*card);
            }
        }
        self.one_or_thirteen = self
            .hands
            .clone()
            .map(|h| h.iter().any(|c| matches!(c, Card::One | Card::Thirteen)));
        self.trade_flag = true;
    }

    pub fn can_play(&self, player: Color) -> bool {
        !self.balls_with(player).is_empty()
    }
}

impl std::fmt::Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for ball in self.all_balls() {
            write!(f, "({}, {:?}), ", ball.0, self.color_on(ball).unwrap())?;
        }
        write!(f, "\nhands: ")?;
        for hand in &self.hands {
            write!(f, "{:?}, ", hand.0)?;
        }
        write!(f, "\nhomes: ")?;
        for home in self.homes {
            write!(f, "{:#b}, ", home.0)?;
        }
        write!(f, "\noutside: ")?;
        for outside in self.outsides {
            write!(f, "{}, ", outside)?;
        }
        write!(f, "\n1/13: ")?;
        for one_or_thirteen in self.one_or_thirteen {
            write!(f, "{}, ", one_or_thirteen)?;
        }
        write!(f, "\ntraded: ")?;
        for traded in self.traded {
            write!(f, "{:?}, ", traded)?;
        }
        write!(f, "\nfresh: ")?;
        for fresh in self.fresh {
            write!(f, "{}, ", fresh)?;
        }
        writeln!(f)?;
        writeln!(
            f,
            "to_move {:?}, discard {}, jester {}, devil, {}, trade, {}",
            self.player_to_move,
            self.discard_flag,
            self.jester_flag,
            self.devil_flag,
            self.trade_flag
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn can_move() {
        let mut board = Board::new();
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
