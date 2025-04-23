use smallvec::SmallVec;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Home(pub u8);

impl Home {
    pub const EMPTY: Self = Self(0);
    const FULL: Self = Self(15);

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == Self::EMPTY.0
    }

    pub fn xor(&mut self, pos: u8) {
        self.0 ^= 1 << pos;
    }

    pub fn set(&mut self, pos: u8) {
        assert!(self.is_free(pos));
        self.xor(pos);
    }

    pub fn unset(&mut self, pos: u8) {
        assert!(!self.is_free(pos));
        self.xor(pos);
    }

    #[must_use]
    pub const fn free(self) -> u8 {
        (self.0 | 0b10000).trailing_zeros() as u8
    }

    #[must_use]
    pub const fn is_free(self, pos: u8) -> bool {
        (self.0 & (1 << pos)) == 0
    }

    #[must_use]
    pub const fn amount(self) -> u8 {
        self.0.count_ones() as u8
    }

    #[must_use]
    pub const fn is_locked(self) -> bool {
        self.0 == 0b1000 || self.0 == 0b1100 || self.0 == 0b1110 || self.0 == 0b1111
    }

    #[must_use]
    pub const fn is_full(self) -> bool {
        self.0 == Self::FULL.0
    }

    #[must_use]
    pub const fn get_single_unlocked(self) -> Option<u8> {
        if !self.is_locked() && !self.is_empty() {
            return Some(self.free());
        }
        None
    }

    #[must_use]
    pub fn get_all_unlocked(self) -> SmallVec<u8, 3> {
        let mut home = self;
        let mut res = SmallVec::new();
        while let Some(unlocked) = home.get_single_unlocked() {
            res.push(unlocked);
            home.xor(unlocked);
        }
        res
    }

    pub fn can_move(self) -> bool {
        !(self.is_locked() || self.is_empty())
    }
}

impl Display for Home {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#06b}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_single_unlocked() {
        let mut home = Home::EMPTY;

        assert_eq!(home.get_single_unlocked(), None);
        home.xor(0);
        assert_eq!(home.get_single_unlocked(), Some(0));
        home.xor(3);
        assert_eq!(home.get_single_unlocked(), Some(0));
        home.xor(0);
        assert_eq!(home.get_single_unlocked(), None);
        home.xor(2);
        assert_eq!(home.get_single_unlocked(), None);
        home.xor(1);
        home.xor(2);
        assert_eq!(home.get_single_unlocked(), Some(1));
    }

    #[test]
    fn get_all_unlocked() {
        let mut home = Home::EMPTY;
        home.xor(2);
        home.xor(0);
        assert_eq!(home.get_all_unlocked().into_vec(), vec![0, 2]);
    }
}
