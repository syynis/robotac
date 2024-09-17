use enum_map::EnumMap;
use tac_types::{Card, CARDS};

use crate::hand::Hand;

#[derive(Debug, Clone, Copy)]
pub struct Knowledge {
    hands: [EnumMap<Card, CardKnowledge>; 3],
    history: EnumMap<Card, u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Default, Ord, Hash)]
pub enum CardKnowledge {
    #[default]
    Unknown,
    Guaranteed,
    Impossible,
}

impl Knowledge {
    pub fn update_with_announce(&mut self, announce: [bool; 3]) {
        for (idx, val) in announce.iter().enumerate() {
            if *val {
                self.hands[idx][Card::One] = CardKnowledge::Guaranteed;
                self.hands[idx][Card::Thirteen] = CardKnowledge::Guaranteed;
            } else {
                self.hands[idx][Card::One] = CardKnowledge::Impossible;
                self.hands[idx][Card::Thirteen] = CardKnowledge::Impossible;
            }
        }
    }

    pub fn update_with_trade(&mut self, trade: Card) {
        self.hands[1][trade] = CardKnowledge::Guaranteed;
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
        for card in CARDS.iter() {
            if !self.possible(*card) {
                self.hands.iter_mut().for_each(|hand| {
                    hand[*card] = CardKnowledge::Impossible;
                });
            }
        }
    }

    pub fn possible(&self, card: Card) -> bool {
        self.history[card] < card.amount()
    }

    pub fn reset(&mut self) {
        self.hands.iter_mut().for_each(|hand| hand.clear());
        self.history.clear();
    }
}
