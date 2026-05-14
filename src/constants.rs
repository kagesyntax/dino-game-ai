use macroquad::prelude::*;

pub const GROUND_RATIO: f32 = 0.85;
pub const GRAVITY: f32 = 2200.0;
pub const JUMP_VEL: f32 = 900.0;

pub fn ground_y() -> f32 {
    screen_height() * GROUND_RATIO
}

// Player sprite (y=0 in sprite, h=94)
pub const PLAYER_SIZE: (f32, f32) = (89., 94.);
pub const PLAYER_SRC_W: f32 = 88.;
pub const PLAYER_SRC_H: f32 = 94.;
pub const PLAYER_FRAME1_X: f32 = 1514.;
pub const PLAYER_FRAME2_X: f32 = 1602.;
pub const PLAYER_JUMP_X: f32 = 1338.;

// Ground sprite (y=104 in sprite, h=18, w=2404)
pub const GROUND_SRC_Y: f32 = 104.;
pub const GROUND_H: f32 = 18.;
pub const GROUND_W: f32 = 2404.;

// Small obstacle (y=2 in sprite)
pub const SMALL_W: f32 = 34.;
pub const SMALL_H: f32 = 70.;
pub const SMALL_PICS: [f32; 2] = [446., 548.];
pub const SMALL_INIT_SCROLL: f32 = -100.;

// Big obstacle (y=2 in sprite)
pub const BIG_W: f32 = 49.;
pub const BIG_H: f32 = 100.;
pub const BIG_PICS: [f32; 2] = [652., 802.];
pub const BIG_INIT_SCROLL: f32 = -200.;
