use rand::{seq::SliceRandom, Rng};
use tac_types::Card;

const DECK_SIZE: usize = 104;
const DECK: [Card; DECK_SIZE] = {
    #[allow(clippy::enum_glob_use)]
    use Card::*;
    [
        One, One, One, One, One, One, One, One, One, Two, Two, Two, Two, Two, Two, Two, Three,
        Three, Three, Three, Three, Three, Three, Four, Four, Four, Four, Four, Four, Four, Five,
        Five, Five, Five, Five, Five, Five, Six, Six, Six, Six, Six, Six, Six, Seven, Seven, Seven,
        Seven, Seven, Seven, Seven, Seven, Eight, Eight, Eight, Eight, Eight, Eight, Eight, Nine,
        Nine, Nine, Nine, Nine, Nine, Nine, Ten, Ten, Ten, Ten, Ten, Ten, Ten, Twelve, Twelve,
        Twelve, Twelve, Twelve, Twelve, Twelve, Thirteen, Thirteen, Thirteen, Thirteen, Thirteen,
        Thirteen, Thirteen, Thirteen, Thirteen, Trickster, Trickster, Trickster, Trickster,
        Trickster, Trickster, Trickster, Devil, Angel, Jester, Warrior, Tac, Tac, Tac, Tac,
    ]
};

#[derive(Debug, Clone)]
pub struct Deck {
    cards: [Card; DECK_SIZE],
    top_idx: usize,
    times_dealt: usize,
}

impl Deck {
    pub fn new<R: Rng>(rng: &mut R) -> Self {
        let mut cards = DECK;
        cards.shuffle(rng);

        Self {
            cards,
            top_idx: 0,
            times_dealt: 0,
        }
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn deal<R: Rng>(&mut self, rng: &mut R) -> Vec<Card> {
        if self.times_dealt == 5 {
            self.reset(rng);
        }
        let deal_amount = if self.times_dealt == 4 { 24 } else { 20 };

        let cards = self
            .cards
            .clone()
            .get(self.top_idx..self.top_idx + deal_amount)
            .expect("deal_amount and top_idx are always set so this is valid")
            .to_vec();

        self.top_idx += deal_amount;
        self.times_dealt += 1;
        cards
    }

    pub fn reset<R: Rng>(&mut self, rng: &mut R) {
        self.cards.shuffle(rng);
        self.top_idx = 0;
        self.times_dealt = 0;
    }
}
