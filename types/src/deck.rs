use crate::{Card, NUM_CARDS};
use rand::{seq::SliceRandom, Rng};
use smallvec::SmallVec;

const DECK: [(Card, u8); NUM_CARDS] = {
    #[allow(clippy::enum_glob_use)]
    use Card::*;
    [
        (One, One.amount()),
        (Two, Two.amount()),
        (Three, Three.amount()),
        (Four, Four.amount()),
        (Five, Five.amount()),
        (Six, Six.amount()),
        (Seven, Seven.amount()),
        (Eight, Eight.amount()),
        (Nine, Nine.amount()),
        (Ten, Ten.amount()),
        (Twelve, Twelve.amount()),
        (Thirteen, Thirteen.amount()),
        (Trickster, Trickster.amount()),
        (Jester, Jester.amount()),
        (Angel, Angel.amount()),
        (Devil, Devil.amount()),
        (Warrior, Warrior.amount()),
        (Tac, Tac.amount()),
    ]
};

#[derive(Debug, Clone)]
pub struct Deck {
    cards: [(Card, u8); NUM_CARDS],
    times_dealt: u8,
}

impl Default for Deck {
    fn default() -> Self {
        Self::new()
    }
}

impl Deck {
    #[must_use]
    pub fn new() -> Self {
        Self {
            cards: DECK,
            times_dealt: 0,
        }
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn deal<R: Rng>(&mut self, rng: &mut R) -> SmallVec<Card, 24> {
        if self.times_dealt == 5 {
            *self = Self::default();
        }
        let deal_amount = if self.times_dealt == 4 { 24 } else { 20 };
        let mut cards = SmallVec::new();
        (0..deal_amount).for_each(|_| {
            let card = self.draw_one(rng);
            cards.push(card);
        });

        self.times_dealt += 1;
        cards
    }

    pub fn take(&mut self, card: Card) {
        let amount = &mut self.cards[card as usize].1;
        debug_assert!(*amount > 0);
        *amount -= 1;
    }

    pub fn put_back(&mut self, card: Card) {
        self.cards[card as usize].1 += 1;
        debug_assert!(self.cards[card as usize].1 <= card.amount());
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn draw_one<R: Rng>(&mut self, rng: &mut R) -> Card {
        let (card, amount) = self
            .cards
            .choose_weighted_mut(rng, |(_, amount)| *amount)
            .expect("Will always be non-empty with valid weights");
        debug_assert!(*amount > 0);
        *amount -= 1;
        *card
    }

    #[must_use]
    pub fn fresh(&self) -> bool {
        self.times_dealt == 1
    }
}
