use crate::constants;
use macroquad::prelude::*;

pub struct Player {
    pos: Vec2,
    vel_y: f32,
    pub on_ground: bool,
}

impl Player {
    pub fn new() -> Self {
        Self {
            pos: Vec2::new(
                screen_width() * 0.10,
                constants::ground_y() - constants::PLAYER_SIZE.1,
            ),
            vel_y: 0.,
            on_ground: true,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.update_with_jump(dt, is_key_pressed(KeyCode::Space));
    }

    pub fn update_with_jump(&mut self, dt: f32, jump: bool) {
        self.vel_y += constants::GRAVITY * dt;
        self.pos.y += self.vel_y * dt;

        let ground = constants::ground_y() - constants::PLAYER_SIZE.1;

        if self.pos.y >= ground {
            self.pos.y = ground;
            self.vel_y = 0.;
            self.on_ground = true;
        }

        if jump && self.on_ground {
            self.vel_y = -constants::JUMP_VEL;
            self.pos.y += self.vel_y * dt;
            self.on_ground = false;
        }
    }

    pub fn draw(&self, texture: &Texture2D, anim_frame: bool) {
        let src_x = if !self.on_ground {
            constants::PLAYER_JUMP_X
        } else if anim_frame {
            constants::PLAYER_FRAME1_X
        } else {
            constants::PLAYER_FRAME2_X
        };

        let params = DrawTextureParams {
            source: Some(Rect::new(
                src_x,
                0.,
                constants::PLAYER_SRC_W,
                constants::PLAYER_SRC_H,
            )),
            dest_size: Some(Vec2::new(
                constants::PLAYER_SIZE.0,
                constants::PLAYER_SIZE.1,
            )),
            ..Default::default()
        };
        draw_texture_ex(texture, self.pos.x, self.pos.y, WHITE, params);
    }

    pub fn rect(&self) -> Rect {
        let (w, h) = constants::PLAYER_SIZE;
        Rect::new(self.pos.x, self.pos.y, w, h)
    }

    pub fn pos_y(&self) -> f32 {
        self.pos.y
    }
    pub fn vel_y(&self) -> f32 {
        self.vel_y
    }
}
