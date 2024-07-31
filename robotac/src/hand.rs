use tac_types::Card;

#[derive(Clone)]
pub struct Hand {
    cards: Vec<Card>,
}

impl Hand {
    pub fn new(cards: Vec<Card>) -> Self {
        Self { cards }
    }

    pub fn is_empty(&self) -> bool {
        self.is_empty()
    }

    pub fn push(&mut self, card: Card) {
        self.cards.push(card);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Card> + '_ {
        self.cards.iter()
    }
}
