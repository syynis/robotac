use itertools::Itertools;
use tac_types::{BitBoard, Card, Color, Home, Square, TacAction, TacMove};

use crate::board::Board;

// Balls which can reach home with the given budget
fn balls_reach_home(
    balls: BitBoard,
    budget: u8,
    player: Color,
) -> impl Iterator<Item = (Square, u8)> {
    balls.iter().filter_map(move |b| {
        let dist = b.distance_to_home(player);
        (dist <= budget).then_some((b, dist))
    })
}

fn moves_for_budget(
    balls: BitBoard,
    budget: u8,
    goal: u8,
    player: Color,
) -> impl Iterator<Item = (TacAction, Square, u8)> {
    balls_reach_home(balls, budget - (goal + 1), player).map(move |(b, dist_home)| {
        (
            TacAction::StepInHome { from: b, to: goal },
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
        let mut moves = Vec::new();
        let num_balls = self.balls_with(player).len();
        let home = self.home(player);
        let balls_bb = self.balls_with(player);
        let can_move_home = !(home.is_locked() || home.is_empty());
        let max_home = if can_move_home { 8 } else { 1 };
        let budget_start = if num_balls > 0 { 0 } else { 7 };
        for home_budget in budget_start..max_home {
            // Get all possiblities of moving balls in home with the given budget
            let mut home_moves = if home_budget != 0 {
                get_home_moves_with_budget(home, home_budget)
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

            // If our budget is entirely for home moves don't check for ring moves
            if home_budget == 7 {
                moves.extend(home_moves.iter().map(|steps| {
                    TacMove::new(
                        Card::Seven,
                        TacAction::SevenSteps {
                            steps: steps.clone(),
                        },
                        player,
                    )
                }));
                return moves;
            }

            let board_budget = 7 - home_budget;

            let mut step_in_home_moves = Vec::new();
            if home_budget % 2 == 0 {
                home_moves.push(Vec::new());
            }
            for home_mvs in &home_moves {
                step_in_home_moves.push((home_mvs.clone(), board_budget, balls_bb));
            }

            // For each possible move combination we can do in our home
            for home_mvs in &home_moves {
                let mut new_home = home;
                // Apply changes
                for home_mv in home_mvs {
                    if let TacAction::StepHome { from, to } = home_mv {
                        new_home.unset(*from);
                        new_home.set(*to);
                    }
                }

                // Can enter home
                if new_home.free() > 0 {
                    // Match on the number of free home squares for entry
                    let new_home_free = new_home.free();
                    match new_home_free {
                        // Easy case. Budget is distance to home + 1
                        1 => {
                            for (action, b, budget) in
                                moves_for_budget(balls_bb, board_budget, 0, player)
                            {
                                step_in_home_moves.push((
                                    [home_mvs.clone(), vec![action]].concat(),
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
                        // TODO this could probably be handled in `balls_reach_home` by sorting with distance
                        // and then carrying the remaining budget
                        // TODO are there edge cases with self capturing???
                        2..=4 => {
                            for goal in 0..new_home_free {
                                let budget = board_budget;
                                if goal + 1 > budget {
                                    continue;
                                }
                                for (action1, b1, budget1) in
                                    moves_for_budget(balls_bb, budget, goal, player)
                                {
                                    // If we can move in home, we are wasting with home moves already
                                    let to_waste = if can_move_home { 0 } else { budget1 };
                                    for wasted in (0..=to_waste).step_by(2) {
                                        debug_assert!(wasted % 2 == 0);
                                        let remaining_budget = budget1 - wasted;
                                        step_in_home_moves.push((
                                            [home_mvs.clone(), vec![action1.clone()]].concat(),
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
                                                player,
                                            ) {
                                                step_in_home_moves.push((
                                                    [
                                                        home_mvs.clone(),
                                                        vec![action1.clone(), action2],
                                                    ]
                                                    .concat(),
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
            let mut combinations = Vec::new();
            for (actions, remaining_budget, balls) in step_in_home_moves {
                let balls = balls.iter().collect_vec();
                match balls.len() {
                    0 => {
                        if remaining_budget == 0 {
                            combinations.push(actions.clone());
                        }
                    }
                    1 => {
                        let mut res = actions.clone();
                        if remaining_budget != 0 {
                            res.push(TacAction::Step {
                                from: balls[0],
                                to: balls[0].add(remaining_budget),
                            });
                        }
                        combinations.push(res);
                    }
                    2 => {
                        for i in 0..=remaining_budget {
                            let j = remaining_budget - i;

                            let mut res = actions.clone();
                            if i != 0 {
                                res.push(TacAction::Step {
                                    from: balls[0],
                                    to: balls[0].add(i),
                                });
                            }
                            if j != 0 {
                                res.push(TacAction::Step {
                                    from: balls[1],
                                    to: balls[1].add(j),
                                });
                            }
                            combinations.push(res);
                        }
                    }
                    3 => {
                        for i in 0..=remaining_budget {
                            for j in 0..=remaining_budget - i {
                                let k = remaining_budget - i - j;
                                let mut res = actions.clone();
                                if i != 0 {
                                    res.push(TacAction::Step {
                                        from: balls[0],
                                        to: balls[0].add(i),
                                    });
                                }
                                if j != 0 {
                                    res.push(TacAction::Step {
                                        from: balls[1],
                                        to: balls[1].add(j),
                                    });
                                }
                                if k != 0 {
                                    res.push(TacAction::Step {
                                        from: balls[2],
                                        to: balls[2].add(k),
                                    });
                                }
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
                                    if i != 0 {
                                        res.push(TacAction::Step {
                                            from: balls[0],
                                            to: balls[0].add(i),
                                        });
                                    }
                                    if j != 0 {
                                        res.push(TacAction::Step {
                                            from: balls[1],
                                            to: balls[1].add(j),
                                        });
                                    }
                                    if k != 0 {
                                        res.push(TacAction::Step {
                                            from: balls[2],
                                            to: balls[2].add(k),
                                        });
                                    }
                                    if l != 0 {
                                        res.push(TacAction::Step {
                                            from: balls[3],
                                            to: balls[3].add(l),
                                        });
                                    }
                                    combinations.push(res);
                                }
                            }
                        }
                    }
                    _ => unreachable!(),
                }
            }

            moves.extend(combinations.iter().map(|steps| {
                TacMove::new(
                    Card::Seven,
                    TacAction::SevenSteps {
                        steps: steps.clone(),
                    },
                    player,
                )
            }));
        }
        moves
    }
}

#[allow(clippy::too_many_lines)]
fn get_home_moves_with_budget(home: Home, budget: u8) -> Vec<Vec<(u8, u8)>> {
    let mut moves = Vec::new();
    let unlocked = home.get_all_unlocked();
    if budget == 0 || unlocked.is_empty() {
        return moves;
    }
    let num_unlocked = unlocked.len();
    let even_budget = budget % 2 == 0;
    // Try to spend budget
    match num_unlocked {
        1 => match home.0 {
            0b0001 => {
                if even_budget {
                    moves.push(vec![(0, 2)]);
                } else {
                    moves.push(vec![(0, 1)]);
                    moves.push(vec![(0, 3)]);
                }
            }
            0b0010 => {
                if even_budget {
                    moves.push(vec![(1, 3)]);
                } else {
                    moves.push(vec![(1, 0)]);
                    moves.push(vec![(1, 2)]);
                }
            }
            0b0100 => {
                if even_budget {
                    moves.push(vec![(2, 0)]);
                } else {
                    moves.push(vec![(2, 1)]);
                    moves.push(vec![(2, 3)]);
                }
            }
            0b1001 => {
                if even_budget {
                    moves.push(vec![(0, 2)]);
                } else {
                    moves.push(vec![(0, 1)]);
                }
            }
            0b1010 => {
                if !even_budget {
                    moves.push(vec![(1, 0)]);
                    moves.push(vec![(1, 2)]);
                }
            }
            0b1101 => {
                if !even_budget {
                    moves.push(vec![(0, 1)]);
                }
            }
            _ => unreachable!(),
        },
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
                    moves.push(vec![(2, 3), (0, 1)]);
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
            _ => unreachable!(),
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
