use smallvec::{smallvec, SmallVec};
use tac_types::{BitBoard, Card, Color, Home, SevenAction, Square, TacAction, TacMove};

use crate::board::Board;

// Balls which can reach home with the given budget
fn balls_reach_home(
    balls: BitBoard,
    budget: u8,
    player: Color,
    fresh: bool,
) -> impl Iterator<Item = (Square, u8)> {
    balls.iter().filter_map(move |b| {
        let dist = b.distance_to_home(player);
        let need_fresh = if b == player.home() { fresh } else { false };
        (dist <= budget && !need_fresh).then_some((b, dist))
    })
}

fn moves_for_budget(
    balls: BitBoard,
    budget: u8,
    goal: u8,
    player: Color,
    fresh: bool,
) -> impl Iterator<Item = (SevenAction, Square, u8)> {
    balls_reach_home(balls, budget - (goal + 1), player, fresh).map(move |(b, dist_home)| {
        (
            SevenAction::StepInHome { from: b, to: goal },
            b,
            budget - (dist_home + goal + 1),
        )
    })
}

impl Board {
    #[allow(clippy::too_many_lines)]
    pub(crate) fn seven_moves(&self, player: Color) -> Vec<TacMove> {
        // TODO Some thoughts about generating seven moves
        // This still needs to take into account moves that go from ring to home
        let play_for = self.play_for(player);
        let mut moves = Vec::new();
        let num_balls = self.balls_with(play_for).len();
        let home = *self.home(play_for);
        let balls_bb = self.balls_with(play_for);
        let can_move_home = !(home.is_locked() || home.is_empty());
        let max_home = if can_move_home { 8 } else { 1 };
        let budget_start = if num_balls > 0 { 0 } else { 7 };
        let fresh = self.fresh(play_for);
        let min_board_budget = (1..8)
            .find(|budget| {
                balls_bb.iter().any(|b| {
                    let dist = b.distance_to_home(player);
                    dist <= (budget - 1) && !self.fresh(play_for)
                })
            })
            .unwrap_or(8);
        for home_budget in budget_start..max_home {
            // Get all possiblities of moving balls in home with the given budget
            let get_home_moves_with_budget = get_home_moves_with_budget(home, home_budget);
            let mut home_moves = get_home_moves_with_budget;

            // If our budget is entirely for home moves don't check for ring moves
            if home_budget == 7 {
                moves.extend(home_moves.into_iter().map(|steps| {
                    TacMove::new(
                        Card::Seven,
                        TacAction::SevenSteps { steps },
                        play_for,
                        player,
                    )
                }));
                return moves;
            }

            let board_budget = 7 - home_budget;

            let mut step_in_home_moves: SmallVec<(SmallVec<SevenAction, 4>, u8, BitBoard), 4> =
                SmallVec::new();
            if home_budget & 1 == 0 {
                home_moves.push(SmallVec::new());
            }
            for home_mvs in &home_moves {
                step_in_home_moves.push((home_mvs.clone(), board_budget, balls_bb));
            }

            if board_budget >= min_board_budget {
                get_step_in_home_moves(
                    play_for,
                    home,
                    balls_bb,
                    can_move_home,
                    &home_moves,
                    board_budget,
                    fresh,
                    &mut step_in_home_moves,
                );
            }

            let push = |res: &mut SmallVec<SevenAction, 4>, from: Square, amount: u8| {
                if amount != 0 {
                    res.push(SevenAction::Step {
                        from,
                        to: from.add(amount),
                    });
                }
            };
            let mut combinations: SmallVec<SmallVec<SevenAction, 4>, 64> = SmallVec::new();
            for (actions, remaining_budget, balls) in step_in_home_moves {
                let balls: SmallVec<Square, 4> = balls.iter().collect();
                match balls.len() {
                    0 => {
                        if remaining_budget == 0 {
                            combinations.push(actions);
                        } else {
                            // If there are no balls in ring or base all must be in home
                            // Then if we are not playing for partner we can use remaining budget
                            // to move their balls
                            if self.num_base(play_for) == 0 && play_for == player {
                                // TODO here we should check if we can move partners balls because
                                // Optimally we would just call `seven_moves` here but that would need a bunch of changes
                            }
                        }
                    }
                    1 => {
                        let mut res = actions.clone();
                        push(&mut res, balls[0], remaining_budget);
                        combinations.push(res);
                    }
                    2 => {
                        for i in 0..=remaining_budget {
                            let j = remaining_budget - i;

                            let mut res = actions.clone();
                            push(&mut res, balls[0], i);
                            push(&mut res, balls[1], j);
                            combinations.push(res);
                        }
                    }
                    3 => {
                        for i in 0..=remaining_budget {
                            for j in 0..=remaining_budget - i {
                                let k = remaining_budget - i - j;
                                let mut res = actions.clone();
                                push(&mut res, balls[0], i);
                                push(&mut res, balls[1], j);
                                push(&mut res, balls[2], k);
                                combinations.push(res);
                            }
                        }
                    }
                    4 => {
                        for i in 0..=remaining_budget {
                            for j in 0..=remaining_budget - i {
                                for k in 0..=remaining_budget - i - j {
                                    let l = remaining_budget - i - j - k;
                                    let mut res = actions.clone();
                                    push(&mut res, balls[0], i);
                                    push(&mut res, balls[1], j);
                                    push(&mut res, balls[2], k);
                                    push(&mut res, balls[3], l);
                                    combinations.push(res);
                                }
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            }

            moves.extend(combinations.into_iter().map(|steps| {
                TacMove::new(
                    Card::Seven,
                    TacAction::SevenSteps { steps },
                    play_for,
                    player,
                )
            }));
        }
        moves
    }
}

fn get_step_in_home_moves(
    play_for: Color,
    home: Home,
    balls_bb: BitBoard,
    can_move_home: bool,
    home_moves: &SmallVec<SmallVec<SevenAction, 4>, 4>,
    board_budget: u8,
    fresh: bool,
    step_in_home_moves: &mut SmallVec<(SmallVec<SevenAction, 4>, u8, BitBoard), 4>,
) {
    // For each possible move combination we can do in our home
    for home_mvs in home_moves {
        let mut new_home = home;
        // Apply changes
        for home_mv in home_mvs.clone() {
            if let SevenAction::StepHome { from, to } = home_mv {
                new_home.unset(from);
                new_home.set(to);
            }
        }

        let new_home_free = new_home.free();
        // Match on the number of free home squares for entry
        match new_home_free {
            // Easy case. Budget is distance to home + 1
            1 => {
                for (action, b, budget) in
                    moves_for_budget(balls_bb, board_budget, 0, play_for, fresh)
                {
                    step_in_home_moves.push((
                        [home_mvs.clone(), smallvec![action]].concat().into(),
                        budget,
                        balls_bb ^ b.bitboard(),
                    ));
                }
            }
            // With two or more free spaces, we first move one ball (probably should be the closest??)
            // Then we can "waste" budget and / or if we wasted an odd amount of budget
            // check if we still have enough to reach the the goal with the second ball
            // We can handle 3 and 4 the same as 2 because there is no way
            // to put 3 or 4 balls in the goal with seven steps
            2..=4 => {
                for goal in 0..new_home_free {
                    let budget = board_budget;
                    if goal + 1 > budget {
                        continue;
                    }
                    for (action1, b1, budget1) in
                        moves_for_budget(balls_bb, budget, goal, play_for, fresh)
                    {
                        // If we can move in home, we are wasting with home moves already
                        let to_waste = if can_move_home { 0 } else { budget1 };
                        for wasted in (0..=to_waste).step_by(2) {
                            debug_assert!(wasted % 2 == 0);
                            let remaining_budget = budget1 - wasted;
                            step_in_home_moves.push((
                                [home_mvs.clone(), smallvec![action1.clone()]]
                                    .concat()
                                    .into(),
                                remaining_budget,
                                balls_bb ^ b1.bitboard(),
                            ));
                            for goal2 in 0..goal {
                                if goal2 + 1 > remaining_budget {
                                    continue;
                                }
                                for (action2, b2, budget2) in moves_for_budget(
                                    balls_bb ^ b1.bitboard(),
                                    remaining_budget,
                                    goal2,
                                    play_for,
                                    fresh,
                                ) {
                                    step_in_home_moves.push((
                                        [home_mvs.clone(), smallvec![action1.clone(), action2]]
                                            .concat()
                                            .into(),
                                        budget2,
                                        balls_bb ^ b1.bitboard() ^ b2.bitboard(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

#[allow(clippy::too_many_lines)]
fn get_home_moves_with_budget(home: Home, budget: u8) -> SmallVec<SmallVec<SevenAction, 4>, 4> {
    let mut moves: SmallVec<SmallVec<(u8, u8), 4>, 4> = SmallVec::new();
    let unlocked = home.get_all_unlocked();
    if budget == 0 || unlocked.is_empty() {
        return SmallVec::new();
    }
    let num_unlocked = unlocked.len();
    let even_budget = budget & 1 == 0;
    // Try to spend budget
    match num_unlocked {
        1 => match home.0 {
            0b0001 => {
                if even_budget {
                    moves.push(smallvec![(0, 2)]);
                } else {
                    moves.push(smallvec![(0, 1)]);
                    moves.push(smallvec![(0, 3)]);
                }
            }
            0b0010 => {
                if even_budget {
                    moves.push(smallvec![(1, 3)]);
                } else {
                    moves.push(smallvec![(1, 0)]);
                    moves.push(smallvec![(1, 2)]);
                }
            }
            0b0100 => {
                if even_budget {
                    moves.push(smallvec![(2, 0)]);
                } else {
                    moves.push(smallvec![(2, 1)]);
                    moves.push(smallvec![(2, 3)]);
                }
            }
            0b1001 => {
                if even_budget {
                    moves.push(smallvec![(0, 2)]);
                } else {
                    moves.push(smallvec![(0, 1)]);
                }
            }
            0b1010 => {
                if !even_budget {
                    moves.push(smallvec![(1, 0)]);
                    moves.push(smallvec![(1, 2)]);
                }
            }
            0b1101 => {
                if !even_budget {
                    moves.push(smallvec![(0, 1)]);
                }
            }
            _ => unreachable!(),
        },
        2 => match home.0 {
            0b0110 => {
                if even_budget {
                    moves.push(smallvec![(2, 3), (1, 2)]);
                    moves.push(smallvec![(2, 3), (1, 0)]);
                    moves.push(smallvec![(1, 0), (2, 1)]);
                } else {
                    moves.push(smallvec![(2, 3)]);
                    moves.push(smallvec![(1, 0)]);
                }
            }
            0b0101 => {
                if even_budget {
                    moves.push(smallvec![(2, 3), (0, 1)]);
                } else {
                    moves.push(smallvec![(0, 1)]);
                    moves.push(smallvec![(2, 3)]);
                    moves.push(smallvec![(2, 1)]);
                    if budget > 1 {
                        moves.push(smallvec![(2, 3), (0, 2)]);
                    }
                }
            }
            0b0011 => {
                if even_budget {
                    moves.push(smallvec![(1, 3)]);
                    moves.push(smallvec![(1, 2), (0, 1)]);
                    if budget > 2 {
                        moves.push(smallvec![(1, 3), (0, 2)]);
                    }
                } else {
                    moves.push(smallvec![(1, 2)]);

                    if budget > 1 {
                        moves.push(smallvec![(1, 3), (0, 1)]);
                    }
                }
            }
            0b1011 => {
                if even_budget {
                    moves.push(smallvec![(1, 2), (0, 1)]);
                } else {
                    moves.push(smallvec![(1, 2)]);
                }
            }
            _ => unreachable!(),
        },
        3 => {
            if even_budget {
                moves.push(smallvec![(2, 3), (1, 2)]);
            } else {
                moves.push(smallvec![(2, 3)]);
                if budget > 2 {
                    moves.push(smallvec![(2, 3), (1, 2), (0, 1)]);
                }
            }
        }
        _ => unreachable!(),
    }
    moves
        .into_iter()
        .map(|hm| {
            hm.into_iter()
                .map(|(from, to)| SevenAction::StepHome { from, to })
                .collect()
        })
        .collect()
}

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
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
}
