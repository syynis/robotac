use std::{
    cmp::Ordering,
    ops::{BitOr, BitXor},
    option::Option,
};

use arraydeque::{ArrayDeque, Wrapping};
use itertools::Itertools;
use rand::{rngs::StdRng, SeedableRng};
use smallvec::SmallVec;
use tac_types::{
    BitBoard, Card, Color, Deck, Hand, Home, Square, TacAction, TacMove, TacMoveResult, ALL_COLORS,
};

use crate::knowledge::Knowledge;

// This is is choosen because the situation which needs the most lookup into past is:
// Card - Jester - Tac - Tac - Tac - Tac - Tac
// These are seven cards but we up it to eight so it's a power of two. The performance impact of this decision has not been measured
const PAST_MOVES_LEN: usize = 8;

#[derive(Clone)]
pub struct Board {
    balls: [BitBoard; 4],
    player_to_move: Color,
    homes: [Home; 4],
    base: [u8; 4],
    fresh: [bool; 4],
    discard_flag: bool,
    jester_flag: bool,
    devil_flag: bool,
    trade_flag: bool,
    started_flag: bool,
    deck_fresh_flag: bool,
    deck: Deck,
    discarded: Vec<Card>,
    past_moves: ArrayDeque<(TacMove, Option<TacMoveResult>), PAST_MOVES_LEN, Wrapping>,
    hands: [Hand; 4],
    traded: [Option<Card>; 4],
    one_or_thirteen: [bool; 4],
    pub move_count: u32,
    seed: u64,
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
    past_moves: ArrayDeque<
        (
            tac_types::PackedTacMove,
            Option<tac_types::PackedTacMoveResult>,
        ),
        PAST_MOVES_LEN,
        Wrapping,
    >,
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
            base: [4; 4],
            fresh: [true; 4],
            discard_flag: false,
            jester_flag: false,
            devil_flag: false,
            trade_flag: false,
            started_flag: false,
            deck_fresh_flag: false,
            deck: Deck::default(),
            discarded: Vec::new(),
            past_moves: ArrayDeque::new(),
            hands: [const { Vec::new() }; 4].map(Hand::new),
            traded: [None; 4],
            one_or_thirteen: [false; 4],
            move_count: 0,
            seed,
        };

        s.deal_new();
        s
    }
    /// Put ball from given player onto the board.
    /// Captures any ball that was on the starting position.
    #[must_use]
    pub fn put_ball_in_play(&mut self, color: Color) -> Option<Color> {
        let capture = self.capture(color.home());
        self.set(color.home(), color);
        // Don't need to check for underflow here
        debug_assert_ne!(self.base[color as usize], 0);
        self.base[color as usize] -= 1;
        capture
    }

    /// Move ball from `start` to `end`.
    /// Captures any ball that was on the `end`.
    #[must_use]
    pub fn move_ball(&mut self, start: Square, end: Square, color: Color) -> Option<Color> {
        // Need to capture first in case color on start and end is the same
        let capture = self.capture(end);
        self.unset(start, color);
        self.set(end, color);
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
    }

    /// Toggles the state of a square for a given player.
    pub(crate) fn xor(&mut self, square: Square, color: Color) {
        self.balls[color as usize] ^= square.bitboard();
    }

    /// Sets square to given color
    /// This is a wrapper around xor with an assert that the square is empty
    pub fn set(&mut self, square: Square, color: Color) {
        debug_assert!(self.color_on(square).is_none());
        self.xor(square, color);
    }

    /// Removes color from square
    /// This is a wrapper around xor with an assert that the square is occupied by the color
    pub fn unset(&mut self, square: Square, color: Color) {
        debug_assert!(
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
    #[must_use]
    pub fn capture(&mut self, target: Square) -> Option<Color> {
        let color = self.color_on(target)?;
        self.unset(target, color);
        self.base[color as usize] += 1;
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
    pub fn num_base(&self, color: Color) -> u8 {
        self.base[color as usize]
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

    /// Returns `true` if the previous player was forced to discarded a card.
    #[must_use]
    pub fn was_force_discard(&self) -> bool {
        let len = self.past_moves.len();
        // Last move discard and move before suspend
        self.past_moves
            .back()
            .map_or(false, |(mv, _)| matches!(mv.action, TacAction::Discard))
            && self
                .past_moves
                .get(len - 2)
                .map_or(false, |(mv, _)| matches!(mv.action, TacAction::Suspend))
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
        self.discarded.iter().last().copied()
    }

    /// Returns past moves
    #[must_use]
    pub fn past_moves(
        &self,
    ) -> &ArrayDeque<(TacMove, Option<TacMoveResult>), PAST_MOVES_LEN, Wrapping> {
        &self.past_moves
    }

    /// Returns `true` if the there is no ball between `start` and `goal`.
    /// Requires that `start` != `goal`
    #[must_use]
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
        if matches!(mv.card, Card::Tac)
            && !matches!(
                mv.action,
                TacAction::Discard | TacAction::Trade | TacAction::Jester
            )
        {
            self.tac_undo();
        }
        if matches!(mv.action, TacAction::Trade) {
            self.trade(mv.card, player);
            if self.traded.iter().all(Option::is_some) {
                self.take_traded();
            };
            self.next_player();
        } else {
            self.hands[player as usize].remove(mv.card);
            self.discarded.push(mv.card);
            let captured = self.apply_action(mv.action.clone(), mv.played_for);
            if !self.jester_flag {
                self.next_player();
            }
            self.past_moves.push_back((mv.clone(), captured));

            if self.hands.iter().all(Hand::is_empty) {
                debug_assert!(!self.discard_flag);
                self.deal_new();
                self.past_moves.clear();
                self.discarded.clear();
                self.next_player();
            }
        }
        self.move_count += 1;
        for c in ALL_COLORS {
            debug_assert_eq!(
                self.balls_with(c).len()
                    + self.home(c).amount() as usize
                    + self.num_base(c) as usize,
                4,
                "{:?} {:?} {:?} {:?} {:?}\n",
                c,
                self.balls_with(c).len(),
                self.home(c).amount(),
                self.num_base(c),
                self
            );
        }
    }

    pub fn apply_action(&mut self, action: TacAction, player: Color) -> Option<TacMoveResult> {
        match action {
            TacAction::Step { from, to } => {
                return self.move_ball(from, to, player).map(TacMoveResult::Capture)
            }
            TacAction::StepHome { from, to } => self.move_ball_in_goal(from, to, player),
            TacAction::StepInHome { from, to } => self.move_ball_to_goal(from, to, player),
            TacAction::Trickster { target1, target2 } => self.swap_balls(target1, target2),
            TacAction::Enter => return self.put_ball_in_play(player).map(TacMoveResult::Capture),
            TacAction::Suspend => self.discard_flag = true,
            TacAction::Jester => {
                self.jester_flag = true;
                self.hands.rotate_left(1);
            }
            TacAction::Devil => self.devil_flag = true,
            TacAction::Discard => self.discard_flag = false,
            TacAction::SevenSteps { steps } => {
                for s in &steps {
                    if let TacAction::StepHome { from, to } = s {
                        self.move_ball_in_goal(*from, *to, player);
                    };
                }

                let mut board_steps = steps
                    .iter()
                    .filter_map(|s| match s {
                        TacAction::Step { from, to } => Some((*from, *to, None)),
                        TacAction::StepInHome { from, to } => {
                            Some((*from, player.home(), Some(*to)))
                        }
                        _ => None,
                    })
                    .sorted_unstable_by(|(s1, _, _), (s2, _, _)| {
                        if s1.0 == 0 && s2.0 == 63 {
                            Ordering::Greater
                        } else if s1.0 == 63 && s2.0 == 0 {
                            Ordering::Less
                        } else {
                            Ord::cmp(s1, s2)
                        }
                    })
                    .rev()
                    .collect_vec();
                let mut res = SmallVec::new();
                let mut change = true;
                while change {
                    change = false;
                    for (s, e, g) in &mut board_steps {
                        if s == e {
                            if let Some(goal) = g {
                                self.move_ball_to_goal(*e, *goal, player);
                                *g = None;
                                change = true;
                            }
                        } else {
                            let next = s.add(1);
                            if let Some(cap) = self.move_ball(*s, next, player) {
                                res.push((next, cap));
                            }
                            *s = next;
                            change = true;
                        }
                    }
                }
                if res.is_empty() {
                    return None;
                }
                return Some(TacMoveResult::SevenCaptures(res));
            }
            TacAction::Warrior { from, to } => {
                if from == to {
                    return self.capture(from).map(TacMoveResult::Capture);
                }
                return self.move_ball(from, to, player).map(TacMoveResult::Capture);
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
                self.set(from, player);
                self.unset(to, player);
                if let Some(TacMoveResult::Capture(captured)) = captured {
                    self.base[captured as usize] -= 1;
                    self.set(to, captured);
                }
            }
            TacAction::StepHome { from, to } => self.move_ball_in_goal(to, from, player),
            TacAction::StepInHome { from, to } => {
                self.set(from, player);
                self.homes[player as usize].unset(to);
            }
            TacAction::Trickster { target1, target2 } => self.swap_balls(target1, target2),
            TacAction::Enter => {
                self.unset(player.home(), player);
                self.base[player as usize] += 1;
                if let Some(TacMoveResult::Capture(captured)) = captured {
                    self.base[captured as usize] -= 1;
                    self.set(player.home(), captured);
                }
            }
            TacAction::Suspend => self.discard_flag = false,
            TacAction::Jester | TacAction::Devil | TacAction::Discard => {}
            TacAction::Warrior { from, to } => {
                if let TacMoveResult::Capture(captured) = captured.expect("Warrior always captures")
                {
                    self.base[captured as usize] -= 1;
                    if from == to {
                        debug_assert_eq!(player, captured);
                        self.set(from, player);
                    } else {
                        self.unset(to, player);
                        self.set(to, captured);
                        self.set(from, player);
                    }
                }
            }
            TacAction::SevenSteps { steps } => {
                // We have to do undoing for step and step home in two steps
                // This is because different two steps could share the same square as
                // their start or end square respectively
                // TODO what about order of stephome / stepinhome
                for s in &steps {
                    match s {
                        TacAction::Step { to, .. } => self.unset(*to, player),
                        TacAction::StepHome { to, .. } | TacAction::StepInHome { to, .. } => {
                            self.homes[player as usize].unset(*to);
                        }
                        _ => unreachable!(),
                    }
                }
                for s in &steps {
                    match s {
                        TacAction::Step { from, .. } | TacAction::StepInHome { from, .. } => {
                            self.set(*from, player);
                        }
                        TacAction::StepHome { from, .. } => self.homes[player as usize].set(*from),
                        _ => unreachable!(),
                    }
                }
                if let Some(TacMoveResult::SevenCaptures(captures)) = captured {
                    for (square, color) in captures {
                        self.base[color as usize] -= 1;
                        self.set(square, color);
                    }
                }
            }
            TacAction::Trade => unreachable!("Can't undo trading"),
        }
    }

    /// Undo last move played according
    pub fn tac_undo(&mut self) {
        // TODO this doesnt handle skipping jester if it wasnt discarded
        let mut stored = Vec::new();
        while let Some((mv, captured)) = self.past_moves.pop_back() {
            stored.push((mv.clone(), captured));
            if !(matches!(mv.action, TacAction::Jester)) {
                break;
            }
        }
        // let (mv, captured) = self
        //     .past_moves
        //     .pop_back() // Pop here so recursive tac works
        //     .expect("Undo only ever called with past_moves non-empty");
        // TODO handle play for here
        let (mv, captured) = stored.last().unwrap();
        self.undo_action(mv.action.clone(), mv.played_for, captured.clone());
        if matches!(mv.card, Card::Tac) {
            self.tac_undo_recursive(
                (!matches!(mv.action, TacAction::Discard)).then_some(true),
                self.player_to_move.prev(),
            );
        }
        // Push back when we are done
        for e in stored {
            self.past_moves.push_back(e);
        }
    }

    fn tac_undo_recursive(&mut self, redo: Option<bool>, player: Color) {
        let (mv, captured) = self
            .past_moves
            .pop_back() // Pop here so recursive tac works
            .expect("Undo only ever called with past_moves non-empty");
        if let Some(redo) = redo {
            if redo {
                self.apply_action(mv.action.clone(), mv.played_for);
            } else {
                self.undo_action(mv.action.clone(), mv.played_for, captured.clone());
            }
        }

        if matches!(mv.card, Card::Tac) {
            if let Some(redo) = redo {
                self.tac_undo_recursive(Some(!redo), player.prev());
            } else {
                self.tac_undo_recursive(None, player.prev());
            }
        }
        // Push back when we are done
        self.past_moves.push_back((mv, captured));
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

    /// Deal a new set of hands to each player
    pub fn deal_new(&mut self) {
        debug_assert!(self.hands.iter().all(Hand::is_empty));
        let mut rng = StdRng::seed_from_u64(self.seed);
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
        let mut rng = rand::thread_rng();
        let observer_hand = self.hand(observer).clone();
        // Store hand count first
        let amounts = ALL_COLORS
            .into_iter()
            .filter_map(|player| {
                debug_assert!(self.hand(player).amount() >= knowledge.known_cards(player).len());
                (player != observer).then_some((
                    player,
                    self.hand(player).amount() - knowledge.known_cards(player).len(),
                ))
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
        }

        // Draw cards equal to the amount put back
        // TODO handle at most -> if we drew atmost amount already put back if we draw
        for (player, amount) in amounts {
            debug_assert!(player != observer);
            let hand = &mut self.hands[player as usize];
            let known = knowledge.known_cards(player);
            (0..amount).for_each(|_| {
                let mut drawn = self.deck.draw_one(&mut rng);
                while known.iter().any(|c| *c == drawn) {
                    self.deck.put_back(drawn);
                    drawn = self.deck.draw_one(&mut rng);
                }
                hand.push(drawn);
            });
            for card in known {
                hand.push(card);
            }
            debug_assert!(hand.amount() == amount + knowledge.known_cards(player).len());
        }
        debug_assert!(self
            .hand(observer)
            .iter()
            .all(|c| { observer_hand.iter().any(|c2| c2 == c) }));
    }

    #[must_use]
    pub fn openings(&self) -> [bool; 4] {
        self.one_or_thirteen
    }

    #[cfg(test)]
    pub fn set_player(&mut self, player: Color) {
        self.player_to_move = player;
    }
    #[cfg(test)]
    pub fn add_hand(&mut self, player: Color, card: Card) {
        self.hands[player as usize].0.push(card);
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
        write!(f, "homes: ")?;
        for home in self.homes {
            write!(f, "{:#b}, ", home.0)?;
        }
        write!(f, "\nbase: ")?;
        for base in self.base {
            write!(f, "{base}, ")?;
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
        write!(f, "\npast_moves:\n")?;
        for (mv, captured) in &self.past_moves {
            writeln!(f, "{mv}, {captured:?}")?;
        }
        write!(f, "\ndiscarded:\n")?;
        for c in &self.discarded {
            writeln!(f, "{c:?}")?;
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
}
