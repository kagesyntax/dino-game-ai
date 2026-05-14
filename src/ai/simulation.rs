use crate::ai::network::INPUT;
use crate::constants;
use ::rand::{Rng, thread_rng};
use macroquad::prelude::*;

pub struct Simulation {
    pub player_y: f32,
    pub player_vy: f32,
    pub on_ground: bool,
    pub ground_offset: f32,
    pub small_scroll: f32,
    pub big_scroll: f32,
    pub small_multi: i32,
    pub big_multi: i32,
    pub small_pic: f32,
    pub big_pic: f32,
    pub active_small: bool,
    score: u32,
    score_timer: f32,
    game_speed: f32,
    elapsed: f32,
}

impl Simulation {
    pub fn new() -> Self {
        let mut rng = thread_rng();
        Self {
            player_y: constants::ground_y() - constants::PLAYER_SIZE.1,
            player_vy: 0.0,
            on_ground: true,
            ground_offset: 0.0,
            score: 0,
            score_timer: 0.0,
            small_scroll: constants::SMALL_INIT_SCROLL,
            big_scroll: constants::BIG_INIT_SCROLL,
            small_multi: rng.gen_range(1..=3),
            big_multi: rng.gen_range(1..=3),
            small_pic: constants::SMALL_PICS[rng.gen_range(0..2)],
            big_pic: constants::BIG_PICS[rng.gen_range(0..2)],
            active_small: true,
            game_speed: 7.0,
            elapsed: 0.0,
        }
    }

    pub fn features(&self) -> [f32; INPUT] {
        let (dist, obs_w, obs_h) = self.nearest_obstacle();
        let sw = screen_width();
        let sh = screen_height();
        let gs_y = constants::ground_y();

        [
            (dist / sw).clamp(0.0, 1.0),
            (obs_w / sw).clamp(0.0, 1.0),
            (obs_h / sh).clamp(0.0, 1.0),
            ((gs_y - self.player_y) / 200.0).clamp(0.0, 1.0),
            (self.player_vy / 800.0).clamp(-1.0, 1.0),
            if self.on_ground { 1.0 } else { 0.0 },
            ((self.game_speed - 7.0) / 10.0).clamp(0.0, 1.0),
        ]
    }

    fn nearest_obstacle(&self) -> (f32, f32, f32) {
        let player_x = screen_width() * 0.10;
        let mut min_dist = f32::MAX;
        let mut w = 0.0;
        let mut h = 0.0;

        for &(scroll, multi, base_w, obs_h) in &[
            (
                self.small_scroll,
                self.small_multi,
                constants::SMALL_W,
                constants::SMALL_H,
            ),
            (
                self.big_scroll,
                self.big_multi,
                constants::BIG_W,
                constants::BIG_H,
            ),
        ] {
            let ox = screen_width() - scroll;
            let ow = base_w * multi as f32;
            let dist = ox - (player_x + constants::PLAYER_SIZE.0);
            if dist < min_dist && ox + ow > player_x {
                min_dist = dist;
                w = ow;
                h = obs_h;
            }
        }

        (min_dist, w, h)
    }

    pub fn step(&mut self, dt: f32, jump: bool, game_speed: f32) {
        self.game_speed = game_speed;
        self.elapsed += dt;

        self.ground_offset = (self.ground_offset + game_speed * dt) % constants::GROUND_W;

        if self.active_small {
            self.small_scroll += game_speed * dt;
            if self.small_scroll
                > screen_width() + constants::SMALL_W * self.small_multi as f32 * 3.0
            {
                self.active_small = false;
                self.big_scroll = constants::BIG_INIT_SCROLL;
                let mut rng = thread_rng();
                self.big_multi = rng.gen_range(1..=3);
                self.big_pic = constants::BIG_PICS[rng.gen_range(0..2)];
            }
        } else {
            self.big_scroll += game_speed * dt;
            if self.big_scroll > screen_width() + constants::BIG_W * self.big_multi as f32 * 3.0 {
                self.active_small = true;
                self.small_scroll = constants::SMALL_INIT_SCROLL;
                let mut rng = thread_rng();
                self.small_multi = rng.gen_range(1..=3);
                self.small_pic = constants::SMALL_PICS[rng.gen_range(0..2)];
            }
        }

        self.player_vy += constants::GRAVITY * dt;
        self.player_y += self.player_vy * dt;

        let ground = constants::ground_y() - constants::PLAYER_SIZE.1;
        if self.player_y >= ground {
            self.player_y = ground;
            self.player_vy = 0.0;
            self.on_ground = true;
        }

        if jump && self.on_ground {
            self.player_vy = -constants::JUMP_VEL;
            self.player_y += self.player_vy * dt;
            self.on_ground = false;
        }

        self.score_timer += dt;
        if self.score_timer >= 0.1 {
            self.score += 1;
            self.score_timer -= 0.1;
        }
    }

    pub fn check_collision(&self) -> bool {
        let player_x = screen_width() * 0.10;
        let player_rect = Rect::new(
            player_x + 10.0,
            self.player_y + 10.0,
            constants::PLAYER_SIZE.0 - 20.0,
            constants::PLAYER_SIZE.1 - 20.0,
        );

        for &(scroll, multi, base_w, obs_h) in &[
            (
                self.small_scroll,
                self.small_multi,
                constants::SMALL_W,
                constants::SMALL_H,
            ),
            (
                self.big_scroll,
                self.big_multi,
                constants::BIG_W,
                constants::BIG_H,
            ),
        ] {
            let ox = screen_width() - scroll;
            let ow = base_w * multi as f32;
            let r = Rect::new(
                ox + 5.0,
                constants::ground_y() - obs_h + 5.0,
                ow - 10.0,
                obs_h - 10.0,
            );
            if r.overlaps(&player_rect) {
                return true;
            }
        }
        false
    }

    pub fn score(&self) -> u32 {
        self.score
    }
    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }
}
