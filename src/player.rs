use raylib::misc::get_random_value;

use crate::assets::*;
use crate::level::*;

pub struct Player {
    position: Vector2,
    size: Vector2,
    velocity: Vector2,
    jumps: u8,
    max_jumps: u8,
    holding_to_wall: bool,
    frame: i8,
}

impl Player {
    pub fn new(position: Vector2, size: Vector2) -> Self {
        Self {
            position,
            size,
            velocity: Vector2::zero(),
            jumps: 0,
            max_jumps: 2,
            holding_to_wall: false,
            frame: 0,
        }
    }

    pub fn collides(&self, level: &Level) -> bool {
        let player_rect = rrect(
            self.position.x + 0.5,
            self.position.y + 0.5,
            self.size.x - 1.0,
            self.size.y - 1.0,
        );
        if player_rect.check_collision_recs(level.door()) {
            return true;
        }
        let player_rect = rrect(
            player_rect.x / 16.0,
            player_rect.y / 16.0,
            player_rect.width / 16.0,
            player_rect.height / 16.0,
        );
        for y in player_rect.y as usize..=(player_rect.y + player_rect.height) as usize {
            for x in player_rect.x as usize..=(player_rect.x + player_rect.width) as usize {
                if match level.tile(rvec2(x as f32, y as f32)) {
                    Tile::Empty => false,
                    Tile::Ground => true,
                    Tile::Spike => false,
                } {
                    return true;
                }
            }
        }
        player_rect.x < 0.0
            || player_rect.y < 0.0
            || player_rect.y + player_rect.height >= level.size().y / 16.0
    }

    pub fn collidable_move(&mut self, rl: &mut RaylibHandle, level: &Level, direction: Vector2) {
        let motion = self.velocity * direction * rl.get_frame_time();
        self.position += motion;
        if self.collides(level) {
            loop {
                self.position -= rvec2(self.velocity.x.signum(), self.velocity.y.signum())
                    * direction
                    * motion.length();
                if !self.collides(level) {
                    break;
                }
            }

            if direction.x != 0.0 {
                if (rl.is_key_down(KeyboardKey::KEY_D) as i32
                    - rl.is_key_down(KeyboardKey::KEY_A) as i32) as f32
                    == motion.x.signum()
                    && rl.is_key_down(KeyboardKey::KEY_S)
                {
                    self.holding_to_wall = true;
                }
                self.velocity.x = 0.0;
            }
            if direction.y != 0.0 {
                self.velocity.y = 0.0;
                if motion.y > 0.0 {
                    self.jumps = self.max_jumps;
                }
            }
        } else if motion.x != 0.0
            || (rl.is_key_down(KeyboardKey::KEY_D) as i32
                - rl.is_key_down(KeyboardKey::KEY_A) as i32) as f32
                == 0.0
            || rl.is_key_released(KeyboardKey::KEY_S)
        {
            self.holding_to_wall = false;
        }
    }

    pub fn update(
        &mut self,
        assets: &mut Assets,
        rl: &mut RaylibHandle,
        level: &mut Level,
        state: &mut State,
    ) {
        // * Jump
        if rl.is_key_pressed(KeyboardKey::KEY_SPACE) && (self.jumps > 0 || self.holding_to_wall) {
            if self.holding_to_wall {
                self.velocity.x = (rl.is_key_down(KeyboardKey::KEY_D) as i32
                    - rl.is_key_down(KeyboardKey::KEY_A) as i32)
                    as f32
                    * -300.0;
                self.holding_to_wall = false;
            } else if self.jumps > 0 {
                self.jumps -= 1;
            }
            self.velocity.y = -300.0;
            assets.audio.play_sound(&assets.jump_sound);
        }

        if rl.is_key_released(KeyboardKey::KEY_SPACE) && self.velocity.y < 0.0 {
            self.velocity.y *= 0.5;
        }

        // Gravity
        self.velocity.y += 1000.0 * rl.get_frame_time();

        if self.holding_to_wall {
            self.velocity.y *= 0.0;
        }

        // * Integration
        let target_velocity = (rl.is_key_down(KeyboardKey::KEY_D) as i32
            - rl.is_key_down(KeyboardKey::KEY_A) as i32) as f32
            * self.size.x
            * 10.0;

        self.velocity.x +=
            (target_velocity - self.velocity.x) * (1.0 - 0.5_f32.powf(rl.get_frame_time() / 0.1));

        while self.collides(level) {
            self.position.y -= 0.5;
        }

        // self.holding_to_wall = false;

        self.collidable_move(rl, level, rvec2(1, 0));
        self.collidable_move(rl, level, rvec2(0, 1));
        self.check_interactibles(assets, level, state);

        if self.velocity.x.abs() > 10.0 {
            self.frame = self.velocity.x.signum() as i8 * (rl.get_time() * 20.0 % 2.0 + 1.0) as i8;
        } else if self.holding_to_wall {
            self.frame = (rl.is_key_down(KeyboardKey::KEY_D) as i8
                - rl.is_key_down(KeyboardKey::KEY_A) as i8)
                * 3;
        } else {
            self.frame = 0;
        }
    }

    fn check_interactibles(&self, assets: &mut Assets, level: &mut Level, state: &mut State) {
        let player_rect = rrect(self.position.x, self.position.y, self.size.x, self.size.y);
        fn explode(level: &mut Level, center: Vector2, count: usize, power: i32, color: Color) {
            for _ in 0..count {
                let velocity = tuple2(
                    (get_random_value::<i32>(-60, 60) as f32 * std::f32::consts::PI / 180.0)
                        .sin_cos(),
                ) * rvec2(1, -1)
                    * get_random_value::<i32>(0, power) as f32;
                level
                    .particles
                    .push(Particle::new(center, velocity, 1.0, color));
            }
        }

        fn game_over(assets: &mut Assets, level: &mut Level, state: &mut State, center: Vector2) {
            *state = State::transition(level.index());
            explode(level, center, 200, 200, Color::RED);
            assets.audio.play_sound(&assets.game_over_sound);
        }

        for i in 0..level.numbers.len() {
            let number = &level.numbers[i];
            if number.rect().check_collision_recs(&player_rect) {
                if level.current_number != number.number() {
                    game_over(assets, level, state, self.center());
                } else {
                    level
                        .overlays
                        .push(Overlay::new(number.number().to_string()));
                    level.current_number -= 1;
                    explode(level, number.center(), 20, 140, Color::WHITE);
                    level.numbers.remove(i);
                    assets.audio.play_sound(&assets.number_sound);
                }
                break;
            }
        }

        let player_rect = rrect(
            player_rect.x / 16.0 + 0.25,
            player_rect.y / 16.0 + 0.25,
            player_rect.width / 16.0 - 0.5,
            player_rect.height / 16.0 - 0.5,
        );

        for y in player_rect.y as usize..=(player_rect.y + player_rect.height) as usize {
            for x in player_rect.x as usize..=(player_rect.x + player_rect.width) as usize {
                if level.tile(rvec2(x as f32, y as f32)) == Tile::Spike {
                    game_over(assets, level, state, self.center());
                }
            }
        }
    }

    pub fn draw<D: RaylibDraw>(&self, assets: &Assets, d: &mut D) {
        d.draw_texture_rec(
            &assets.player,
            rrect(
                (self.frame + 3) * self.size.x as i8,
                0,
                self.size.x,
                self.size.y,
            ),
            self.position,
            Color::WHITE,
        );
    }

    pub fn camera(&self, level: &Level) -> Camera2D {
        let position = self.center() / 256.0;
        Camera2D {
            offset: Vector2::zero(),
            target: rvec2(
                position.x.floor().min(level.size().x / 256.0 - 1.0),
                position.y.floor(),
            ) * 256.0,
            rotation: 0.0,
            zoom: 3.0,
        }
    }

    pub fn position(&self) -> Vector2 {
        self.position
    }

    fn center(&self) -> Vector2 {
        self.position + self.size / 2.0
    }
}
