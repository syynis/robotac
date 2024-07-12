#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Home(pub u8);

impl Home {
    pub const EMPTY: Self = Self(0);
    const FULL: Self = Self(15);

    pub fn is_empty(&self) -> bool {
        *self == Self::EMPTY
    }
    pub fn xor(&mut self, pos: u8) {
        self.0 ^= 1 << pos;
    }

    pub fn free(&self) -> u8 {
        self.0.trailing_zeros() as u8
    }

    pub fn len(&self) -> u8 {
        self.0.count_ones() as u8
    }

    pub fn is_locked(&self) -> bool {
        self.0 == 8 || self.0 == 12 || self.0 == 14 || self.0 == 15
    }

    pub fn is_full(&self) -> bool {
        *self == Self::FULL
    }

    pub fn get_single_unlocked(&self) -> Option<u8> {
        if !self.is_locked() {
            let unlocked = self.0.trailing_zeros() as u8;
            let without_unlocked = Home(self.0 ^ (1 << unlocked));
            if !without_unlocked.is_locked() {
                return Some(unlocked);
            }
        }
        None
    }
}
