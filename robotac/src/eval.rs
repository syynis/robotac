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
        eval += goal_cnt * 250;

        // Is our goal free to enter
        let free = self.home_free(p) as u8;
        let p_free = self.home_free(p_p) as u8;
        let e_free = self.home_free(e) as u8;
        let ep_free = self.home_free(e_p) as u8;

        eval += ((free + p_free) as i64 - (e_free + ep_free) as i64) * 5;

        // Is our goal clean
        let clean = self.home_clean(p) as u8;
        let p_clean = self.home_clean(p_p) as u8;
        let e_clean = self.home_clean(e) as u8;
        let ep_clean = self.home_clean(e_p) as u8;

        eval += ((clean + p_clean) as i64 - (e_clean + ep_clean) as i64) * 2;

        // How many balls do we have that are near the goal
        let fwd = self.near_goal(p);
        let p_fwd = self.near_goal(p_p);
        let e_fwd = self.near_goal(e);
        let ep_fwd = self.near_goal(e_p);
        let our = fwd + p_fwd;
        let theirs = e_fwd + ep_fwd;
        eval += our - theirs;

        // Do we have balls in play
        eval += ((self.ball_in_play(p) as i64 + self.ball_in_play(p_p) as i64)
            - (self.ball_in_play(e) as i64 + self.ball_in_play(e_p) as i64))
            * 50;
        println!("{eval}");
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

    fn near_goal(&self, player: Color) -> i64 {
        let mine = self.balls_with(player);
        let in_four_proximity = self
            .moves_for_card_squares(mine, player, player, tac_types::Card::Four)
            .iter()
            .any(|mv| matches!(mv.action, tac_types::TacAction::StepInHome { .. }));

        let fwd = |start: Square, player: Color| -> i64 {
            let dist = start.distance_to_home(player);
            let dist = if dist == 0 && self.fresh(player) {
                64
            } else {
                dist
            };
            let dist_factor = (1.0 - ((dist as f32) / 64.0)).powi(2);
            (20.0 * dist_factor) as i64 + if dist < 13 { 10 } else { 0 }
        };

        let count = |bb: BitBoard, color: Color| -> i64 {
            // Cast is valid in all cases because iterating bitboard
            // can return square with value at most 64
            bb.iter().map(|sq| fwd(sq, color)).sum::<i64>()
        };
        count(mine, player)
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
    }

    // Amount of cards we can play
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
                (dist * 3) as i64
            })
            .sum::<i64>()
    }
}
