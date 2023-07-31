use ldtk_easy::entity::Field;
use ldtk_easy::layer::Autotile;
use raylib::misc::get_random_value;

use crate::assets::*;
use crate::player::Player;

pub struct Level {
    index: usize,
    size: Vector2,
    grid: Vec<Tile>,
    background: Vec<Autotile>,
    pub numbers: Vec<Number>,
    web: Vec<Vector2>,
    pub current_number: u8,
    door: Rectangle,
    target_door_y: f32,

    pub particles: Vec<Particle>,
    pub overlays: Vec<Overlay>,
}

impl Level {
    pub fn load(assets: &Assets, index: usize) -> Result<Option<(Self, Player)>> {
        if let Some(level) = assets.world.levels().get(index).cloned() {
            let grid = level
                .get_layer("Level")
                .context("No level map found in level!")?
                .int_grid()
                .iter()
                .map(|tile| match tile {
                    0 => Tile::Empty,
                    1 => Tile::Ground,
                    2 => Tile::Spike,
                    3 => Tile::Spike,
                    _ => panic!("Undefined tile '{}'!", tile),
                })
                .collect();

            let background = level
                .get_layer("Level")
                .context("No autotile level map found in level!")?
                .autotiles();

            let mut numbers = Vec::new();
            let mut web = Vec::new();
            let mut player = Player::new(Vector2::default(), Vector2::default());
            let mut door = Rectangle::default();

            for entity in &level
                .get_layer("Entities")
                .context("No entities found in level!")?
                .entities()
            {
                if entity.identifier() == "Player" {
                    player = Player::new(
                        tuple2(entity.pixel_coordinates()),
                        rvec2(entity.width(), entity.height()),
                    );
                } else if entity.identifier() == "Number" {
                    let value = match entity
                        .field("Number")
                        .context("Number entity has no number field!")?
                    {
                        Field::String { value } => value,
                        _ => bail!("Entity number field is of unexpected type!"),
                    };
                    numbers.push(Number::new(
                        tuple2(entity.pixel_coordinates()),
                        value[6..]
                            .parse()
                            .context(format!("Failed to parse number type '{}'!", value))?,
                        None,
                    ));
                } else if entity.identifier() == "Spider" {
                    let value = match entity
                        .field("Number")
                        .context("Spider entity has no number field!")?
                    {
                        Field::String { value } => value,
                        _ => bail!("Entity number field is of unexpected type!"),
                    };
                    let target = match entity
                        .field("Target")
                        .context("Spider entity has no target field!")?
                    {
                        Field::Map { value } => match (&value["cx"], &value["cy"]) {
                            (Field::Int { value: x }, Field::Int { value: y }) => {
                                rvec2(*x as f32, *y as f32)
                            }
                            _ => bail!("Entity target field is of unexpected type!"),
                        },
                        _ => bail!("Entity target field is of unexpected type!"),
                    };
                    numbers.push(Number::new(
                        tuple2(entity.pixel_coordinates()),
                        value[6..]
                            .parse()
                            .context(format!("Failed to parse number type '{}'!", value))?,
                        Some(target),
                    ));
                } else if entity.identifier() == "Door" {
                    door = rrect(
                        entity.pixel_coordinates().0,
                        entity.pixel_coordinates().1,
                        entity.width(),
                        entity.height(),
                    );
                } else if entity.identifier() == "Web" {
                    web.push(tuple2(entity.pixel_coordinates()));
                }
            }

            Ok(Some((
                Self {
                    index,
                    size: rvec2(level.pixel_size().0 as f32, level.pixel_size().1 as f32),
                    grid,
                    background,
                    numbers,
                    web,
                    current_number: match level
                        .field("TargetNumber")
                        .context("Level has no target number!")?
                    {
                        Field::Int { value } => value as _,
                        _ => bail!("Target number field is of unexpected type!"),
                    },
                    door,
                    target_door_y: door.y - 32.0,

                    particles: Vec::new(),
                    overlays: Vec::new(),
                },
                player,
            )))
        } else {
            Ok(None)
        }
    }

    pub fn update(&mut self, rl: &mut RaylibHandle) {
        if self.current_number == 0 {
            self.door.y = (self.door.y - rl.get_frame_time() * 16.0).max(self.target_door_y);
        }

        for number in &mut self.numbers {
            number.update(rl);
        }

        for particle in &mut self.particles {
            particle.update(rl);
        }
        self.particles.retain(Particle::alive);

        for overlay in &mut self.overlays {
            overlay.time -= rl.get_frame_time();
        }
        self.overlays.retain(|overlay| overlay.time >= 0.0);
    }

    pub fn draw<D: RaylibDraw>(&self, assets: &Assets, d: &mut D) {
        for tile in &self.background {
            d.draw_texture_rec(
                &assets.tileset,
                rrect(tile.source.0, tile.source.1, 16, 16),
                tuple2(tile.pixel_position),
                Color::WHITE,
            );
        }
        for web in &self.web {
            d.draw_texture_v(&assets.web, web, Color::WHITE)
        }
        d.draw_texture(
            &assets.door,
            self.door.x as _,
            self.door.y as _,
            Color::WHITE,
        );
        for number in &self.numbers {
            number.draw(assets, d);
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn size(&self) -> Vector2 {
        self.size
    }

    pub fn tile(&self, position: Vector2) -> Tile {
        let size = self.size / 16.0;
        if position.x < 0.0 || position.y < 0.0 || position.x >= size.x || position.y >= size.y {
            return Tile::Empty;
        }
        self.grid[position.x as usize + position.y as usize * size.x as usize]
    }

    pub fn door(&self) -> &Rectangle {
        &self.door
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tile {
    Empty,
    Ground,
    Spike,
}

pub struct Number {
    position: Vector2,
    number: u8,
    timer: f32,
    spider: Option<Spider>,
}

impl Number {
    fn new(position: Vector2, number: u8, spider: Option<Vector2>) -> Self {
        Self {
            position,
            number,
            timer: get_random_value::<i32>(0, 120) as f32 / 180.0 * std::f32::consts::PI,
            spider: spider.map(|spider| Spider {
                origin: position,
                target: spider,
                timer: 0.0,
            }),
        }
    }

    fn update(&mut self, rl: &mut RaylibHandle) {
        if let Some(spider) = &mut self.spider {
            spider.timer = (spider.timer + rl.get_frame_time()) % 18.0;
            match (spider.timer / 3.0) as i32 {
                0 => self.position = spider.origin.lerp(spider.target, spider.timer / 3.0),
                2 => {
                    self.position = spider
                        .target
                        .lerp(spider.origin, (spider.timer - 6.0) / 3.0)
                }
                _ => (),
            }
        } else {
            self.timer += rl.get_frame_time();
        }
    }

    fn draw<D: RaylibDraw>(&self, assets: &Assets, d: &mut D) {
        let position = if let Some(spider) = &self.spider {
            d.draw_line_ex(
                spider.origin + 12.0,
                self.position + 12.0,
                2.0,
                Color::WHITE,
            );
            d.draw_texture_v(&assets.spider, self.position, Color::WHITE);
            self.position + 4.0
        } else {
            self.position + rvec2(0, (self.timer * 3.0).sin() * 8.0)
        };
        d.draw_texture_rec(
            &assets.numbers,
            rrect((self.number - 1) * 16, 0, 16, 16),
            position,
            Color::WHITE,
        );
    }

    pub fn rect(&self) -> Rectangle {
        rrect(self.position.x, self.position.y, 16, 16)
    }

    pub fn center(&self) -> Vector2 {
        self.position + 8.0
    }

    pub fn number(&self) -> u8 {
        self.number
    }
}

pub struct Spider {
    origin: Vector2,
    target: Vector2,
    timer: f32,
}

pub struct Particle {
    position: Vector2,
    velocity: Vector2,
    life_time: f32,
    color: Color,
}

impl Particle {
    pub fn new(position: Vector2, velocity: Vector2, life_time: f32, color: Color) -> Self {
        Self {
            position,
            velocity,
            life_time,
            color,
        }
    }

    pub fn update(&mut self, rl: &mut RaylibHandle) {
        self.life_time -= rl.get_frame_time();
        self.position += self.velocity * rl.get_frame_time();
        self.velocity.y += 1000.0 * rl.get_frame_time();
    }

    pub fn draw<D: RaylibDraw>(&self, d: &mut D) {
        d.draw_pixel_v(self.position, self.color);
    }

    pub fn alive(&self) -> bool {
        self.life_time > 0.0
    }
}

pub struct Overlay {
    text: String,
    pub time: f32,
}

impl Overlay {
    pub fn new(text: String) -> Self {
        Self { text, time: 0.5 }
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}
