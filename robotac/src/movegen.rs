use itertools::Itertools;
use tac_types::{Card, Color, Home, Square, TacAction, TacMove};

use crate::{board::Board, hand::Hand};

impl Board {
    pub fn get_moves(&self, player: Color, hand: &Hand) -> Vec<TacMove> {
        let mut moves = Vec::new();

        // If in trade phase trade move for every card in hand
        if self.need_trade() {
            for card in hand.iter().sorted().dedup() {
                moves.push(TacMove::new(*card, TacAction::Trade));
            }
            return moves;
        }

        // If we are forced to discard, either respond with tac or discard any card in hand
        if self.force_discard() {
            if hand.iter().any(|c| matches!(c, Card::Tac)) {
                moves.extend(self.tac_moves(player));
            }
            for card in hand.iter().sorted().dedup() {
                moves.push(TacMove::new(*card, TacAction::Discard));
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
                moves.push(TacMove::new(*card, TacAction::Discard));
            }
        }

        moves
    }

    pub fn moves_for_card(&self, player: Color, card: Card) -> Vec<TacMove> {
        let mut moves = Vec::new();

        match card {
            Card::One | Card::Thirteen => {
                // If we still have balls outside of play, we can put them on the board
                if self.num_outside(player) > 0 {
                    moves.push(TacMove::new(card, TacAction::Enter));
                }
            }
            Card::Seven => {
                // NOTE The number of possible seven moves scales extremely unwell for 3 (~7^2) and 4 (~7^3) moveable balls
                // Consider special casing them so move evaluation can prune them effectively with expert knowledge
                if !self.home(player).is_locked() || self.can_play(player) {
                    moves.extend(self.seven_moves(player));
                }
            }
            Card::Eight => {
                if self.can_play(player) {
                    moves.push(TacMove::new(card, TacAction::Suspend));
                }
            }
            Card::Juggler => {
                if self.can_play(player) {
                    moves.extend(self.switching_moves());
                }
            }
            Card::Jester => {
                moves.push(TacMove::new(card, TacAction::Jester));
            }
            Card::Angel => {
                // If player after us still has balls out of play
                if self.num_outside(player.next()) > 0 {
                    moves.push(TacMove::new(card, TacAction::AngelEnter));
                } else {
                    for ball in self.balls_with(player.next()) {
                        moves.extend(self.moves_for_card_square(ball, player.next(), Card::One));
                        moves.extend(self.moves_for_card_square(
                            ball,
                            player.next(),
                            Card::Thirteen,
                        ));
                    }
                }
            }
            Card::Devil => {
                moves.push(TacMove::new(card, TacAction::Devil));
            }
            Card::Tac => {
                moves.extend(self.tac_moves(player));
            }
            _ => {}
        }

        // Moves for balls that are not locked in their home
        // Uses matching on the bit patterns that correspond to states in which there are unlocked balls
        // with enough space to move the desired amount
        if !self.home(player).is_locked() {
            let home = self.home(player);
            match card {
                Card::One => match home.0 {
                    0b0001 | 0b1001 | 0b1101 => {
                        moves.push(TacMove::new(card, TacAction::StepHome { from: 0, to: 1 }))
                    }
                    0b0010 | 0b1010 | 0b0011 | 0b1011 => {
                        moves.push(TacMove::new(card, TacAction::StepHome { from: 1, to: 2 }))
                    }
                    0b0100 | 0b0110 | 0b0111 => {
                        moves.push(TacMove::new(card, TacAction::StepHome { from: 2, to: 3 }))
                    }
                    0b0101 => {
                        moves.push(TacMove::new(card, TacAction::StepHome { from: 0, to: 1 }));
                        moves.push(TacMove::new(card, TacAction::StepHome { from: 2, to: 3 }));
                    }
                    _ => {}
                },
                Card::Two => match home.0 {
                    0b0001 => {
                        moves.push(TacMove::new(card, TacAction::StepHome { from: 0, to: 2 }));
                    }
                    0b0010 | 0b0011 => {
                        moves.push(TacMove::new(card, TacAction::StepHome { from: 1, to: 3 }));
                    }
                    0b1001 => {
                        moves.push(TacMove::new(card, TacAction::StepHome { from: 0, to: 2 }));
                    }
                    _ => {}
                },
                Card::Three => {
                    if home.0 == 0b0001 {
                        moves.push(TacMove::new(card, TacAction::StepHome { from: 0, to: 3 }));
                    }
                }
                _ => {}
            }
        }

        // Moves we can only do with balls on the board
        if self.can_play(player) {
            for ball in self.balls_with(player).iter() {
                moves.extend(self.moves_for_card_square(ball, player, card));
            }
        }
        moves
    }

    pub fn moves_for_card_square(&self, start: Square, player: Color, card: Card) -> Vec<TacMove> {
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
                ));
            }
            if start.distance_to_home(player) < amount && self.can_move(start, player.home()) {
                // TODO Compute the range of possible value to reach the home beforehand, to reduce computation
                if let Some(goal_pos) = self.position_in_home(start, amount, player) {
                    moves.push(TacMove::new(
                        card,
                        TacAction::StepInHome {
                            from: start,
                            to: goal_pos,
                        },
                    ))
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
                    ));
                }

                // Minimum reverse dist to goal
                let min_rev_dist = 64 - start.distance_to_home(player) + 1;
                let free = self.home(player).free();

                // We are right infront of goal and moved in some way after entering play before
                if min_rev_dist == 1 && free == 4 && !self.fresh(player) {
                    moves.push(TacMove::new(
                        card,
                        TacAction::StepInHome { from: start, to: 4 },
                    ));
                } else if free > 0 // Goal needs to be free
                    && min_rev_dist < 5 // At most 4 away from goal
                    && min_rev_dist > 1 // Not standing on home
                    && min_rev_dist + free > 3 // Enough space to move in
                    && (0..min_rev_dist - 1).all(|i| !self.occupied(player.home().add(i)))
                {
                    let goal = 4 - min_rev_dist;
                    moves.push(TacMove::new(
                        card,
                        TacAction::StepInHome {
                            from: start,
                            to: goal,
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
        // At most n choose 2 -> n * (n-1) / 2
        // This only gets called if there are balls on the board so the length can never be 0
        let mut moves = Vec::with_capacity(
            (self.all_balls().len() * (self.all_balls().len() - 1)) as usize / 2,
        );
        let mut same_switch = [false; 4];
        let mut home_switch = [false; 4];
        for (idx, target1) in self.all_balls().iter().enumerate() {
            let c1 = self.color_on(target1).unwrap();
            for target2 in self.all_balls().iter().skip(idx + 1) {
                let c2 = self.color_on(target2).unwrap();
                // Check if we can prune this move in case we already have one
                // that results in the same game state
                if c1 == c2 {
                    if c1.home() == target1 || c1.home() == target2 {
                        if home_switch[c1 as usize] {
                            // Already have one switching moves with same color on home square
                            continue;
                        } else {
                            home_switch[c1 as usize] = true;
                        }
                    } else {
                        if same_switch[c1 as usize] {
                            // Already have one switching moves with same color
                            continue;
                        } else {
                            same_switch[c1 as usize] = true;
                        }
                    }
                }
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

        // We know there is at least another ball
        others.rotate_right(start.0).next_square().add(start.0)
    }

    pub fn tac_moves(&self, player: Color) -> Vec<TacMove> {
        let mut moves = Vec::new();

        if let Some((last_move, captured)) = self.past_moves().iter().rev().find(|&(c, _)| {
            !(matches!(c.card, Card::Tac) || (matches!(c.card, Card::Jester) && self.jester_flag()))
        }) {
            let mut state = self.clone();
            state.tac_undo();
            moves.extend(
                state
                    .moves_for_card(player, last_move.card)
                    .iter()
                    .map(|m| TacMove::new(Card::Tac, m.action.clone()))
                    .collect_vec(),
            );
        }

        moves
    }

    fn get_home_moves_with_budget(&self, home: &Home, budget: u8) -> Vec<Vec<(u8, u8)>> {
        let mut moves = Vec::new();
        let mut unlocked = home.get_all_unlocked();
        if budget == 0 || unlocked.is_empty() {
            return moves;
        }
        unlocked.reverse();
        let num_unlocked = unlocked.len();
        let even_budget = budget % 2 == 0;
        // Try to spend budget
        match num_unlocked {
            1 => {
                let pos = unlocked[0];
                if even_budget {
                    if home.free_after(pos) && home.free_after(pos + 1) {
                        moves.push(vec![(pos, pos + 2)]);
                    }
                    if home.free_behind(pos) && home.free_behind(pos - 1) {
                        moves.push(vec![(pos, pos - 2)])
                    }
                } else {
                    if budget > 2 && pos == 0 {
                        moves.push(vec![(pos, pos + 3)])
                    }
                    if home.free_after(pos) {
                        moves.push(vec![(pos, pos + 1)]);
                    }
                    if home.free_behind(pos) {
                        moves.push(vec![(pos, pos - 1)])
                    }
                }
            }
            2 => match home.0 {
                0b0110 => {
                    if even_budget {
                        moves.push(vec![(2, 3), (1, 2)]);
                        moves.push(vec![(2, 3), (1, 0)]);
                        moves.push(vec![(1, 0), (2, 1)]);
                    } else {
                        moves.push(vec![(2, 3)]);
                        moves.push(vec![(1, 0)]);
                    }
                }
                0b0101 => {
                    if even_budget {
                        moves.push(vec![(2, 3), (0, 1)])
                    } else {
                        moves.push(vec![(0, 1)]);
                        moves.push(vec![(2, 3)]);
                        moves.push(vec![(2, 1)]);
                        if budget > 1 {
                            moves.push(vec![(2, 3), (0, 2)]);
                        }
                    }
                }
                0b0011 => {
                    if even_budget {
                        moves.push(vec![(1, 3)]);
                        moves.push(vec![(1, 2), (0, 1)]);
                        if budget > 2 {
                            moves.push(vec![(1, 3), (0, 2)]);
                        }
                    } else {
                        moves.push(vec![(1, 2)]);

                        if budget > 1 {
                            moves.push(vec![(1, 3), (0, 1)]);
                        }
                    }
                }
                0b1011 => {
                    if even_budget {
                        moves.push(vec![(1, 2), (0, 1)]);
                    } else {
                        moves.push(vec![(1, 2)]);
                    }
                }
                _ => {}
            },
            3 => {
                if even_budget {
                    moves.push(vec![(2, 3), (1, 2)]);
                } else {
                    moves.push(vec![(2, 3)]);
                    if budget > 2 {
                        moves.push(vec![(2, 3), (1, 2), (0, 1)]);
                    }
                }
            }
            _ => unreachable!(),
        }
        moves
    }

    pub fn seven_moves(&self, player: Color) -> Vec<TacMove> {
        // TODO Some thoughts about generating seven moves
        // This still needs to take into account moves that go from ring to home
        let mut moves = Vec::new();
        let num_balls = self.balls_with(player).len();
        let home = self.home(player);
        let balls = self.balls_with(player).iter().collect_vec();
        let max_home = if !home.is_locked() && !home.is_empty() {
            8
        } else {
            1
        };
        for home_budget in 0..max_home {
            let home_moves = if home_budget != 0 {
                self.get_home_moves_with_budget(&home, home_budget)
                    .iter()
                    .map(|hm| {
                        hm.iter()
                            .map(|&(from, to)| TacAction::StepHome { from, to })
                            .collect_vec()
                    })
                    .collect_vec()
            } else {
                Vec::new()
            };
            let board_budget = 7 - home_budget;
            if home_budget == 7 {
                moves.extend(home_moves.iter().map(|steps| {
                    TacMove::new(
                        Card::Seven,
                        TacAction::SevenSteps {
                            steps: steps.to_vec(),
                        },
                    )
                }));
                break;
            }
            let mut steps = Vec::new();
            match num_balls {
                1 => {
                    steps.push(vec![TacAction::Step {
                        from: balls[0],
                        to: balls[0].add(board_budget),
                    }]);
                }
                2 => {
                    for i in 0..board_budget + 1 {
                        let j = board_budget - i;
                        steps.push(vec![
                            TacAction::Step {
                                from: balls[0],
                                to: balls[0].add(i),
                            },
                            TacAction::Step {
                                from: balls[1],
                                to: balls[1].add(j),
                            },
                        ]);
                    }
                }
                3 => {
                    for i in 0..board_budget + 1 {
                        for j in 0..board_budget + 1 - i {
                            let k = board_budget - i - j;
                            steps.push(vec![
                                TacAction::Step {
                                    from: balls[0],
                                    to: balls[0].add(i),
                                },
                                TacAction::Step {
                                    from: balls[1],
                                    to: balls[1].add(j),
                                },
                                TacAction::Step {
                                    from: balls[2],
                                    to: balls[2].add(k),
                                },
                            ]);
                        }
                    }
                }
                4 => {
                    for i in 0..board_budget + 1 {
                        for j in 0..board_budget + 1 - i {
                            for k in 0..board_budget + 1 - i - j {
                                let l = board_budget - i - j - k;
                                steps.push(vec![
                                    TacAction::Step {
                                        from: balls[0],
                                        to: balls[0].add(i),
                                    },
                                    TacAction::Step {
                                        from: balls[1],
                                        to: balls[1].add(j),
                                    },
                                    TacAction::Step {
                                        from: balls[2],
                                        to: balls[2].add(k),
                                    },
                                    TacAction::Step {
                                        from: balls[3],
                                        to: balls[3].add(l),
                                    },
                                ]);
                            }
                        }
                    }
                }
                _ => unreachable!(),
            }
            for step in steps {
                if home_moves.is_empty() {
                    moves.push(TacMove::new(
                        Card::Seven,
                        TacAction::SevenSteps {
                            steps: step.clone(),
                        },
                    ));
                }
                for home_move in home_moves.clone() {
                    moves.push(TacMove::new(
                        Card::Seven,
                        TacAction::SevenSteps {
                            steps: [home_move, step.clone()].concat(),
                        },
                    ));
                }
            }
        }
        moves
    }
}

#[cfg(test)]
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
        let moves = board.switching_moves();
        assert_eq!(moves.len(), 6);
        board.move_ball(Square(0), Square(4), Color::Black);
        board.put_ball_in_play(Color::Black);
        board.move_ball(Square(0), Square(8), Color::Black);
        board.put_ball_in_play(Color::Black);
        // If we have multiple balls of the same color we can deduplicate the moves between them.
        // There are only 2 unique possibilities, either one ball is on home square (makes ball not fresh) or both are in ring.
        // So for each color we know the amount of moves we can prune is:
        // same_color_cnt * (same_color_cnt - 1) / 2 - 2
        let moves = board.switching_moves();
        // 3 * 2 / 2 - 2 = 1
        assert_eq!(moves.len(), 15 - 1);
        board.move_ball(Square(0), Square(12), Color::Black);
        board.put_ball_in_play(Color::Black);
        let moves = board.switching_moves();
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
        let moves = board.switching_moves();
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
                }
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
                }
            )
        );
        board.put_ball_in_play(Color::Red);
        assert_eq!(board.color_on(Square(48)), Some(Color::Red));
        board.move_ball(Square(48), Square(3), Color::Red);
        let moves = board.moves_for_card_square(Square(4), Color::Black, Card::Four);
        assert_eq!(moves.len(), 0);
        board.move_ball(Square(3), Square(5), Color::Red);
        let moves = board.moves_for_card_square(Square(4), Color::Black, Card::Four);
        assert_eq!(moves.len(), 1);
        board.move_ball(Square(4), Square(3), Color::Black);
        let moves = board.moves_for_card_square(Square(3), Color::Black, Card::Four);
        assert_eq!(moves.len(), 2);
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
                }
            )
        );
        board.play(moves[0].clone());
        let moves = board.moves_for_card(Color::Black, Card::Warrior);
        assert_eq!(moves.len(), 1);
        assert_eq!(
            moves[0],
            TacMove::new(
                Card::Warrior,
                TacAction::Step {
                    from: Color::Red.home(),
                    to: Color::Red.home()
                }
            )
        );
    }

    #[test]
    fn tac() {
        let mut board = Board::new();
        let mv = TacMove {
            card: Card::One,
            action: TacAction::Enter,
        };
        board.play(mv);
        assert_eq!(
            board
                .color_on(board.current_player().prev().home())
                .unwrap(),
            Color::Black
        );
        assert_eq!(board.current_player(), Color::Blue);
        let moves = board.moves_for_card(board.current_player(), Card::Tac);
        assert_eq!(moves.len(), 1);
        board.play(moves[0].clone());
        assert_eq!(board.current_player(), Color::Green);
        assert_eq!(
            board.color_on(board.current_player().prev().prev().home()),
            None
        );
        assert_eq!(
            board
                .color_on(board.current_player().prev().home())
                .unwrap(),
            Color::Blue
        );
    }
}
