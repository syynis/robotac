use enum_map::EnumMap;
use tac_types::{Card, Color, Hand, TacAction, TacMove, CARDS};

use crate::board::Board;

#[derive(Clone, Copy)]
pub struct Knowledge {
    observer: Color,
    hands: [EnumMap<Card, CardKnowledgeKind>; 3],
    has_opening: [bool; 3],
    pub history: EnumMap<Card, u8>,
    traded_away: Option<Card>,
    got_traded: Option<Card>,
    played_jester: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Default, Ord, Hash)]
enum CardKnowledgeKind {
    #[default]
    Unknown,
    Atmost(u8),
    Exact(u8),
}

impl Knowledge {
    #[must_use]
    pub fn new(observer: Color) -> Self {
        Self {
            observer,
            hands: [EnumMap::default(); 3],
            has_opening: [false; 3],
            history: EnumMap::default(),
            traded_away: None,
            got_traded: None,
            played_jester: false,
        }
    }

    #[must_use]
    pub fn new_from_board(observer: Color, board: &Board) -> Self {
        let mut res = Self::new(observer);
        if board.just_started() {
            let openings = board.openings();
            let others_openings = [
                openings[observer.next() as usize],
                openings[observer.partner() as usize],
                openings[observer.prev() as usize],
            ];
            res.set_openings(others_openings);
        }
        res.update_with_hand(board.hand(observer), observer);
        res.sync();
        res
    }

    pub fn set_openings(&mut self, openings: [bool; 3]) {
        self.has_opening = openings;

        let (next, prev) = (openings[0], openings[2]);
        let partner = openings[1];
        if !partner {
            self.hands[1][Card::One] = CardKnowledgeKind::Exact(0);
            self.hands[1][Card::Thirteen] = CardKnowledgeKind::Exact(0);
        }
        // If both enemies have no openings we know for sure both can't have any
        // If only one of them has no openings, we know they can have at most one (traded from partner)
        // TODO use this information to know when the enemy with no openings played one, we know he can't have any more
        if !next && !prev {
            self.hands[0][Card::One] = CardKnowledgeKind::Exact(0);
            self.hands[0][Card::Thirteen] = CardKnowledgeKind::Exact(0);
            self.hands[2][Card::One] = CardKnowledgeKind::Exact(0);
            self.hands[2][Card::Thirteen] = CardKnowledgeKind::Exact(0);
        } else if !next {
            self.hands[0][Card::One] = CardKnowledgeKind::Atmost(1);
            self.hands[0][Card::Thirteen] = CardKnowledgeKind::Atmost(1);
        } else if !prev {
            self.hands[2][Card::One] = CardKnowledgeKind::Atmost(1);
            self.hands[2][Card::Thirteen] = CardKnowledgeKind::Atmost(1);
        }
    }

    pub fn update_with_move(&mut self, mv: &TacMove, board: &Board) {
        let player = board.current_player();
        // Account for when jester was played this hand
        let has_traded_card = if self.played_jester {
            self.observer.partner().prev()
        } else {
            self.observer.partner()
        };
        if board.just_started() {
            for (card, v) in self.history {
                debug_assert!(v <= card.amount());
            }
            if board.deck_fresh() {
                self.history = EnumMap::default();
            }
            self.hands = [EnumMap::default(); 3];
            self.update_with_hand(board.hand(self.observer), self.observer);
            let announce = board.openings();
            let announce_without_observer = [
                announce[self.observer.next() as usize],
                announce[self.observer.partner() as usize],
                announce[self.observer.prev() as usize],
            ];
            self.set_openings(announce_without_observer);
            self.played_jester = false;
        }
        self.sync();
        for (card, v) in self.history {
            debug_assert!(v <= card.amount(), "{v:?} {card:?} {:?}", card.amount());
        }
        if matches!(mv.action, TacAction::Trade) {
            if mv.played_for == self.observer.partner() {
                self.got_traded = Some(mv.card);
                self.history[mv.card] += 1;
            } else if mv.played_for == self.observer {
                self.traded_away = Some(mv.card);
                if let CardKnowledgeKind::Exact(x) = self.hands[1][mv.card] {
                    self.hands[1][mv.card] = CardKnowledgeKind::Exact(x + 1);
                }
            }
            return;
        }
        // If partner plays card we traded away, don't update history because it is already accounted for
        if let Some(traded) = self.traded_away {
            if traded == mv.card && has_traded_card == player {
                self.traded_away.take();
            } else {
                // Update history with card played
                if player != self.observer {
                    self.update_with_card(mv.card, player);
                }
            }
        } else {
            // Update history with card played
            if player != self.observer {
                self.update_with_card(mv.card, player);
            }
        }
        // Previous player discard because they couldn't play anything
        if matches!(mv.action, tac_types::TacAction::Discard)
            && !board.force_discard()
            && mv.played_for != self.observer
        {
            // TODO If able to tac previous move but discard instead, we know no tac in hand
            self.discarded_no_balls_in_play(board, player);
            if !board.balls_with(player).is_empty() {
                self.discarded_balls_in_play(board, mv.card, player);
            }
        }

        if matches!(mv.action, TacAction::Devil) && mv.played_for == self.observer {
            let next = mv.played_for.next();
            let hand = board.hand(next);
            self.update_with_hand(hand, next);
            self.hands[0] = EnumMap::default();
            for c in hand.iter() {
                self.hands[0][*c] = match self.hands[0][*c] {
                    CardKnowledgeKind::Unknown => CardKnowledgeKind::Exact(1),
                    CardKnowledgeKind::Exact(x) => CardKnowledgeKind::Exact(x + 1),
                    CardKnowledgeKind::Atmost(_) => unreachable!(),
                };
            }
        }

        if matches!(mv.action, TacAction::Jester) {
            // Update new information we will get after jester is performed
            let mut hand = board.hand(self.observer.next()).clone();
            // If we receive the hand of the player that performed jester action,
            // remove the card that caused the action, as it was already accounted for when played
            if self.observer == mv.played_for.prev() {
                hand.remove(mv.card);
            }
            self.update_with_hand(&hand, self.observer.next());
            // Apply rotation for hand knowledge
            self.hands.rotate_left(1);
            // Our hand is already the hand from the player after us before jester
            // So we know every card in it
            self.hands[2] = EnumMap::default();
            for c in board.hand(self.observer).iter() {
                self.hands[2][*c] = match self.hands[2][*c] {
                    CardKnowledgeKind::Unknown => CardKnowledgeKind::Exact(1),
                    CardKnowledgeKind::Exact(x) => CardKnowledgeKind::Exact(x + 1),
                    CardKnowledgeKind::Atmost(_) => unreachable!(),
                };
            }
            self.played_jester = true;
        }
        // Check which cards can still be in the deck
        self.sync();
    }

    pub fn discarded_no_balls_in_play(&mut self, board: &Board, player: Color) {
        let home = *board.home(board.play_for(player));
        // All these can be played even with no balls in play
        // One and thirteen also require for there to be balls inside base
        if board.num_base(board.play_for(player)) > 0 {
            self.rule_out(Card::One, player);
            self.rule_out(Card::Thirteen, player);
        }
        self.rule_out(Card::Devil, player);
        self.rule_out(Card::Jester, player);
        self.rule_out(Card::Angel, player);
        // If there are moveable balls in home
        if !(home.is_locked() || home.is_empty()) {
            // Seven can always be played with unlocked balls
            self.rule_out(Card::Seven, player);
            for c in &[Card::One, Card::Two, Card::Three] {
                // If no moves available for two or three, rule out aswell
                if !Board::home_moves_for(home, player, *c).is_empty() {
                    self.rule_out(*c, player);
                }
            }
        }
    }

    pub fn discarded_balls_in_play(&mut self, board: &Board, card: Card, player: Color) {
        // Card is used to step forward
        if card.is_simple().is_some() {
            let ours = board.balls_with(player);
            let all = board.all_balls();
            // Get the ball with the highest distance forwards to the next ball
            let max_amount_between_balls = ours
                .iter()
                .map(|ball| {
                    (all ^ ball.bitboard())
                        .rotate_right(ball.0)
                        .try_next_square()
                        .map_or(0, |s| s.0)
                })
                .max()
                .map_or(0, |x| x);
            // Rule out any card that could move forwards with the maximum space we have
            for steps in 1..max_amount_between_balls {
                if let Some(c) = Card::from_steps(steps) {
                    self.rule_out(c, player);
                }
            }
        }
        // No four in hand if possible moves but not played
        if !board
            .moves_for_card_squares(board.balls_with(player), player, Card::Four)
            .is_empty()
        {
            self.rule_out(Card::Four, player);
        }
    }

    #[must_use]
    pub fn known_cards(&self, player: Color) -> Vec<(Card, u8, bool)> {
        let mut cards = Vec::new();
        if player == self.observer {
            return cards;
        }
        for (card, knowledge) in self.hands[self.idx(player)] {
            match knowledge {
                CardKnowledgeKind::Exact(x) => cards.push((card, x, true)),
                CardKnowledgeKind::Atmost(x) => cards.push((card, x, false)),
                CardKnowledgeKind::Unknown => {}
            }
        }
        cards
    }

    pub fn rule_out(&mut self, card: Card, player: Color) {
        debug_assert!(player != self.observer);
        self.hands[self.idx(player)][card] = CardKnowledgeKind::Exact(0);
    }

    pub fn set_exact(&mut self, card: Card, player: Color, amount: u8) {
        self.hands[self.idx(player)][card] = CardKnowledgeKind::Exact(amount);
    }

    pub fn update_with_card(&mut self, card: Card, player: Color) {
        if player == self.observer {
            self.history[card] += 1;
        } else {
            // If we know of an exact non-zero amount then history was already accounted for (jester / devil)
            match self.hands[self.idx(player)][card] {
                CardKnowledgeKind::Unknown => self.history[card] += 1,
                CardKnowledgeKind::Atmost(1) => {
                    self.history[card] += 1;
                    self.set_exact(card, player, 0);
                }
                CardKnowledgeKind::Atmost(x) => {
                    debug_assert!(x > 0);
                    self.history[card] += 1;
                    self.hands[self.idx(player)][card] = CardKnowledgeKind::Atmost(x - 1);
                }
                CardKnowledgeKind::Exact(x) => {
                    debug_assert!(x > 0, "{player:?} {card:?}\n{self:?}");
                    self.set_exact(card, player, x - 1);
                }
            }
        }
    }

    pub fn update_with_hand(&mut self, hand: &Hand, player: Color) {
        hand.iter()
            .for_each(|card| self.update_with_card(*card, player));
    }

    pub fn sync(&mut self) {
        for card in &CARDS {
            if !self.possible(*card) {
                self.hands.iter_mut().for_each(|hand| {
                    if let CardKnowledgeKind::Unknown = hand[*card] {
                        hand[*card] = CardKnowledgeKind::Exact(0);
                    };
                });
            }
        }
    }

    #[must_use]
    pub fn possible(&self, card: Card) -> bool {
        self.history[card] < card.amount()
    }

    pub fn reset(&mut self) {
        self.hands.iter_mut().for_each(EnumMap::clear);
        self.history.clear();
    }

    #[must_use]
    fn idx(&self, player: Color) -> usize {
        self.observer.between(player)
    }
}

impl std::fmt::Debug for Knowledge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Obs  {:?}, ", self.observer)?;
        write!(f, "Open {:?}, ", self.has_opening)?;
        write!(f, "Away {:?}, ", self.traded_away)?;
        write!(f, "Got  {:?}, ", self.got_traded)?;
        write!(f, "Jest {:?}\n", self.played_jester)?;
        for (idx, k) in self.hands.iter().enumerate() {
            if idx == 0 {
                write!(f, "next: ")?;
            } else if idx == 1 {
                write!(f, "part: ")?;
            } else {
                write!(f, "prev: ")?;
            }
            for (c, v) in k {
                if matches!(
                    v,
                    CardKnowledgeKind::Atmost(_) | CardKnowledgeKind::Exact(_)
                ) {
                    write!(f, "({c:?}, {v:?}), ")?;
                }
            }
            writeln!(f)?;
        }
        writeln!(f, "{:?}", self.history)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use mcts::GameState;
    use rand::{rngs::StdRng, seq::IteratorRandom, thread_rng, SeedableRng};

    use super::*;
    #[test]
    fn announce() {
        // 2 -> jester
        let mut board = Board::new_with_seed(2);
        let mut rng = StdRng::seed_from_u64(2);
        println!("{board:?}");
        let mut know: [_; 4] =
            core::array::from_fn(|i| Knowledge::new_from_board(Color::from(i), &board));
        for k in know {
            println!("{k:?}");
        }
        for i in 0..5000 {
            let get_moves = &board.get_moves(board.current_player());
            let Some(mv) = get_moves.iter().choose(&mut rng) else {
                // Game over
                break;
            };
            println!("{i}: {mv}");
            if i == 1150 {
                println!("------------------------------------------------------");
                for k in know {
                    println!("{k:?}");
                }
                println!("{board:?}");
                println!("{:?}", board.deck());
                println!("------------------------------------------------------");
            }
            for k in &mut know {
                k.update_with_move(mv, &board);
            }
            board.make_move(mv);
        }
        for k in know {
            println!("{k:?}");
        }
        println!("{board:?}");
    }
}
