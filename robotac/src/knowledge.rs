use enum_map::EnumMap;
use smallvec::SmallVec;
use tac_types::{Card, Color, Hand, TacAction, TacMove, CARDS};

use crate::board::Board;

// TODO Currently there are a lot of pitfalls because the knowledge of traded cards
// is not integrated into `CardKnowledgKind`. This lead to a lot of logic bugs and should be done
#[derive(Clone, Copy)]
pub struct Knowledge {
    // Owner
    observer: Color,
    // Hand information state for each other player
    hands: [EnumMap<Card, CardKnowledgeKind>; 3],
    // Announcement information for each other player
    has_opening: [bool; 3],
    // How many of each card type seen already
    pub history: EnumMap<Card, u8>,
    // Card we traded away. This holds a value until the card is played
    traded_away: Option<Card>,
    // Card we got. This holds a value until the card is played
    got_traded: Option<Card>,
    // Did jester get played this round
    played_jester: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Default, Ord, Hash)]
enum CardKnowledgeKind {
    #[default]
    Unknown,
    // NOTE this variant will only ever hold 1 because it's only used for tracking announce info
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
            res.has_opening = others_openings;
            res.update_after_announce();
        }
        res.update_with_hand(board.hand(observer), observer);
        res.sync();
        res
    }

    pub fn update_after_announce(&mut self) {
        for (idx, has) in self.has_opening.iter().enumerate() {
            if !has {
                self.hands[idx][Card::One] = CardKnowledgeKind::Exact(0);
                self.hands[idx][Card::Thirteen] = CardKnowledgeKind::Exact(0);
            }
        }
    }

    pub fn update_after_trade(&mut self) {
        let [next, _, prev] = self.has_opening;
        // If only one of enemies has no openings, we know they can have at most one (traded from partner)
        // TODO use this information to know when the enemy with no openings played one, we know he can't have any more
        let one_possible = self.possible(Card::One);
        let thirteen_possible = self.possible(Card::Thirteen);
        if !next {
            if one_possible {
                self.hands[0][Card::One] = CardKnowledgeKind::Atmost(1);
            }
            if thirteen_possible {
                self.hands[0][Card::Thirteen] = CardKnowledgeKind::Atmost(1);
            }
        } else if !prev {
            if one_possible {
                self.hands[2][Card::One] = CardKnowledgeKind::Atmost(1);
            }
            if thirteen_possible {
                self.hands[2][Card::Thirteen] = CardKnowledgeKind::Atmost(1);
            }
        }
    }

    pub fn update_with_move(&mut self, mv: &TacMove, board: &Board) {
        assert_eq!(mv.played_by, board.current_player(), "{mv}");
        let player = mv.played_by;
        // Account for when jester was played this hand
        let has_traded_card = self.has_traded_card();
        // New hand
        if board.just_started() {
            for (card, v) in self.history {
                assert!(v <= card.amount());
            }
            // Full reset knowledge if new deck is played
            if board.deck_fresh() {
                self.history = EnumMap::default();
            }
            // Reset knowledge about hands
            self.hands = [EnumMap::default(); 3];
            // Update with our own hand
            self.update_with_hand(board.hand(self.observer), self.observer);
            // Update with announce
            let announce = board.openings();
            let announce_without_observer = [
                announce[self.observer.next() as usize],
                announce[self.observer.partner() as usize],
                announce[self.observer.prev() as usize],
            ];
            self.has_opening = announce_without_observer;
            self.update_after_announce();
            self.played_jester = false;
        }
        for (card, v) in self.history {
            assert!(v <= card.amount(), "{v:?} {card:?} {:?}", card.amount());
        }
        // Update knowledge after trade
        if matches!(mv.action, TacAction::Trade) {
            // If this move was the last trade update info on enemy openings
            if board.is_trade_almost_finished() {
                self.update_after_trade();
            }
            // Card we got
            if player == self.observer.partner() {
                self.got_traded = Some(mv.card);
                self.history[mv.card] += 1;
            // Card we traded
            } else if player == self.observer {
                self.traded_away = Some(mv.card);
                if let CardKnowledgeKind::Exact(x) = self.hands[1][mv.card] {
                    self.hands[1][mv.card] = CardKnowledgeKind::Exact(x + 1);
                }
            }
            self.sync();
            return;
        }
        // If partner plays card we traded away, don't update history because it is already accounted for
        if let Some(traded) = self.traded_away {
            if traded == mv.card && has_traded_card == player {
                self.traded_away.take();
                // This is copied from `update_with_card` but with history management removed
                let card = mv.card;
                match self.hands[self.idx(player)][card] {
                    CardKnowledgeKind::Unknown => {}
                    CardKnowledgeKind::Atmost(1) => {
                        self.set_exact(card, player, 0);
                    }
                    CardKnowledgeKind::Atmost(x) => {
                        assert!(x > 0);
                        self.hands[self.idx(player)][card] = CardKnowledgeKind::Atmost(x - 1);
                    }
                    CardKnowledgeKind::Exact(x) => {
                        assert!(x > 0, "{player:?} {card:?}\n{self:?}");
                        self.set_exact(card, player, x - 1);
                    }
                }
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
            && player != self.observer
        {
            // TODO If able to tac previous move but discard instead, we know no tac in hand
            self.discarded_no_balls_in_play(board, player);
            if !board.balls_with(player).is_empty() {
                self.discarded_balls_in_play(board, mv.card, player);
            }
        }

        // We played devil so we have perfect knowledge about hand of player after us
        if matches!(mv.action, TacAction::Devil) && player == self.observer {
            let next = player.next();
            assert_eq!(self.idx(next), 0);
            // Get hand of player after us
            let mut hand = board.hand(next).clone();
            // Update history with hand next player
            // TODO this technically does things with knowledge not necessary, look into specializing
            self.update_with_hand(&hand, next);
            // Make hand knowledge exact
            self.hands[0] = EnumMap::default();
            for c in hand.iter() {
                self.hands[0][*c] = match self.hands[0][*c] {
                    CardKnowledgeKind::Unknown => CardKnowledgeKind::Exact(1),
                    CardKnowledgeKind::Exact(x) => CardKnowledgeKind::Exact(x + 1),
                    CardKnowledgeKind::Atmost(_) => unreachable!(),
                };
            }
            // If jester was played and card we traded away was not played yet remove one from history again
            // to account for the fact that we counted it already when seeing it in our hand
            if self.played_jester {
                if let Some(card) = self.traded_away {
                    self.history[card] -= 1;
                }
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
        // All these can be played even with no balls in play
        // One and thirteen also require for there to be balls inside base
        if board.num_base(board.play_for(player)) > 0 {
            self.rule_out(Card::One, player);
            self.rule_out(Card::Thirteen, player);
        }
        self.rule_out(Card::Devil, player);
        self.rule_out(Card::Jester, player);
        self.rule_out(Card::Angel, player);

        let home = *board.home(board.play_for(player));
        // If there are moveable balls in home
        if home.can_move() {
            // Seven can always be played with unlocked balls
            self.rule_out(Card::Seven, player);
            for c in &[Card::One, Card::Two, Card::Three] {
                // If no moves available for two or three, rule out aswell
                if !Board::home_moves_for(home, player, player, *c).is_empty() {
                    self.rule_out(*c, player);
                }
            }
        }
    }

    pub fn discarded_balls_in_play(&mut self, board: &Board, card: Card, player: Color) {
        // Card is used to step forward
        if card.is_simple().is_some() {
            let ours = board.balls_with(player);
            // Get the ball with the highest distance forwards to the next ball
            assert!(!(board.all_balls() ^ ours).is_empty());
            let max_amount_between_balls = ours
                .iter()
                .map(|ball| board.distance_to_next(ball))
                .max()
                .expect("Requirement for this function to be called");
            // Rule out any card that could move forwards with the maximum space we have
            for steps in 1..max_amount_between_balls {
                if let Some(c) = Card::from_steps(steps) {
                    self.rule_out(c, player);
                }
            }
        }
        // No four in hand if possible moves but not played
        if !board
            .moves_for_card_squares(board.balls_with(player), player, player, Card::Four)
            .is_empty()
        {
            self.rule_out(Card::Four, player);
        }
    }

    #[must_use]
    pub fn known_cards(&self, player: Color) -> SmallVec<(Card, u8, bool), 6> {
        let mut cards = SmallVec::new();
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
        assert!(player != self.observer);
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
                    assert!(x > 0);
                    self.history[card] += 1;
                    self.hands[self.idx(player)][card] = CardKnowledgeKind::Atmost(x - 1);
                }
                CardKnowledgeKind::Exact(x) => {
                    assert!(x > 0, "{player:?} {card:?}\n{self:?}");
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
        let traded_card_played = self.traded_card_played();
        let traded_card_owner_idx = self.idx(self.has_traded_card());
        for card in &CARDS {
            if !self.possible(*card) {
                self.hands.iter_mut().enumerate().for_each(|(idx, hand)| {
                    if (traded_card_played || traded_card_owner_idx != idx)
                        && matches!(hand[*card], CardKnowledgeKind::Unknown)
                    {
                        hand[*card] = CardKnowledgeKind::Exact(0);
                    }
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

    pub fn traded_card_played(&self) -> bool {
        self.traded_away.is_none()
    }

    pub fn has_traded_card(&self) -> Color {
        if self.played_jester {
            self.observer.partner().prev()
        } else {
            self.observer.partner()
        }
    }

    pub fn traded_away(&self) -> Option<Card> {
        self.traded_away
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
        writeln!(f, "Jest {:?}", self.played_jester)?;
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
    use tac_types::ALL_COLORS;

    use super::*;
    #[test]
    fn announce() {
        for seed in 0..10000 {
            println!("SEED {seed}");
            let mut board = Board::new_with_seed(seed);
            println!("{board:?}");
            let mut rng = StdRng::seed_from_u64(seed);
            let mut know: [_; 4] =
                core::array::from_fn(|i| Knowledge::new_from_board(Color::from(i), &board));
            for i in 0..10000 {
                let get_moves = &board.get_moves(board.current_player());
                let Some(mv) = get_moves.iter().choose(&mut rng) else {
                    // Game over
                    break;
                };
                println!("{i}: {mv}");
                for k in &mut know {
                    k.update_with_move(mv, &board);
                }
                board.make_move(mv);
            }
        }
    }
    #[test]
    fn redetermine() {
        let seed = 0;
        let mut board = Board::new_with_seed(0);
        let mut rng = StdRng::seed_from_u64(0);
        println!("{board:?}");
        let mut know: [_; 4] =
            core::array::from_fn(|i| Knowledge::new_from_board(Color::from(i), &board));
        for i in 0..123 {
            let get_moves = &board.get_moves(board.current_player());
            let Some(mv) = get_moves.iter().choose(&mut rng) else {
                // Game over
                break;
            };
            for k in &mut know {
                k.update_with_move(mv, &board);
            }
            board.make_move(mv);
        }
        println!("{board:?}");
        for k in know {
            println!("{k:?}");
        }
        for (i, c) in ALL_COLORS.into_iter().enumerate() {
            let mut board = board.clone();
            board.redetermine(c, &know[i]);
        }
    }
}
