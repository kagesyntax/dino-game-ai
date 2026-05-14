use crate::constants;
use ::rand::Rng;
use macroquad::prelude::*;

enum ActiveType {
    Small,
    Big,
}

struct Obstacle {
    base_w: f32,
    h: f32,
    scroll: f32,
    multi: i32,
    pic: f32,
    pics: &'static [f32],
    init_scroll: f32,
}

impl Obstacle {
    fn small() -> Self {
        Self {
            base_w: constants::SMALL_W,
            h: constants::SMALL_H,
            scroll: constants::SMALL_INIT_SCROLL,
            multi: 1,
            pic: constants::SMALL_PICS[0],
            pics: &constants::SMALL_PICS,
            init_scroll: constants::SMALL_INIT_SCROLL,
        }
    }

    fn big() -> Self {
        Self {
            base_w: constants::BIG_W,
            h: constants::BIG_H,
            scroll: constants::BIG_INIT_SCROLL,
            multi: 1,
            pic: constants::BIG_PICS[0],
            pics: &constants::BIG_PICS,
            init_scroll: constants::BIG_INIT_SCROLL,
        }
    }

    fn w(&self) -> f32 {
        self.base_w * self.multi as f32
    }

    fn screen_x(&self) -> f32 {
        screen_width() - self.scroll
    }

    fn finished(&self) -> bool {
        self.scroll > screen_width() + self.w() * 3.
    }

    fn activate(&mut self) {
        let mut rng = ::rand::thread_rng();
        self.multi = rng.gen_range(1..=3);
        self.pic = self.pics[rng.gen_range(0..self.pics.len())];
        self.scroll = self.init_scroll;
    }
}

pub struct Enemies {
    small: Obstacle,
    big: Obstacle,
    active: ActiveType,
}

impl Enemies {
    pub fn new() -> Self {
        let mut small = Obstacle::small();
        small.activate();
        Self {
            small,
            big: Obstacle::big(),
            active: ActiveType::Small,
        }
    }

    pub fn update(&mut self, dt: f32, game_speed: f32) {
        match self.active {
            ActiveType::Small => {
                self.small.scroll += game_speed * dt;
                if self.small.finished() {
                    self.active = ActiveType::Big;
                    self.big.activate();
                }
            }
            ActiveType::Big => {
                self.big.scroll += game_speed * dt;
                if self.big.finished() {
                    self.active = ActiveType::Small;
                    self.small.activate();
                }
            }
        }
    }

    pub fn draw(&self, texture: &Texture2D) {
        for obs in [&self.small, &self.big] {
            let src_y = 2.;
            let params = DrawTextureParams {
                source: Some(Rect::new(obs.pic, src_y, obs.w(), obs.h)),
                dest_size: Some(Vec2::new(obs.w(), obs.h)),
                ..Default::default()
            };
            draw_texture_ex(
                texture,
                obs.screen_x(),
                constants::ground_y() - obs.h,
                WHITE,
                params,
            );
        }
    }

    pub fn hitboxes(&self) -> Vec<Rect> {
        [&self.small, &self.big]
            .iter()
            .map(|obs| {
                Rect::new(
                    obs.screen_x() + 5.0,
                    constants::ground_y() - obs.h + 5.0,
                    obs.w() - 10.0,
                    obs.h - 10.0,
                )
            })
            .collect()
    }

    pub fn check_collision(&self, player_rect: &Rect) -> bool {
        let collision = |obs: &Obstacle| -> bool {
            let r = Rect::new(
                obs.screen_x() + 5.0,
                constants::ground_y() - obs.h + 5.0,
                obs.w() - 10.0,
                obs.h - 10.0,
            );
            r.overlaps(player_rect)
        };
        collision(&self.small) || collision(&self.big)
    }
}
