use itertools::Itertools;
use tac_types::{Card, Color, Square, TacAction, TacMove};

use crate::board::Board;

impl Board {
    #[must_use]
    pub fn get_moves(&self, player: Color) -> Vec<TacMove> {
        let mut moves = Vec::new();
        let hand = self.hand(player);
        // If in trade phase trade move for every card in hand
        if self.need_trade() {
            for card in hand.iter().sorted().dedup() {
                moves.push(TacMove::new(*card, TacAction::Trade, player));
            }
            return moves;
        }

        // If we are forced to discard, either respond with tac or discard any card in hand
        if self.force_discard() {
            if hand.iter().any(|c| matches!(c, Card::Tac)) {
                moves.extend(self.tac_moves(self.play_for(player)));
            }
            for card in hand.iter().sorted().dedup() {
                moves.push(TacMove::new(*card, TacAction::Discard, player));
            }
            return moves;
        }

        // Compute moves for each card in hand
        for card in hand.iter().sorted().dedup() {
            moves.extend(self.moves_for_card(player, *card));
        }

        // We can't do anything so discard any card
        if moves.is_empty() {
            for card in hand.iter().sorted().dedup() {
                moves.push(TacMove::new(*card, TacAction::Discard, player));
            }
        }

        moves
    }

    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn moves_for_card(&self, player: Color, card: Card) -> Vec<TacMove> {
        let play_for = self.play_for(player);
        let play_for_next = self.play_for(player.next());
        let mut moves = Vec::new();

        match card {
            Card::One | Card::Thirteen => {
                // If we still have balls in base, we can put them on the board
                if self.num_base(play_for) > 0 {
                    moves.push(TacMove::new(card, TacAction::Enter, play_for));
                }
            }
            Card::Seven => {
                // NOTE The number of possible seven moves scales extremely unwell for 3 (~7^2) and 4 (~7^3) moveable balls
                // Consider special casing them so move evaluation can prune them effectively with expert knowledge
                if (!self.home(play_for).is_empty() && !self.home(play_for).is_locked())
                    || self.can_play(play_for)
                {
                    moves.extend(self.seven_moves(play_for));
                }
            }
            Card::Eight => {
                if self.can_play(play_for) && !self.hand(player.next()).is_empty() {
                    moves.push(TacMove::new(card, TacAction::Suspend, player));
                }
            }
            Card::Trickster => {
                if self.can_play(play_for) {
                    moves.extend(self.trickster_moves(play_for));
                }
            }
            Card::Jester => {
                moves.push(TacMove::new(card, TacAction::Jester, player));
            }
            Card::Angel => {
                // If player after us still has balls out of play
                if self.num_base(play_for_next) > 0 {
                    moves.push(TacMove::new(card, TacAction::AngelEnter, play_for_next));
                } else {
                    for ball in self.balls_with(play_for_next) {
                        moves.extend(
                            self.moves_for_card_square(ball, play_for_next, Card::One)
                                .iter()
                                .map(|e| TacMove {
                                    card: Card::Angel,
                                    action: e.action.clone(),
                                    played_for: play_for_next,
                                })
                                .collect_vec(),
                        );
                        moves.extend(
                            self.moves_for_card_square(ball, play_for_next, Card::Thirteen)
                                .iter()
                                .map(|e| TacMove {
                                    card: Card::Angel,
                                    action: e.action.clone(),
                                    played_for: play_for_next,
                                })
                                .collect_vec(),
                        );
                    }
                }
            }
            Card::Devil => {
                moves.push(TacMove::new(card, TacAction::Devil, play_for));
            }
            Card::Tac => {
                moves.extend(self.tac_moves(play_for));
            }
            _ => {}
        }

        // Moves for balls that are not locked in their home
        // Uses matching on the bit patterns that correspond to states in which there are unlocked balls
        // with enough space to move the desired amount
        if !self.home(play_for).is_locked() {
            let home = self.home(play_for);
            match card {
                Card::One => match home.0 {
                    0b0001 | 0b1001 | 0b1101 => moves.push(TacMove::new(
                        card,
                        TacAction::StepHome { from: 0, to: 1 },
                        play_for,
                    )),
                    0b0010 | 0b1010 | 0b0011 | 0b1011 => moves.push(TacMove::new(
                        card,
                        TacAction::StepHome { from: 1, to: 2 },
                        play_for,
                    )),
                    0b0100 | 0b0110 | 0b0111 => moves.push(TacMove::new(
                        card,
                        TacAction::StepHome { from: 2, to: 3 },
                        play_for,
                    )),
                    0b0101 => {
                        moves.push(TacMove::new(
                            card,
                            TacAction::StepHome { from: 0, to: 1 },
                            play_for,
                        ));
                        moves.push(TacMove::new(
                            card,
                            TacAction::StepHome { from: 2, to: 3 },
                            play_for,
                        ));
                    }
                    _ => {}
                },
                Card::Two => match home.0 {
                    0b0001 | 0b1001 => {
                        moves.push(TacMove::new(
                            card,
                            TacAction::StepHome { from: 0, to: 2 },
                            play_for,
                        ));
                    }
                    0b0010 | 0b0011 => {
                        moves.push(TacMove::new(
                            card,
                            TacAction::StepHome { from: 1, to: 3 },
                            play_for,
                        ));
                    }
                    _ => {}
                },
                Card::Three => {
                    if home.0 == 0b0001 {
                        moves.push(TacMove::new(
                            card,
                            TacAction::StepHome { from: 0, to: 3 },
                            play_for,
                        ));
                    }
                }
                _ => {}
            }
        }

        // Moves we can only do with balls on the board
        if self.can_play(play_for) {
            for ball in self.balls_with(play_for) {
                moves.extend(self.moves_for_card_square(ball, play_for, card));
            }
        }
        moves
    }

    #[must_use]
    pub fn moves_for_card_square(&self, start: Square, color: Color, card: Card) -> Vec<TacMove> {
        let mut moves = Vec::new();

        // Simple forward movement
        if let Some(amount) = card.is_simple() {
            if self.can_move(start, start.add(amount)) {
                moves.push(TacMove::new(
                    card,
                    TacAction::Step {
                        from: start,
                        to: start.add(amount),
                    },
                    color,
                ));
            }
            // Need to add here in case there is ball on home square
            if start.distance_to_home(color) < amount && self.can_move(start, color.home().add(1)) {
                // TODO Compute the range of possible value to reach the home beforehand, to reduce computation
                if let Some(goal_pos) = self.position_in_home(start, amount, color) {
                    moves.push(TacMove::new(
                        card,
                        TacAction::StepInHome {
                            from: start,
                            to: goal_pos,
                        },
                        color,
                    ));
                }
            }
        }

        match card {
            Card::Four => {
                // Each of the four positions behind us are not occupied
                if (1..5).all(|i| !self.occupied(start.sub(i))) {
                    moves.push(TacMove::new(
                        card,
                        TacAction::Step {
                            from: start,
                            to: start.add(60),
                        },
                        color,
                    ));
                }

                // Minimum reverse dist to goal
                let min_rev_dist = 64 - start.distance_to_home(color) + 1;
                let free = self.home(color).free();

                // We are right infront of goal and moved in some way after entering play before
                if min_rev_dist == 65 && free == 4 && !self.fresh(color) {
                    moves.push(TacMove::new(
                        card,
                        TacAction::StepInHome { from: start, to: 4 },
                        color,
                    ));
                } else if free > 0 // Goal needs to be free
                    && min_rev_dist + free > 4 // Enough space to move in
                    && (2..=4).contains(&min_rev_dist) // In range to move in home
                    && (0..min_rev_dist - 1).all(|i| !self.occupied(color.home().add(i)))
                {
                    let goal = 4 - min_rev_dist;
                    moves.push(TacMove::new(
                        card,
                        TacAction::StepInHome {
                            from: start,
                            to: goal,
                        },
                        color,
                    ));
                }
            }
            Card::Warrior => {
                moves.push(TacMove::new(
                    card,
                    TacAction::Step {
                        from: start,
                        to: self.warrior_target(start, color),
                    },
                    color,
                ));
            }
            _ => {}
        }
        moves
    }

    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn trickster_moves(&self, play_for: Color) -> Vec<TacMove> {
        // At most n choose 2 -> n * (n-1) / 2
        // This only gets called if there are balls on the board so the length can never be 0
        let mut moves =
            Vec::with_capacity((self.all_balls().len() * (self.all_balls().len() - 1)) / 2);
        let mut same_switch = [false; 4];
        let mut home_switch = [false; 4];
        for (idx, target1) in self.all_balls().iter().enumerate() {
            let c1 = self
                .color_on(target1)
                .expect("Square value from all_balls means it's occupied");
            for target2 in self.all_balls().iter().skip(idx + 1) {
                let c2 = self
                    .color_on(target2)
                    .expect("Square value from all_balls means it's occupied");
                // Check if we can prune this move in case we already have one
                // that results in the same game state
                if c1 == c2 {
                    if c1.home() == target1 || c1.home() == target2 {
                        if home_switch[c1 as usize] {
                            // Already have one switching moves with same color on home square
                            continue;
                        }
                        home_switch[c1 as usize] = true;
                    } else if same_switch[c1 as usize] {
                        // Already have one switching moves with same color
                        continue;
                    } else {
                        same_switch[c1 as usize] = true;
                    }
                }
                moves.push(TacMove::new(
                    Card::Trickster,
                    TacAction::Trickster { target1, target2 },
                    play_for,
                ));
            }
        }
        moves
    }

    #[must_use]
    /// # Panics
    /// If the given square is not occupied by the given color
    pub fn warrior_target(&self, start: Square, player: Color) -> Square {
        debug_assert!(self.color_on(start).expect("Should work") == player);
        let others = self.all_balls() ^ start.bitboard();
        // Only ball on field
        if others.is_empty() {
            return start;
        }

        // We know there is at least another ball
        others.rotate_right(start.0).next_square().add(start.0)
    }

    #[must_use]
    pub fn tac_moves(&self, player: Color) -> Vec<TacMove> {
        let mut moves = Vec::new();

        if let Some((last_move, _)) = self.past_moves().iter().rev().find(|&(c, _)| {
            !(matches!(c.card, Card::Tac) || (matches!(c.card, Card::Jester) && self.jester_flag()))
        }) {
            let mut state = self.clone();
            state.tac_undo();
            moves.extend(
                state
                    .moves_for_card(player, last_move.card)
                    .iter()
                    .map(|m| TacMove::new(Card::Tac, m.action.clone(), player))
                    .collect_vec(),
            );
        }

        moves
    }
}

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use tac_types::ALL_COLORS;

    use super::*;

    #[test]
    fn seven_moves() {
        let mut board = Board::new();
        let player = Color::Black;
        board.put_ball_in_play(player);
        let moves = board.seven_moves(player);

        assert_eq!(moves.len(), 1);
        board.move_ball(Square(0), Square(7), player);
        board.put_ball_in_play(player);
        let moves = board.seven_moves(player);

        assert_eq!(moves.len(), 8);
        board.move_ball(Square(7), Square(14), player);
        board.move_ball(Square(0), Square(7), player);
        board.put_ball_in_play(player);
        let moves = board.seven_moves(player);

        assert_eq!(moves.len(), 36);
        board.move_ball(Square(14), Square(21), player);
        board.move_ball(Square(7), Square(14), player);
        board.move_ball(Square(0), Square(7), player);
        board.put_ball_in_play(player);
        let moves = board.seven_moves(player);

        assert_eq!(moves.len(), 120);
    }

    #[test]
    fn switching_moves() {
        let mut board = Board::new();
        for color in ALL_COLORS {
            board.put_ball_in_play(color);
        }
        let moves = board.trickster_moves(Color::Black);
        assert_eq!(moves.len(), 6);
        board.move_ball(Square(0), Square(4), Color::Black);
        board.put_ball_in_play(Color::Black);
        board.move_ball(Square(0), Square(8), Color::Black);
        board.put_ball_in_play(Color::Black);
        // If we have multiple balls of the same color we can deduplicate the moves between them.
        // There are only 2 unique possibilities, either one ball is on home square (makes ball not fresh) or both are in ring.
        // So for each color we know the amount of moves we can prune is:
        // same_color_cnt * (same_color_cnt - 1) / 2 - 2
        let moves = board.trickster_moves(Color::Black);
        // 3 * 2 / 2 - 2 = 1
        assert_eq!(moves.len(), 15 - 1);
        board.move_ball(Square(0), Square(12), Color::Black);
        board.put_ball_in_play(Color::Black);
        let moves = board.trickster_moves(Color::Black);
        // 4 * 3 / 2 - 2 = 4
        assert_eq!(moves.len(), 21 - 4);
        for c in [Color::Blue, Color::Green, Color::Red] {
            board.move_ball(c.home(), c.home().add(4), c);
            board.put_ball_in_play(c);
            board.move_ball(c.home(), c.home().add(8), c);
            board.put_ball_in_play(c);
            board.move_ball(c.home(), c.home().add(12), c);
            board.put_ball_in_play(c);
        }
        assert_eq!(board.all_balls().len(), 16);
        let moves = board.trickster_moves(Color::Black);
        assert_eq!(moves.len(), (16 * 15 / 2) - 4 * (4 * 3 / 2 - 2));
    }

    #[test]
    fn four() {
        let mut board = Board::new();
        board.put_ball_in_play(Color::Black);
        let moves = board.moves_for_card_square(Square(0), Color::Black, Card::Four);
        assert_eq!(moves.len(), 1);
        assert_eq!(
            moves[0],
            TacMove::new(
                Card::Four,
                TacAction::Step {
                    from: Square(0),
                    to: Square(60)
                },
                Color::Black,
            )
        );
        board.move_ball(Square(0), Square(4), Color::Black);
        assert_eq!(board.color_on(Square(4)), Some(Color::Black));
        let moves = board.moves_for_card_square(Square(4), Color::Black, Card::Four);
        assert_eq!(moves.len(), 1);
        assert_eq!(
            moves[0],
            TacMove::new(
                Card::Four,
                TacAction::Step {
                    from: Square(4),
                    to: Square(0)
                },
                Color::Black,
            )
        );
    }

    #[test]
    fn four_in_goal() {
        let mut board = Board::new();
        let black = Color::Black;
        board.put_ball_in_play(black);
        board.move_ball(Square(0), Square(1), black);
        board.apply_action(
            board.moves_for_card_square(Square(1), black, Card::Four)[1]
                .action
                .clone(),
            black,
        );
        assert!(board.home(black).is_free(0));
        assert!(board.home(black).is_free(1));
        assert!(!board.home(black).is_free(2));
        assert!(board.home(black).is_free(3));
        board.put_ball_in_play(black);
        board.move_ball(Square(0), Square(1), black);
        assert_eq!(
            board
                .moves_for_card_square(Square(1), black, Card::Four)
                .len(),
            1
        );
        board.move_ball(Square(1), Square(3), black);
        board.apply_action(
            board.moves_for_card_square(Square(3), black, Card::Four)[1]
                .action
                .clone(),
            black,
        );
        assert!(!board.home(black).is_free(0));
        assert!(board.home(black).is_free(1));
        assert!(!board.home(black).is_free(2));
        assert!(board.home(black).is_free(3));
    }

    #[test]
    fn warrior() {
        let mut board = Board::new();
        board.put_ball_in_play(Color::Black);
        board.put_ball_in_play(Color::Red);

        let moves = board.moves_for_card(Color::Black, Card::Warrior);
        assert_eq!(moves.len(), 1);
        assert_eq!(
            moves[0],
            TacMove::new(
                Card::Warrior,
                TacAction::Step {
                    from: Color::Black.home(),
                    to: Color::Red.home()
                },
                Color::Black
            ),
        );
        board.apply_action(moves[0].action.clone(), Color::Black);
        board.set_player(Color::Black);
        let moves = board.moves_for_card(Color::Black, Card::Warrior);
        assert_eq!(moves.len(), 1);
        assert_eq!(
            moves[0],
            TacMove::new(
                Card::Warrior,
                TacAction::Step {
                    from: Color::Red.home(),
                    to: Color::Red.home()
                },
                Color::Black
            )
        );
    }

    #[test]
    fn tac() {
        let mut board = Board::new();
        ALL_COLORS
            .iter()
            .for_each(|c| board.add_hand(*c, Card::Tac));
        board.add_hand(Color::Black, Card::One);
        let mv = TacMove::new(Card::One, TacAction::Enter, Color::Black);
        board.play(&mv);
        assert_eq!(board.color_on(Color::Black.home()).unwrap(), Color::Black);
        assert_eq!(board.current_player(), Color::Blue);
        let moves = board.moves_for_card(board.current_player(), Card::Tac);
        assert_eq!(moves.len(), 1);
        board.play(&moves[0]);
        assert_eq!(board.current_player(), Color::Green);
        assert_eq!(board.color_on(Color::Black.home()), None);
        assert_eq!(board.color_on(Color::Blue.home()).unwrap(), Color::Blue);
        let moves = board.moves_for_card(board.current_player(), Card::Tac);
        assert_eq!(moves.len(), 1);
        board.play(&moves[0]);
        assert_eq!(board.current_player(), Color::Red);
        assert_eq!(board.color_on(Color::Black.home()).unwrap(), Color::Black);
        assert_eq!(board.color_on(Color::Blue.home()), None);
        assert_eq!(board.color_on(Color::Green.home()).unwrap(), Color::Green);
        let moves = board.moves_for_card(board.current_player(), Card::Tac);
        assert_eq!(moves.len(), 1);
        board.play(&moves[0]);
        assert_eq!(board.current_player(), Color::Black);
        assert_eq!(board.color_on(Color::Black.home()), None);
        assert_eq!(board.color_on(Color::Blue.home()).unwrap(), Color::Blue);
        assert_eq!(board.color_on(Color::Green.home()), None);
        assert_eq!(board.color_on(Color::Red.home()).unwrap(), Color::Red);
    }
}
