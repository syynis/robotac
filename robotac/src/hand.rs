use tac_types::Card;

#[derive(Clone)]
pub struct Hand(pub Vec<Card>);

impl Hand {
    pub fn new(cards: Vec<Card>) -> Self {
        Self(cards)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn push(&mut self, card: Card) {
        self.0.push(card);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Card> + '_ {
        self.0.iter()
    }
}
