use enum_map::EnumMap;
use tac_types::{Card, CARDS};

use crate::hand::Hand;

#[derive(Debug, Clone, Copy)]
pub struct Knowledge {
    hands: [EnumMap<Card, CardKnowledgeKind>; 3],
    history: EnumMap<Card, u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Default, Ord, Hash)]
pub enum CardKnowledgeKind {
    #[default]
    Unknown,
    Guaranteed,
    Impossible,
}

impl Knowledge {
    pub fn update_with_announce(&mut self, announce: [bool; 3]) {
        for (idx, val) in announce.iter().enumerate() {
            if *val {
                self.hands[idx][Card::One] = CardKnowledgeKind::Guaranteed;
                self.hands[idx][Card::Thirteen] = CardKnowledgeKind::Guaranteed;
            } else {
                self.hands[idx][Card::One] = CardKnowledgeKind::Impossible;
                self.hands[idx][Card::Thirteen] = CardKnowledgeKind::Impossible;
            }
        }
    }

    pub fn update_with_trade(&mut self, trade: Card) {
        self.hands[1][trade] = CardKnowledgeKind::Guaranteed;
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
                    hand[*card] = CardKnowledgeKind::Impossible;
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
}
