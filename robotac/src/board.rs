use std::{ops::BitXor, option::Option};

use itertools::Itertools;
use rand::{
    rngs::StdRng,
    seq::{IteratorRandom, SliceRandom},
    thread_rng, Rng, SeedableRng,
};
use smallvec::SmallVec;
use tac_types::{
    BitBoard, BitBoardGen, Card, Color, Deck, Hand, Home, SevenAction, Square, TacAction, TacMove,
    ALL_COLORS,
};

use crate::knowledge::Knowledge;

#[derive(Clone)]
pub struct Board {
    balls: [BitBoard; 4],
    player_to_move: Color,
    homes: [Home; 4],
    fresh: [bool; 4],
    discard_flag: bool,
    jester_flag: bool,
    devil_flag: bool,
    trade_flag: bool,
    started_flag: bool,
    deck_fresh_flag: bool,
    deck: Deck,
    last_tacable_card: Option<Card>,
    last_tacable_non_jester_card: Option<Card>,
    hands: [Hand; 4],
    traded: [Option<Card>; 4],
    one_or_thirteen: [bool; 4],
    pub move_count: u32,
    seed: u64,
    started: Color,
    previous_balls: [BitBoard; 4],
    previous_homes: [Home; 4],
    previous_fresh: [bool; 4],
}

#[allow(dead_code)]
pub struct PackedBoard {
    balls: [BitBoard; 4],
    // homes: [Home; 4],
    // 4 bits per home
    homes: u16,
    // base: [u8; 4],
    // 3 bits per base (max is four -> 100)
    // 4 unused bits
    base: u16,
    // fresh: [bool; 4],
    // + 4 bits
    // discard_flag: bool,
    // jester_flag: bool,
    // devil_flag: bool,
    // trade_flag: bool,
    // + 1 bit each
    // one_or_thirteen: [bool; 4],
    // + 4 bits
    // player_to_move: Color,
    // + 2 bits
    // -> 14 bits
    flags: u16,
    // Could be improved?
    // Maybe enum map with u8 for each card which should be 18 * (u8 + u8) -> 18 * 2 bytes
    // top_idx, times_dealt -> u8 each -> 2 bytes
    // Total: 18 * 2 + 2 = 38 bytes, down from
    deck: Deck,
    // 24 u8, could be smallvec
    discarded: Vec<Card>,
    last_move: Option<tac_types::PackedTacMove>,
    // 1 card -> u8, 6 cards in hand -> 48 bits -> u64
    // 1 card could be 5 bits so 6 cards -> 30 bits -> u32
    // would be 96 bytes -> 16 (u32) or 32 (u64) bytes
    hands: [u32; 4],
    // Can't be improved I think
    traded: [Option<Card>; 4],
    // This doesn't belong here
    pub move_count: u32,
    seed: u64,
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    #[must_use]
    pub fn new() -> Self {
        Self::new_with_seed(0)
    }

    pub fn new_with_seed(seed: u64) -> Self {
        let mut s = Self {
            balls: [BitBoard::EMPTY; 4],
            player_to_move: Color::Black,
            homes: [Home::EMPTY; 4],
            fresh: [true; 4],
            discard_flag: false,
            jester_flag: false,
            devil_flag: false,
            trade_flag: false,
            started_flag: false,
            deck_fresh_flag: false,
            deck: Deck::default(),
            last_tacable_card: None,
            last_tacable_non_jester_card: None,
            hands: [const { Vec::new() }; 4].map(Hand::new),
            traded: [None; 4],
            one_or_thirteen: [false; 4],
            move_count: 0,
            seed,
            started: Color::Black,
            previous_balls: [BitBoard::EMPTY; 4],
            previous_homes: [Home::EMPTY; 4],
            previous_fresh: [true; 4],
        };

        s.deal_new();
        s
    }

    pub fn new_random_state(seed: u64) -> Self {
        let mut s = Self::new_with_seed(seed);

        // Generate random bitboard, iterate through all its bits as squares and set them to random colors
        let mut rng = StdRng::seed_from_u64(seed);
        let bb = BitBoardGen::default().with_max(16).gen();
        for sq in bb.iter() {
            let color = loop {
                let c = ALL_COLORS.choose(&mut rng).cloned().unwrap_or(Color::Black);
                if s.balls[c as usize].len() < 4 {
                    break c;
                }
            };
            s.set(sq, color);
        }

        for color in ALL_COLORS {
            let base = s.num_base(color);
            let num_home = (0..=base).choose(&mut rng).unwrap_or(0);
            let home = loop {
                let res: u8 = rng.gen_range(0..16);
                if res.count_ones() == num_home as u32 {
                    break res;
                }
            };
            s.homes[color as usize] = Home(home);
        }
        s
    }
    /// Put ball from given player onto the board.
    /// Captures any ball that was on the starting position.
    pub fn put_ball_in_play(&mut self, color: Color) -> Option<Color> {
        assert!(self.num_base(color) != 0);
        let capture = self.capture(color.home());
        self.set(color.home(), color);
        self.fresh[color as usize] = true;
        capture
    }

    /// Move ball from `start` to `end`.
    /// Captures any ball that was on the `end`.
    pub fn move_ball(&mut self, start: Square, end: Square, color: Color) -> Option<Color> {
        let capture = self.capture(end);
        self.unset(start, color);
        self.set(end, color);
        if color.home() == start {
            self.fresh[color as usize] = false;
        }
        capture
    }

    /// Move ball from `start` to `goal_pos`.
    pub fn move_ball_to_goal(&mut self, start: Square, goal_pos: u8, color: Color) {
        self.unset(start, color);
        self.homes[color as usize].set(goal_pos);
    }

    /// Move ball that is in it's home from `start` to `end`.
    pub fn move_ball_in_goal(&mut self, start: u8, end: u8, color: Color) {
        self.homes[color as usize].unset(start);
        self.homes[color as usize].set(end);
    }

    /// Swaps the position of the balls on `sq1` and `sq2`.
    pub fn swap_balls(&mut self, sq1: Square, sq2: Square) {
        let c1 = self.color_on(sq1).expect("Square has ball");
        let c2 = self.color_on(sq2).expect("Square has ball");

        self.unset(sq1, c1);
        self.set(sq1, c2);
        self.unset(sq2, c2);
        self.set(sq2, c1);
        // If any of the two squares belong to the home of one of the balls it's no longer fresh
        if sq1 == c1.home() || sq2 == c1.home() {
            self.fresh[c1 as usize] = false;
        }
        if sq1 == c2.home() || sq2 == c2.home() {
            self.fresh[c2 as usize] = false;
        }
    }

    /// Toggles the state of a square for a given player.
    pub(crate) fn xor(&mut self, square: impl Into<Square>, color: Color) {
        self.balls[color as usize] ^= square.into().bitboard();
    }

    /// Sets square to given color
    /// This is a wrapper around xor with an assert that the square is empty
    pub fn set(&mut self, square: Square, color: Color) {
        assert!(
            self.color_on(square).is_none(),
            "{:?} {:?} {:?}",
            square,
            self.color_on(square),
            color,
        );
        self.xor(square, color);
    }

    /// Removes color from square
    /// This is a wrapper around xor with an assert that the square is occupied by the color
    pub fn unset(&mut self, square: Square, color: Color) {
        assert!(
            self.color_on(square) == Some(color),
            "{:?} {:?} {:?}\n",
            square,
            color,
            self.color_on(square),
        );
        self.xor(square, color);
    }

    /// Checks if there is a ball on `square` and returns it's color if there is any.
    #[must_use]
    pub fn color_on(&self, square: Square) -> Option<Color> {
        for color in &ALL_COLORS {
            if self.balls[(*color) as usize].has(square) {
                return Some(*color);
            }
        }
        None
    }

    /// Try to remove target ball and return its color if there was any.
    pub fn capture(&mut self, target: Square) -> Option<Color> {
        let color = self.color_on(target)?;
        self.unset(target, color);
        Some(color)
    }

    /// Advance to the next player according to turn order.
    pub fn next_player(&mut self) {
        self.player_to_move = self.player_to_move.next();
    }

    #[must_use]
    pub fn current_player(&self) -> Color {
        self.player_to_move
    }

    /// Returns a `BitBoard` representing every ball on the board.
    #[must_use]
    pub fn all_balls(&self) -> BitBoard {
        ALL_COLORS
            .into_iter()
            .fold(BitBoard::EMPTY, |acc, color| acc | self.balls_with(color))
    }

    /// Returns a `BitBoard` representing the balls of a given player.
    #[must_use]
    pub fn balls_with(&self, color: Color) -> BitBoard {
        self.balls[color as usize]
    }

    /// Returns the amount of balls from a given player not in play.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub fn num_base(&self, color: Color) -> u8 {
        // Amount of bits in a bitboard is at most 64 which fits into u8
        4 - self.home(color).amount() - self.balls_with(color).len() as u8
    }

    /// Returns the `Home` of a given player.
    #[must_use]
    pub fn home(&self, color: Color) -> &Home {
        &self.homes[color as usize]
    }

    /// Returns true if player has no ball on home square
    /// or if it is on the home square but hasn't been moved yet.
    #[must_use]
    pub fn fresh(&self, color: Color) -> bool {
        self.fresh[color as usize]
    }

    /// Returns players hand
    #[must_use]
    pub fn hand(&self, color: Color) -> &Hand {
        &self.hands[color as usize]
    }

    /// Returns `true` if the current player is forced to discard a card.
    #[must_use]
    pub fn force_discard(&self) -> bool {
        self.discard_flag
    }

    /// Returns `true` if new round start.
    #[must_use]
    pub fn just_started(&self) -> bool {
        self.started_flag
    }

    /// Returns `true` if first round for deck.
    #[must_use]
    pub fn deck_fresh(&self) -> bool {
        self.deck_fresh_flag
    }

    /// Returns `true` if current player played jester and needs to play another card.
    #[must_use]
    pub fn jester_flag(&self) -> bool {
        self.jester_flag
    }

    /// Checks if a ball at a given position can reach its home with a given amount.
    /// Returns the position in the goal if able to.
    #[must_use]
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
    #[must_use]
    pub fn last_played(&self) -> Option<Card> {
        self.last_tacable_card
            .zip(self.last_tacable_non_jester_card)
            .map(|(l, lj)| if l == lj { l } else { lj })
    }

    /// Returns `true` if the there is no ball between `start` and `goal`.
    /// Requires that `start` != `goal`
    #[must_use]
    pub fn can_move(&self, start: Square, goal: Square) -> bool {
        if start == goal {
            return false;
        }
        self.distance_to_next(start) >= start.distance_to(goal)
    }

    #[must_use]
    pub fn distance_to_next(&self, start: Square) -> u8 {
        // Get the distance of the start to the zero square
        let offset = start.0;
        // Convert to bitboard representation
        let start = start.bitboard();

        self.all_balls() // Need to check all balls for potential blockers
            .bitxor(start) // Remove start bit
            .rotate_right(offset) // Rotate by distance start bit has to the 0th bit
            .next_square() // Get the next set bit which also holds the distance
            .0
    }

    /// Returns true if square is occupied
    #[must_use]
    pub fn occupied(&self, square: Square) -> bool {
        self.all_balls().has(square)
    }

    /// Apply a `TacMove` to the current state
    pub fn play(&mut self, mv: &TacMove) {
        self.jester_flag = false;
        self.devil_flag = false;
        self.started_flag = false;
        self.deck_fresh_flag = false;
        let player = self.player_to_move;
        if matches!(mv.action, TacAction::Trade) {
            self.trade(mv.card, player);
            if self.traded.iter().all(Option::is_some) {
                self.take_traded();
            }
            self.next_player();
        } else {
            let current_balls = self.balls;
            let current_homes = self.homes;
            let current_fresh = self.fresh;
            if matches!(mv.card, Card::Tac)
                && !matches!(mv.action, TacAction::Discard | TacAction::Jester)
            {
                assert!(!matches!(mv.action, TacAction::Trade));
                self.tac_undo();
            }
            let could_be_removed = self.hands[player as usize].remove(mv.card);
            if !could_be_removed {
                panic!(
                    "We require the card to be in hand {:?} {:?}",
                    mv.card, self.hands[player as usize]
                );
            }
            self.apply_action(mv.action.clone(), mv.played_for);
            if !matches!(mv.card, Card::Tac) && !matches!(mv.card, Card::Jester) {
                if !matches!(mv.action, TacAction::Jester) {
                    self.last_tacable_card = Some(mv.card);
                }
                self.last_tacable_non_jester_card = Some(mv.card);
            }

            if !matches!(mv.action, TacAction::Jester) {
                self.previous_balls = current_balls;
                self.previous_homes = current_homes;
                self.previous_fresh = current_fresh;
            }

            if self.hands.iter().all(Hand::is_empty) {
                assert!(!self.discard_flag);
                self.deal_new();
                self.last_tacable_card.take();
                self.last_tacable_non_jester_card.take();
                self.player_to_move = self.started.next();
                self.started = self.player_to_move;
            } else if !self.jester_flag {
                self.next_player();
            }
        }
        self.move_count += 1;
    }

    pub fn apply_action(&mut self, action: TacAction, player: Color) {
        match action {
            TacAction::Step { from, to } => {
                self.move_ball(from, to, player);
            }
            TacAction::StepHome { from, to } => self.move_ball_in_goal(from, to, player),
            TacAction::StepInHome { from, to } => self.move_ball_to_goal(from, to, player),
            TacAction::Trickster { target1, target2 } => self.swap_balls(target1, target2),
            TacAction::Enter => {
                self.put_ball_in_play(player);
            }
            TacAction::Suspend => self.discard_flag = true,
            TacAction::Jester => {
                self.jester_flag = true;
                self.hands.rotate_left(1);
            }
            TacAction::Devil => self.devil_flag = true,
            TacAction::Discard => self.discard_flag = false,
            TacAction::SevenSteps { steps, partner_idx } => {
                let partner_idx = partner_idx.unwrap_or(steps.len());
                let steps = steps
                    .into_iter()
                    .enumerate()
                    .map(|(idx, s)| {
                        let play_for = if idx < partner_idx {
                            player
                        } else {
                            player.partner()
                        };
                        (s, play_for)
                    })
                    .collect_vec();
                for (s, play_for) in &steps {
                    match s {
                        // Can't do full move here, because of how we handle capturing below
                        SevenAction::Step { from, .. } => self.unset(*from, *play_for),
                        SevenAction::StepHome { from, to } => {
                            self.move_ball_in_goal(*from, *to, *play_for);
                        }
                        SevenAction::StepInHome { from, to } => {
                            self.move_ball_to_goal(*from, *to, *play_for);
                        }
                    }
                }

                let mut board_steps: SmallVec<_, 4> = steps
                    .clone()
                    .into_iter()
                    .filter_map(|(s, play_for)| match s {
                        SevenAction::Step { from, to } => Some((from, to, false, play_for)),
                        SevenAction::StepInHome { from, .. } => {
                            Some((from, play_for.home(), true, play_for))
                        }
                        SevenAction::StepHome { .. } => None,
                    })
                    .collect();
                // Remove any steps which do not go into home and are fully contained by another step
                board_steps = board_steps
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, (s, e, g, p))| {
                        (*g || !board_steps
                            .iter()
                            .enumerate()
                            .any(|(idx2, (s2, e2, _, _))| {
                                idx != idx2 && s.in_range(*s2, *e2) && e.in_range(*s2, *e2)
                            }))
                        .then_some((*s, *e, *g, *p))
                    })
                    .collect();
                let mut change = true;
                while change {
                    change = false;
                    for (s, e, _, _) in &mut board_steps {
                        if s != e {
                            // Step one square forwards
                            let next = s.add(1);
                            self.capture(next);
                            *s = next;
                            change = true;
                        }
                    }
                }
                // Finally finish step moves by setting at destination
                for (_, e, g, p) in board_steps.into_iter() {
                    if !g {
                        self.set(e, p);
                    }
                }
            }
            TacAction::Warrior { from, to } => {
                if from == to {
                    self.capture(from);
                } else {
                    self.move_ball(from, to, player);
                }
            }
            TacAction::Trade => {}
        }
    }

    /// Undo to last state
    pub fn tac_undo(&mut self) {
        self.discard_flag = false;
        std::mem::swap(&mut self.balls, &mut self.previous_balls);
        std::mem::swap(&mut self.homes, &mut self.previous_homes);
        std::mem::swap(&mut self.fresh, &mut self.previous_fresh);
    }

    /// Set card to be traded
    pub fn trade(&mut self, card: Card, player: Color) {
        self.hands[player as usize].remove(card);
        self.traded[player.partner() as usize] = Some(card);
    }

    /// Put each traded card into the hand they belong to
    pub fn take_traded(&mut self) {
        self.trade_flag = false;
        for player in &ALL_COLORS {
            self.hands[*player as usize].push(
                self.traded[*player as usize]
                    .take()
                    .expect("Every player put up a card for trade"),
            );
        }
    }

    /// Returns true if we are in trade phase
    #[must_use]
    pub fn need_trade(&self) -> bool {
        self.trade_flag && self.traded[self.player_to_move.partner() as usize].is_none()
    }

    /// Begin trade phase
    pub fn begin_trade(&mut self) {
        self.trade_flag = true;
    }

    /// Returns true if there is exactly one player that hasn't traded yet
    pub fn is_trade_almost_finished(&self) -> bool {
        self.traded
            .iter()
            .filter(|x| x.is_none())
            .exactly_one()
            .is_ok()
    }

    /// Deal a new set of hands to each player
    pub fn deal_new(&mut self) {
        assert!(self.hands.iter().all(Hand::is_empty));
        let mut rng = StdRng::seed_from_u64(self.seed);
        // let mut rng = thread_rng();
        let dealt_cards = self.deck.deal(&mut rng);
        self.deck_fresh_flag = self.deck.fresh();
        for set in dealt_cards.chunks_exact(4) {
            for (cidx, card) in set.iter().enumerate() {
                self.hands[cidx].push(*card);
            }
        }
        self.one_or_thirteen = self
            .hands
            .clone()
            .map(|h| h.iter().any(|c| matches!(c, Card::One | Card::Thirteen)));
        self.started_flag = true;
        self.begin_trade();
    }

    #[must_use]
    pub fn can_play(&self, player: Color) -> bool {
        !self.balls_with(player).is_empty()
    }

    #[must_use]
    pub fn play_for(&self, player: Color) -> Color {
        if self.home(player).is_full() {
            player.partner()
        } else {
            player
        }
    }

    pub fn redetermine(&mut self, observer: Color, knowledge: &Knowledge) {
        // let mut rng = StdRng::seed_from_u64(self.seed);
        let mut rng = rand::thread_rng();
        let observer_hand = self.hand(observer).clone();
        // Store hand count first
        let amounts = ALL_COLORS
            .into_iter()
            .filter_map(|player| {
                let exact_sum = knowledge
                    .known_cards(player)
                    .into_iter()
                    .fold(0, |acc, (_, amount, exact)| {
                        acc + if exact { amount } else { 0 }
                    });
                assert!(self.hand(player).amount() >= exact_sum as usize);
                (player != observer)
                    .then_some((player, self.hand(player).amount() - exact_sum as usize))
            })
            .collect_vec();

        // Put back cards in hand back into deck
        for player in ALL_COLORS {
            if player == observer {
                continue;
            }
            for card in self.hands[player as usize].0.drain(..) {
                self.deck.put_back(card);
            }
            assert!(self.hands[player as usize].is_empty());
        }

        // Draw cards equal to the amount put back
        for (player, amount) in amounts {
            assert!(player != observer);
            let hand = &mut self.hands[player as usize];
            let mut known = knowledge.known_cards(player);
            for (card, amnt, is_exact) in &mut known {
                if *is_exact {
                    (0..*amnt).for_each(|_| {
                        self.deck.take(*card);
                        hand.push(*card);
                    });
                    *amnt = 0;
                }
            }
            (0..amount).for_each(|_| {
                let mut drawn = self.deck.draw_one(&mut rng);
                while known.iter().any(|(c, a, _)| *c == drawn && *a == 0) {
                    self.deck.put_back(drawn);
                    drawn = self.deck.draw_one(&mut rng);
                }
                if let Some((_, a, _)) = known.iter_mut().find(|(c, _, _)| *c == drawn) {
                    assert!(*a > 0);
                    *a -= 1;
                }

                hand.push(drawn);
            });
        }
        assert!(self
            .hand(observer)
            .iter()
            .all(|c| { observer_hand.iter().any(|c2| c2 == c) }));
    }

    #[must_use]
    pub fn openings(&self) -> [bool; 4] {
        self.one_or_thirteen
    }

    #[must_use]
    pub fn deck(&self) -> &Deck {
        &self.deck
    }

    #[must_use]
    pub fn won(&self, player: Color) -> bool {
        self.home(player).is_full() && self.home(player.partner()).is_full()
    }

    #[cfg(test)]
    pub fn set_player(&mut self, player: Color) {
        self.player_to_move = player;
    }
    #[cfg(test)]
    pub fn add_hand(&mut self, player: Color, card: Card) {
        self.hands[player as usize].0.push(card);
    }

    pub fn print_balls(&self) {
        for (idx, ball) in self.all_balls().iter().enumerate() {
            if idx % 8 == 0 {
                println!();
            }
            print!("({}, {:?}), ", ball.0, self.color_on(ball).unwrap());
        }
        println!();
    }
}

impl std::fmt::Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.move_count)?;
        for (idx, ball) in self.all_balls().iter().enumerate() {
            if idx % 8 == 0 {
                writeln!(f)?;
            }
            write!(f, "({}, {:?}), ", ball.0, self.color_on(ball).unwrap())?;
        }
        write!(f, "\nhands:\n")?;
        for hand in &self.hands {
            writeln!(f, "{:?}, ", hand.0)?;
        }
        write!(f, "deck: ")?;
        write!(f, "{:?}, ", self.deck)?;
        write!(f, "\nhomes: ")?;
        for home in self.homes {
            write!(f, "{:#06b}, ", home.0)?;
        }
        write!(f, "\n1/13: ")?;
        for one_or_thirteen in self.one_or_thirteen {
            write!(f, "{one_or_thirteen}, ")?;
        }
        write!(f, "\ntraded: ")?;
        for traded in self.traded {
            write!(f, "{traded:?}, ")?;
        }
        write!(f, "\nfresh: ")?;
        for fresh in self.fresh {
            write!(f, "{fresh}, ")?;
        }
        write!(f, "\nlast_move: {:?}\n", self.last_tacable_card)?;
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
    use smallvec::smallvec;
    #[test]
    fn can_move() {
        let mut board = Board::new();
        board.xor(Square(10), Color::Black);
        for i in 1..64u8 {
            assert!(board.can_move(Square(10), Square(10).add(i)));
        }
        board.xor(Square(12), Color::Blue);
        for i in 1..3u8 {
            assert!(board.can_move(Square(10), Square(10).add(i)));
        }
        for i in 3..13u8 {
            assert!(!board.can_move(Square(10), Square(10).add(i)));
        }
    }

    #[test]
    fn selfcapture() {
        use Color::*;
        let mut board = Board::new();
        board.xor(0, Black);
        board.xor(7, Black);
        board.add_hand(Black, Card::Seven);
        board.add_hand(Blue, Card::Tac);
        let black_move = TacMove::new(
            Card::Seven,
            TacAction::SevenSteps {
                steps: smallvec![SevenAction::Step {
                    from: Square(0),
                    to: Square(7),
                }],
                partner_idx: None,
            },
            Black,
            Black,
        );
        board.play(&black_move);
        println!("{:?}", board.moves_for_card(Blue, Card::Tac));
        println!("{board:?}");
    }
    #[test]
    fn seven_undo_bug() {
        use Color::*;
        use TacAction::*;
        let mut board = Board::new();
        board.xor(0, Black);
        board.xor(3, Green);
        board.xor(10, Green);
        board.xor(28, Red);
        board.xor(34, Green);
        board.xor(46, Green);
        board.xor(60, Red);
        board.xor(61, Red);
        board.xor(62, Black);

        board.homes[Black as usize].set(0);

        board.player_to_move = Green;

        board.add_hand(Green, Card::Seven);
        board.add_hand(Red, Card::Tac);
        board.add_hand(Black, Card::Tac);
        board.add_hand(Blue, Card::Tac);
        let green_move = TacMove::new(
            Card::Seven,
            TacAction::SevenSteps {
                steps: smallvec![
                    SevenAction::Step {
                        from: Square(3),
                        to: Square(5),
                    },
                    SevenAction::Step {
                        from: Square(10),
                        to: Square(12),
                    },
                    SevenAction::Step {
                        from: Square(34),
                        to: Square(35),
                    },
                    SevenAction::Step {
                        from: Square(46),
                        to: Square(48),
                    },
                ],
                partner_idx: None,
            },
            Green,
            Green,
        );
        let red_move = TacMove::new(
            Card::Tac,
            TacAction::SevenSteps {
                steps: smallvec![SevenAction::Step {
                    from: Square(60),
                    to: Square(3),
                }],
                partner_idx: None,
            },
            Red,
            Red,
        );
        let black_move = TacMove::new(
            Card::Tac,
            TacAction::SevenSteps {
                steps: smallvec![
                    SevenAction::Step {
                        from: Square(0),
                        to: Square(1),
                    },
                    SevenAction::StepHome { from: 0, to: 1 },
                    SevenAction::StepInHome {
                        from: Square(62),
                        to: 0,
                    },
                ],
                partner_idx: None,
            },
            Black,
            Black,
        );

        board.play(&green_move);
        board.play(&red_move);
        board.play(&black_move);
        println!("{board:?}");
        println!("{:?}", board.moves_for_card(Blue, Card::Tac));
    }

    #[test]
    fn swap_fresh() {
        use Color::*;
        let mut board = Board::new();
        board.put_ball_in_play(Black);
        board.put_ball_in_play(Green);
        assert!(board.fresh(Black));
        assert!(board.fresh(Green));
        board.swap_balls(Black.home(), Green.home());
        assert!(!board.fresh(Black));
        assert!(!board.fresh(Green));
        board.swap_balls(Black.home(), Green.home());
        assert!(!board.fresh(Black));
        assert!(!board.fresh(Green));
    }
}
