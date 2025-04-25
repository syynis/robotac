use tac_types::{BitBoard, Color, Square};

use crate::board::Board;

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_lossless)]
impl Board {
    #[must_use]
    pub fn eval(&self) -> i64 {
        let mut eval = 0;
        let p = self.current_player();
        let e = self.current_player().next();
        let p_p = self.current_player().partner();
        let e_p = self.current_player().next().partner();
        if self.won(p) {
            return 10000;
        } else if self.won(e) {
            return -10000;
        }

        // How many more balls do we have in goal
        let goal_cnt = self.balls_in_home(p) as i64 - self.balls_in_home(e) as i64;
        eval += goal_cnt * 100;

        // Is our goal free to enter
        let free = self.home_free(p) as u8;
        let p_free = self.home_free(p_p) as u8;
        let e_free = self.home_free(e) as u8;
        let ep_free = self.home_free(e_p) as u8;

        eval += ((free + p_free) as i64 - (e_free + ep_free) as i64) * 10;

        // Is our goal clean
        let clean = self.home_clean(p) as u8;
        let p_clean = self.home_clean(p_p) as u8;
        let e_clean = self.home_clean(e) as u8;
        let ep_clean = self.home_clean(e_p) as u8;

        eval += ((clean + p_clean) as i64 - (e_clean + ep_clean) as i64) * 5;

        // How many balls do we have that are near the goal
        let (fwd, four) = self.near_goal(p);
        let (p_fwd, p_four) = self.near_goal(p_p);
        let (e_fwd, e_four) = self.near_goal(e);
        let (ep_fwd, ep_four) = self.near_goal(e_p);
        let goal_proximity = |f, b, fr| (f * (fr + 1) * 30 + b * (fr + 1) * 15) as i64;
        let our = goal_proximity(fwd, four, free) + goal_proximity(p_fwd, p_four, p_free);
        let theirs =
            goal_proximity(e_fwd, e_four, e_free) + goal_proximity(ep_fwd, ep_four, ep_free);
        eval += our - theirs;

        // Do we have balls in play
        eval += ((self.ball_in_play(p) as i64 + self.ball_in_play(p_p) as i64)
            - (self.ball_in_play(e) as i64 + self.ball_in_play(e_p) as i64))
            * 50;
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

    fn near_goal(&self, player: Color) -> (u8, u8) {
        let mine = self.balls_with(player);

        let in_fwd_proximity =
            |start: Square, player: Color| -> bool { start.distance_to_home(player) < 13 };

        let in_four_proximity =
            |start: Square, player: Color| -> bool { start.distance_to_home(player) > 60 };

        let count = |bb: BitBoard, color: Color| -> (u8, u8) {
            // Cast is valid in all cases because iterating bitboard
            // can return square with value at most 64
            (
                bb.iter()
                    .filter(|ball| in_fwd_proximity(*ball, color))
                    .count() as u8,
                bb.iter()
                    .filter(|ball| in_four_proximity(*ball, color))
                    .count() as u8,
            )
        };
        count(mine, player)
    }
}
