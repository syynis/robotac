use tac_types::{BitBoard, Color, Square};

use crate::board::Board;

const WIN: i64 = 10000;
const IN_HOME: i64 = 500;
const HOME_FREE: i64 = 13;
const HOME_CLEAN: i64 = 4;
const IN_PLAY: i64 = 28;
const FWD_DIST_MAX: i64 = 17;
const FWD_IN_HOME: i64 = 21;
const MOBILITY: i64 = 2;
const CAPTURABILITY: i64 = 12;
const FOUR_PROXIMITY: i64 = 23;

impl Board {
    #[must_use]
    pub fn eval(&self) -> i64 {
        let mut eval = 0;
        let p = self.current_player();
        let e = self.current_player().next();
        let p_p = self.current_player().partner();
        let e_p = self.current_player().next().partner();
        if self.won(p) {
            return WIN;
        } else if self.won(e) {
            return -WIN;
        }

        // How many more balls do we have in goal
        let goal_cnt = self.balls_in_home(p) as i64 - self.balls_in_home(e) as i64;
        eval += goal_cnt * IN_HOME;

        // Is our goal free to enter
        let free = self.home_free(p) as u8;
        let p_free = self.home_free(p_p) as u8;
        let e_free = self.home_free(e) as u8;
        let ep_free = self.home_free(e_p) as u8;

        let free = ((free + p_free) as i64 - (e_free + ep_free) as i64) * HOME_FREE;
        eval += free;

        // Is our goal clean
        let clean = self.home_clean(p) as u8;
        let p_clean = self.home_clean(p_p) as u8;
        let e_clean = self.home_clean(e) as u8;
        let ep_clean = self.home_clean(e_p) as u8;

        let clean = ((clean + p_clean) as i64 - (e_clean + ep_clean) as i64) * HOME_CLEAN;
        eval += clean;

        // How many balls do we have that are near the goal
        let fwd = self.near_goal(p);
        let p_fwd = self.near_goal(p_p);
        let e_fwd = self.near_goal(e);
        let ep_fwd = self.near_goal(e_p);
        let our = fwd + p_fwd;
        let theirs = e_fwd + ep_fwd;
        eval += our - theirs;

        // Do we have balls in play
        let in_play = ((self.ball_in_play(p) as i64 + self.ball_in_play(p_p) as i64)
            - (self.ball_in_play(e) as i64 + self.ball_in_play(e_p) as i64))
            * IN_PLAY;
        eval += in_play;

        let capturability = (self.capturability(e) + self.capturability(e_p))
            - (self.capturability(p) + self.capturability(p_p));
        eval += capturability;

        let mobility =
            (self.mobility(p) + self.mobility(p_p)) - (self.mobility(e) + self.mobility(e_p));
        eval += mobility;

        let backup = (self.balls_with(p).len() + self.balls_with(p_p).len()) as i64
            - (self.balls_with(e).len() + self.balls_with(e_p).len()) as i64;
        let backup = backup * 12;
        eval += backup;
        // println!("free {free}");
        // println!("clean {clean}");
        // println!("near goal {}", our - theirs);
        // println!("play {in_play}");
        // println!("cap: {capturability}");
        // println!("mob: {mobility}");
        // println!("back: {backup}");
        eval
    }

    fn ball_in_play(&self, player: Color) -> bool {
        !self.balls_with(player).is_empty()
    }

    fn home_free(&self, player: Color) -> bool {
        self.home(player).amount() > 0 && self.home(player).free() > 0
    }

    fn home_clean(&self, player: Color) -> bool {
        // If home locked we are fine
        if self.home(player).is_locked() {
            return true;
        }
        // Else only one unlocked
        self.home(player).amount() - self.home(player).get_all_unlocked().len() as u8 == 1
    }

    fn balls_in_home(&self, player: Color) -> u8 {
        self.home(player).amount() + self.home(player.partner()).amount()
    }

    fn count(bb: BitBoard, color: Color, f: impl Fn(Square, Color) -> i64) -> i64 {
        bb.iter().map(|sq| f(sq, color)).sum::<i64>()
    }

    fn near_goal(&self, player: Color) -> i64 {
        let mine = self.balls_with(player);
        let in_four_proximity = self
            .moves_for_card_squares(mine, player, player, tac_types::Card::Four)
            .iter()
            .any(|mv| matches!(mv.action, tac_types::TacAction::StepInHome { .. }));
        let in_four_proximity = if in_four_proximity { FOUR_PROXIMITY } else { 0 };

        let fwd = |start: Square, player: Color| -> i64 {
            let dist = start.distance_to_home(player);
            let dist = if dist == 0 && self.fresh(player) {
                64
            } else {
                dist
            };
            let dist_factor = (1.0 - ((dist as f32) / 64.0)).powi(2);
            (FWD_DIST_MAX as f32 * dist_factor) as i64 + if dist < 13 { FWD_IN_HOME } else { 0 }
        };

        Self::count(mine, player, fwd) + in_four_proximity
    }

    fn capturability(&self, player: Color) -> i64 {
        // TODO
        // Should also take into account how valueable the balls are
        let enemies = self.balls_with(player.prev()) | self.balls_with(player.next());

        self.balls_with(player)
            .into_iter()
            .map(|m| {
                enemies
                    .iter()
                    .filter(|enemy| {
                        let enemy_me = enemy.distance_to(m);
                        let me_enemy = m.distance_to(*enemy);
                        // Enemy can is in distance to potentially capture us
                        let can_reach = enemy_me < 14 && self.can_move(*enemy, m);
                        // Enemy can play for to capture us
                        let can_reach_four = me_enemy == 4 && self.can_move(m, *enemy);
                        // Enemy can play seven to capture us. Seperately because we don't need `can_move` check
                        let can_reach_seven = enemy_me < 8;
                        // Eleven is always save
                        let eleven = enemy_me == 11;
                        can_reach || can_reach_four || can_reach_seven && !eleven
                    })
                    .count() as i64
            })
            .sum::<i64>()
            * CAPTURABILITY
    }

    /// A measure of the amount of cards we can play
    /// Returns the sum of distances to next ball for each ball belonging to `player`
    fn mobility(&self, player: Color) -> i64 {
        self.balls_with(player)
            .into_iter()
            .map(|m| {
                let next = self.distance_to_next(m);
                let home = m.distance_to_home(player);
                // Don't want to overstep home so if we are closer to home than next ball take that distance as our mobility instead
                // NOTE the tradeoff is that we are slightly discouraging going towards goal but hopefully this is handled by exploring deeper
                let dist = if next > home {
                    home + self.home(player).free()
                } else {
                    next
                };
                dist.clamp(0, 13) as i64
            })
            .sum::<i64>()
            * MOBILITY
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use tac_types::*;

    #[test]
    fn eval() {
        let mut rand_board = Board::new_random_state(0);
        println!("{:?}", rand_board);
        for color in ALL_COLORS.iter().step_by(3) {
            rand_board.set_player(*color);
            println!("{:?} {}", color, rand_board.eval());
        }
    }
}
