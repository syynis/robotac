use crate::Card;
use smallvec::SmallVec;

#[derive(Clone, Debug)]
pub struct Hand(pub SmallVec<Card, 6>);

impl Hand {
    #[must_use]
    pub fn new(cards: Vec<Card>) -> Self {
        Self(cards.into())
    }

    #[must_use]
    pub fn amount(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn push(&mut self, card: Card) {
        self.0.push(card);
    }

    /// # Panics
    /// If card is not in hand
    pub fn remove(&mut self, card: Card) -> bool {
        let Some(pos) = self.0.iter().position(|x| *x == card) else {
            return false;
        };
        // .unwrap_or_else(|| panic!("We require the card to be in hand {card:?} {self:?}")),
        self.0.swap_remove(pos);
        true
    }

    #[must_use]
    pub fn contains(&self, card: Card) -> bool {
        self.0.iter().any(|c| *c == card)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Card> + '_ {
        self.0.iter()
    }
}
