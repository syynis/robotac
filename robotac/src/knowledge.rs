use enum_map::EnumMap;
use tac_types::{Card, Color, TacAction, TacMove, CARDS};

use crate::{board::Board, hand::Hand};

#[derive(Clone, Copy)]
pub struct Knowledge {
    observer: Color,
    hands: [EnumMap<Card, CardKnowledgeKind>; 3],
    has_opening: [bool; 3],
    history: EnumMap<Card, u8>,
    traded_away: Option<Card>,
    got_traded: Option<Card>,
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
        }
    }

    #[must_use]
    pub fn new_from_board(observer: Color, board: &Board) -> Self {
        let mut res = Self::new(observer);
        res.update_with_hand(board.hand(observer));
        if board.just_started() {
            res.hands = [EnumMap::default(); 3];
            let openings = board.openings();
            let others_openings = [
                openings[observer.next() as usize],
                openings[observer.partner() as usize],
                openings[observer.prev() as usize],
            ];
            res.set_openings(others_openings);
        }
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

    pub fn update_after_move(&mut self, mv: &TacMove, board: &Board) {
        if board.just_started() {
            if board.deck_fresh() {
                self.history = EnumMap::default();
            }
            self.hands = [EnumMap::default(); 3];
            self.update_with_hand(board.hand(self.observer));
            let announce = board.openings();
            let announce_without_observer = [
                announce[self.observer.next() as usize],
                announce[self.observer.partner() as usize],
                announce[self.observer.prev() as usize],
            ];
            self.set_openings(announce_without_observer);
        }
        if matches!(mv.action, TacAction::Trade) {
            if mv.played_for == self.observer.partner() {
                self.got_traded = Some(mv.card);
                self.update_with_card(mv.card);
            } else if mv.played_for == self.observer {
                self.traded_away = Some(mv.card);
            }
            return;
        }

        let player = board.play_for(board.current_player());
        // If partner plays card we traded away, don't update history because it is already accounted for
        if let Some(traded) = self.traded_away {
            if traded == mv.card && self.observer.partner() == player.prev() {
                self.traded_away.take();
            }
        } else {
            // Update history with card played
            self.update_with_card(mv.card);
        }
        // Check which cards can still be in the deck
        self.sync();
        // Previous player discard because they couldn't play anything
        if matches!(mv.action, tac_types::TacAction::Discard) && !board.was_force_discard() {
            if board.balls_with(player).is_empty() {
                self.discarded_no_balls_in_play(board);
            } else {
                self.discarded_balls_in_play(board, mv.card);
            }
        }
    }

    pub fn discarded_no_balls_in_play(&mut self, board: &Board) {
        let player = board.current_player();
        let home = *board.home(board.play_for(player));
        // All these can be played even with no balls in play
        self.rule_out(Card::One, player);
        self.rule_out(Card::Thirteen, player);
        self.rule_out(Card::Devil, player);
        self.rule_out(Card::Jester, player);
        self.rule_out(Card::Angel, player);
        // If there are moveable balls in home
        if !(home.is_locked() || home.is_empty()) {
            // Seven can always be played with unlocked balls
            self.rule_out(Card::Seven, player);
            for c in &[Card::Two, Card::Three] {
                // If no moves available for two or three, rule out aswell
                if !Board::home_moves_for(home, player, *c).is_empty() {
                    self.rule_out(*c, player);
                }
            }
        }
        // TODO If previous played card satisfies any of the above, rule out tac aswell
    }

    pub fn discarded_balls_in_play(&mut self, board: &Board, card: Card) {
        self.discarded_no_balls_in_play(board);
        let player = board.play_for(board.current_player());
        // Card is used to step forward
        if card.is_simple().is_some() {
            let ours = board.balls_with(player);
            let all = board.all_balls();
            // Get the ball with the highest distance forwards to the next ball
            let max_amount_between_balls = ours
                .iter()
                .max_by_key(|ball| {
                    (all ^ ball.bitboard())
                        .rotate_right(ball.0)
                        .try_next_square()
                        .map_or(0, |s| s.0)
                })
                .map_or(0, |s| s.0);
            // Rule out any card that could move forwards with the maximum space we have
            for steps in 1..max_amount_between_balls {
                if let Some(c) = Card::from_steps(steps) {
                    self.rule_out(c, player);
                }
            }
        }
        // TODO handle four
    }

    #[must_use]
    pub fn known_cards(&self, player: Color) -> Vec<Card> {
        let mut cards = Vec::new();
        // TODO this check shouldnt be necessary
        if player == self.observer {
            return cards;
        }
        for (card, knowledge) in self.hands[self.idx(player)] {
            match knowledge {
                CardKnowledgeKind::Exact(x) => {
                    (0..x).for_each(|_| cards.push(card));
                }
                CardKnowledgeKind::Atmost(_) | CardKnowledgeKind::Unknown => {}
            }
        }
        cards
    }

    pub fn rule_out(&mut self, card: Card, player: Color) {
        // TODO this check shouldnt be necessary
        if player == self.observer {
            return;
        }
        self.hands[self.idx(player)][card] = CardKnowledgeKind::Exact(0);
    }

    pub fn make_exact(&mut self, card: Card, player: Color) {
        let x = &mut self.hands[self.idx(player)][card];
        *x = match x {
            CardKnowledgeKind::Unknown => CardKnowledgeKind::Unknown,
            CardKnowledgeKind::Exact(x) | CardKnowledgeKind::Atmost(x) => {
                CardKnowledgeKind::Exact(*x)
            }
        }
    }

    pub fn set_exact(&mut self, card: Card, player: Color, amount: u8) {
        self.hands[self.idx(player)][card] = CardKnowledgeKind::Exact(amount);
    }

    /// This doesnt check if the card was already accounted for in `update_with_hand`
    pub fn update_with_card(&mut self, card: Card) {
        self.history[card] += 1;
    }

    pub fn update_with_hand(&mut self, hand: &Hand) {
        hand.iter().for_each(|card| {
            self.history[*card] += 1;
        });
    }

    pub fn sync(&mut self) {
        for card in &CARDS {
            if !self.possible(*card) {
                self.hands.iter_mut().for_each(|hand| {
                    hand[*card] = CardKnowledgeKind::Exact(0);
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

    fn idx(&self, player: Color) -> usize {
        self.observer.between(player)
    }
}

impl std::fmt::Debug for Knowledge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Obs  {:?}", self.observer)?;
        writeln!(f, "Open {:?}", self.has_opening)?;
        writeln!(f, "Away {:?}", self.traded_away)?;
        writeln!(f, "Got  {:?}", self.got_traded)?;
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
    use rand::{seq::IteratorRandom, thread_rng};

    use super::*;
    #[test]
    fn announce() {
        let mut board = Board::new_with_seed(2);
        let mut rng = thread_rng();
        println!("{board:?}");
        let mut know: [_; 4] =
            core::array::from_fn(|i| Knowledge::new_from_board(Color::from(i), &board));
        for k in know {
            println!("{k:?}");
        }
        (0..4).for_each(|_| {
            let get_moves = &board.get_moves(board.current_player());
            let mv = get_moves.iter().choose(&mut rng).unwrap();
            board.make_move(mv);
            for k in &mut know {
                k.update_after_move(mv, &board);
            }
        });
        for k in know {
            println!("{k:?}");
        }
    }
}
