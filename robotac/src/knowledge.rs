use enum_map::EnumMap;
use tac_types::{Card, Color, TacMove, CARDS};

use crate::{board::Board, hand::Hand};

#[derive(Debug, Clone, Copy)]
pub struct Knowledge {
    observer: Color,
    hands: [EnumMap<Card, CardKnowledgeKind>; 3],
    announce: [bool; 3],
    history: EnumMap<Card, u8>,
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
            announce: [false; 3],
            history: EnumMap::default(),
        }
    }

    pub fn set_announce(&mut self, announce: [bool; 3]) {
        self.announce = announce;

        for (idx, has_one_thirteen) in announce.iter().enumerate() {
            let has_one_thirteen = *has_one_thirteen;
            // Partner
            if idx == 1 {
                if !has_one_thirteen {
                    self.hands[idx][Card::One] = CardKnowledgeKind::Exact(0);
                    self.hands[idx][Card::Thirteen] = CardKnowledgeKind::Exact(0);
                }
                continue;
            }
            if !has_one_thirteen {
                self.hands[idx][Card::One] = CardKnowledgeKind::Atmost(1);
                self.hands[idx][Card::Thirteen] = CardKnowledgeKind::Atmost(1);
            }
        }
    }

    pub fn update_after_move(&mut self, mv: &TacMove, board: &Board) {
        let player = board.play_for(board.current_player());
        self.update_with_card(mv.card);
        self.sync();
        if matches!(mv.action, tac_types::TacAction::Discard) && !board.force_discard() {
            if board.balls_with(player).is_empty() {
                self.discarded_no_balls_in_play(board);
            } else {
                self.discarded_balls_in_play(board, mv.card);
            }
        }
    }

    pub fn discarded_no_balls_in_play(&mut self, board: &Board) {
        let player = board.current_player();
        let home = board.home(board.play_for(player));
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
        let player = board.current_player();
        if card.is_simple().is_some() {
            let ours = board.balls_with(player);
            let all = board.all_balls();
            let max_amount_between_balls = ours
                .iter()
                .max_by_key(|ball| {
                    (all ^ ball.bitboard())
                        .rotate_right(ball.0)
                        .try_next_square()
                        .map_or(0, |s| s.0)
                })
                .map_or(0, |s| s.0);
            for steps in 1..max_amount_between_balls {
                if let Some(c) = Card::from_steps(steps) {
                    self.rule_out(c, player);
                }
            }
        }
    }

    #[must_use]
    pub fn known_cards(&self, player: Color) -> Vec<Card> {
        let mut cards = Vec::new();
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
