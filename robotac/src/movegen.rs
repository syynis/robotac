use itertools::Itertools;
use tac_types::{Card, Color, Square, TacAction, TacMove};

use crate::{board::Board, hand::Hand};

impl Board {
    pub fn get_moves(&self, player: Color, hand: Hand) -> Vec<TacMove> {
        let mut moves = Vec::new();
        let balls = self.balls_with(player);

        for card in hand.iter().sorted().dedup() {
            // If player before us played an eight to suspend we have to discard
            if self.force_discard() {
                if matches!(card, Card::Tac) {
                    moves.push(TacMove {
                        card: Card::Tac,
                        action: TacAction::Suspend,
                    });
                }
                moves.push(TacMove::new(*card, TacAction::Discard));
                continue;
            }
            // If we still have balls outside of play, we can put them on the board
            if matches!(card, Card::One | Card::Thirteen) && self.num_outside(player) > 0 {
                moves.push(TacMove::new(*card, TacAction::Enter));
            }
            // Master cards
            if matches!(card, Card::Jester) {
                moves.push(TacMove::new(*card, TacAction::Jester));
            }
            if matches!(card, Card::Devil) {
                moves.push(TacMove::new(*card, TacAction::Devil));
            }

            if matches!(card, Card::Angel) {
                // If player after us still has balls out of play
                if self.num_outside(player.next()) > 0 {
                    moves.push(TacMove::new(*card, TacAction::AngelEnter));
                } else {
                    for ball in self.balls_with(player.next()) {
                        moves.extend(self.moves_for_card(ball, player.next(), Card::One));
                        moves.extend(self.moves_for_card(ball, player.next(), Card::Thirteen));
                    }
                }
            }

            if matches!(card, Card::Tac) {
                moves.extend(self.handle_tac(player));
            }

            // NOTE The number of possible seven moves scales extremely unwell for 3 (~7^2) and 4 (~7^3) moveable balls
            // Consider special casing them so move evaluation can prune them effectively with expert knowledge
            if (!self.home(player).is_locked() || self.num_outside(player) > 0)
                && matches!(card, Card::Seven)
            {
                moves.extend(self.seven_moves(player));
            }

            // Moves for balls that are not locked in their home
            // Uses matching on the bit patterns that correspond to states in which there are unlocked balls
            // with enough space to move the desired amount
            if !self.home(player).is_locked() {
                let home = self.home(player);
                match card {
                    Card::One => match home.0 {
                        0b0001 | 0b1001 | 0b1101 => {
                            moves.push(TacMove::new(*card, TacAction::StepHome { from: 0, to: 1 }))
                        }
                        0b0010 | 0b1010 | 0b0011 | 0b1011 => {
                            moves.push(TacMove::new(*card, TacAction::StepHome { from: 1, to: 2 }))
                        }
                        0b0100 | 0b0110 | 0b0111 => {
                            moves.push(TacMove::new(*card, TacAction::StepHome { from: 2, to: 3 }))
                        }
                        0b0101 => {
                            moves.push(TacMove::new(*card, TacAction::StepHome { from: 0, to: 1 }));
                            moves.push(TacMove::new(*card, TacAction::StepHome { from: 2, to: 3 }));
                        }
                        _ => {}
                    },
                    Card::Two => match home.0 {
                        0b0001 => {
                            moves.push(TacMove::new(*card, TacAction::StepHome { from: 0, to: 2 }));
                        }
                        0b0010 | 0b0011 => {
                            moves.push(TacMove::new(*card, TacAction::StepHome { from: 1, to: 3 }));
                        }
                        0b1001 => {
                            moves.push(TacMove::new(*card, TacAction::StepHome { from: 0, to: 2 }));
                        }
                        _ => {}
                    },
                    Card::Three => {
                        if home.0 == 0b0001 {
                            moves.push(TacMove::new(*card, TacAction::StepHome { from: 0, to: 3 }));
                        }
                    }
                    _ => {}
                }
            }
            // Moves we can only do with balls on the board
            if !self.balls_with(player).is_empty() {
                if matches!(card, Card::Juggler) {
                    moves.extend(self.switching_moves());
                } else {
                    if matches!(card, Card::Eight) {
                        moves.push(TacMove::new(*card, TacAction::Suspend));
                    }
                    for ball in balls.iter() {
                        moves.extend(self.moves_for_card(ball, player, *card));
                    }
                }
            }
        }

        moves
    }

    pub fn moves_for_card(&self, start: Square, player: Color, card: Card) -> Vec<TacMove> {
        let mut moves = Vec::new();

        // Simple forward movement
        if let Some(amount) = card.is_simple() {
            if self.can_move(start, start.add(amount), false) {
                moves.push(TacMove::new(
                    card,
                    TacAction::Step {
                        from: start,
                        to: start.add(amount),
                    },
                ));
            }
        }

        match card {
            Card::Four => {
                // We add here because it's easier.
                // 64 positions with wrapping so +60 is the same as -4
                if self.can_move(start.add(60), start, true) {
                    moves.push(TacMove::new(
                        card,
                        TacAction::Step {
                            from: start,
                            to: start.add(60),
                        },
                    ));
                }
            }
            Card::Warrior => {
                moves.push(TacMove::new(
                    card,
                    TacAction::Step {
                        from: start,
                        to: self.warrior_target(start, player),
                    },
                ));
            }
            _ => {}
        }
        moves
    }

    pub fn switching_moves(&self) -> Vec<TacMove> {
        // n choose 2 -> n * (n-1) / 2
        // This only gets called if there are balls on the board so the length can never be 0
        let mut moves = Vec::with_capacity(
            (self.all_balls().len() * (self.all_balls().len() - 1)) as usize / 2,
        );
        for (idx, target1) in self.all_balls().iter().enumerate() {
            for target2 in self.all_balls().iter().skip(idx + 1) {
                moves.push(TacMove::new(
                    Card::Juggler,
                    TacAction::Switch { target1, target2 },
                ))
            }
        }
        moves
    }

    pub fn warrior_target(&self, start: Square, player: Color) -> Square {
        assert!(self.color_on(start).expect("Should work") == player);
        let others = self.all_balls() ^ start.bitboard();
        // Only ball on field
        if others.is_empty() {
            return start;
        }

        others
            .rotate_right(start.0)
            .next_square()
            .expect("We know there is at least another balls")
    }

    pub fn handle_tac(&self, _player: Color) -> Vec<TacMove> {
        let moves = Vec::new();
        let _last = self.last_played();
        // TODO
        moves
    }

    pub fn seven_moves(&self, player: Color) -> Vec<TacMove> {
        // TODO Some thoughts about generating seven moves
        // This still needs to take into account moves in the house
        // With one unfixed ball in the house, either walk one (three, five, seven)
        // in either direction or two (four, six)
        // More than one unfixed ball -> ???
        // Self capture should never be an issue because we can always do the moves
        // for the captured ball first
        let mut moves = Vec::new();
        let num_balls = self.balls_with(player).len();
        let balls = self.balls_with(player).iter().collect_vec();
        match num_balls {
            1 => moves.push(TacMove::new(
                Card::Seven,
                TacAction::Step {
                    from: balls[0],
                    to: balls[0].add(7),
                },
            )),
            2 => {
                for i in 0..8 {
                    let j = 7 - i;
                    moves.push(TacMove::new(
                        Card::Seven,
                        TacAction::Step {
                            from: balls[0],
                            to: balls[0].add(i),
                        },
                    ));
                    moves.push(TacMove::new(
                        Card::Seven,
                        TacAction::Step {
                            from: balls[1],
                            to: balls[1].add(j),
                        },
                    ));
                }
            }
            3 => {
                for i in 0..8 {
                    for j in 0..(8 - i) {
                        let k = 7 - i - j;
                        moves.push(TacMove::new(
                            Card::Seven,
                            TacAction::Step {
                                from: balls[0],
                                to: balls[0].add(i),
                            },
                        ));
                        moves.push(TacMove::new(
                            Card::Seven,
                            TacAction::Step {
                                from: balls[1],
                                to: balls[1].add(j),
                            },
                        ));
                        moves.push(TacMove::new(
                            Card::Seven,
                            TacAction::Step {
                                from: balls[2],
                                to: balls[2].add(k),
                            },
                        ));
                    }
                }
            }
            4 => {
                for i in 0..8 {
                    for j in 0..(8 - i) {
                        for k in 0..(8 - i - j) {
                            let l = 7 - i - j - k;
                            moves.push(TacMove::new(
                                Card::Seven,
                                TacAction::Step {
                                    from: balls[0],
                                    to: balls[0].add(i),
                                },
                            ));
                            moves.push(TacMove::new(
                                Card::Seven,
                                TacAction::Step {
                                    from: balls[1],
                                    to: balls[1].add(j),
                                },
                            ));
                            moves.push(TacMove::new(
                                Card::Seven,
                                TacAction::Step {
                                    from: balls[2],
                                    to: balls[2].add(k),
                                },
                            ));
                            moves.push(TacMove::new(
                                Card::Seven,
                                TacAction::Step {
                                    from: balls[3],
                                    to: balls[3].add(l),
                                },
                            ));
                        }
                    }
                }
            }
            _ => unreachable!(),
        }
        moves
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seven_moves() {
        let mut board = Board::default();
        let player = Color::Black;
        board.put_ball_in_play(player);
        let moves = board.seven_moves(player);

        assert_eq!(moves.len(), 1);
        board.move_ball(Square(0), Square(7), player);
        board.put_ball_in_play(player);
        let moves = board.seven_moves(player);

        assert_eq!(moves.len(), 2 * 8);
        board.move_ball(Square(7), Square(14), player);
        board.move_ball(Square(0), Square(7), player);
        board.put_ball_in_play(player);
        let moves = board.seven_moves(player);

        assert_eq!(moves.len(), 3 * 36);
    }

    #[test]
    fn switching_moves() {
        let mut board = Board::default();
        for color in [Color::Black, Color::Blue, Color::Green, Color::Red] {
            board.put_ball_in_play(color);
        }
        let moves = board.switching_moves();
        assert_eq!(moves.len(), 6);
    }

    #[test]
    fn four() {
        let mut board = Board::default();
        board.put_ball_in_play(Color::Black);
        let moves = board.moves_for_card(Square(0), Color::Black, Card::Four);
        assert_eq!(moves.len(), 1);
        assert_eq!(
            moves[0],
            TacMove::new(
                Card::Four,
                TacAction::Step {
                    from: Square(0),
                    to: Square(60)
                }
            )
        );
        board.move_ball(Square(0), Square(4), Color::Black);
        let moves = board.moves_for_card(Square(4), Color::Black, Card::Four);
        assert_eq!(moves.len(), 1);
        assert_eq!(
            moves[0],
            TacMove::new(
                Card::Four,
                TacAction::Step {
                    from: Square(4),
                    to: Square(0)
                }
            )
        );
    }
}
