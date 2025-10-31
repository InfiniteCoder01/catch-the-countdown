#![windows_subsystem = "windows"]

pub mod assets;
pub mod level;
pub mod player;
use assets::*;

fn format_time(time: f32) -> String {
    format!(
        "{:0>2}:{:0>2}:{:0>5.2}",
        time as u32 / 3600,
        (time as u32 / 60) % 60,
        time % 60.0
    )
}

struct Button {
    scale: f32,
    last_hovered: bool,
    position: Vector2,
}

impl Button {
    fn new(position: Vector2) -> Self {
        Self {
            scale: 0.0,
            last_hovered: false,
            position,
        }
    }

    fn update(
        &mut self,
        rl: &mut RaylibHandle,
        asset: &Texture2D,
        audio: &mut RaylibAudio,
        hover_sound: &Sound,
        click_sound: &Sound,
    ) -> bool {
        let hovered = self
            .rect(asset)
            .check_collision_point_rec(rl.get_mouse_position());

        if hovered && !self.last_hovered {
            audio.play_sound(hover_sound);
        }

        self.last_hovered = hovered;
        self.scale += (if hovered { 1.3 } else { 1.0 } * 3.0 - self.scale)
            * (1.0 - 0.5_f32.powf(rl.get_frame_time() / 0.3));

        if hovered && rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            audio.play_sound(click_sound);
            true
        } else {
            false
        }
    }

    fn tl(&self, asset: &Texture2D) -> Vector2 {
        self.position - rvec2(asset.width, asset.height) / 2.0 * self.scale
    }

    fn rect(&self, asset: &Texture2D) -> Rectangle {
        let tl = self.tl(asset);
        rrect(
            tl.x,
            tl.y,
            asset.width as f32 * self.scale,
            asset.height as f32 * self.scale,
        )
    }
}

fn main() -> Result<()> {
    let (mut rl, thread) = raylib::init()
        .size(768, 768)
        .title("Catch the Countdown!")
        .build();
    let mut assets = Assets::load(&mut rl, &thread).context("Failed to load assets!")?;
    let mut music = false;

    {
        let center = rvec2(rl.get_screen_width(), rl.get_screen_height()) / 2.0 - rvec2(0, 80);
        let mut button_play = Button::new(center);
        let mut button_music = Button::new(center + rvec2(0, 150));

        loop {
            if rl.window_should_close() {
                return Ok(());
            }

            if button_play.update(
                &mut rl,
                &assets.button_play,
                &mut assets.audio,
                &assets.button_hover_sound,
                &assets.button_click_sound,
            ) {
                break;
            }

            if button_music.update(
                &mut rl,
                if music {
                    &assets.button_nomusic
                } else {
                    &assets.button_music
                },
                &mut assets.audio,
                &assets.button_hover_sound,
                &assets.button_click_sound,
            ) {
                music = !music;
            }

            let music_asset = if music {
                &assets.button_nomusic
            } else {
                &assets.button_music
            };

            let mut d = rl.begin_drawing(&thread);
            d.draw_texture_ex(
                &assets.title_screen,
                Vector2::zero(),
                0.0,
                3.0,
                Color::WHITE,
            );

            d.draw_texture_ex(
                &assets.button_play,
                button_play.tl(&assets.button_play),
                0.0,
                button_play.scale,
                Color::WHITE,
            );

            d.draw_texture_ex(
                music_asset,
                button_music.tl(music_asset),
                0.0,
                button_music.scale,
                Color::WHITE,
            );
        }
    }

    let (mut level, mut player) = level::Level::load(&assets, 0)
        .context("Failed to load level!")?
        .context("Failed to find first level!")?;
    let mut state = State::Playing;
    let mut time = 0.0;
    loop {
        if rl.window_should_close() {
            return Ok(());
        }

        if rl.is_key_pressed(KeyboardKey::KEY_P) {
            state = match state {
                State::Playing => State::Paused,
                State::Paused => State::Playing,
                state => state,
            };
        }

        if music && !assets.audio.is_sound_playing(&assets.song) {
            assets.audio.play_sound(&assets.song);
        }

        if state != State::Paused {
            time += rl.get_frame_time();
            level.update(&mut rl);
        }
        if state == State::Playing {
            player.update(&mut assets, &mut rl, &mut level, &mut state);
            if player.position().x >= level.size().x {
                state = State::transition(level.index() + 1);
            }
        }

        if let State::LevelTransition {
            next_level,
            timer,
            sound_played,
            loaded,
        } = &mut state
        {
            *timer -= rl.get_frame_time();

            if *timer <= 0.5 && !*sound_played {
                if *next_level != level.index() {
                    assets.audio.play_sound(&assets.next_level_sound);
                }
                *sound_played = true;
            } else if *timer <= 0.0 && !*loaded {
                if let Some((next_level, next_player)) =
                    level::Level::load(&assets, *next_level).context("Failed to load level!")?
                {
                    (level, player) = (next_level, next_player);
                    *loaded = true;
                } else {
                    break;
                }
            }
            if *timer <= -0.5 {
                state = State::Playing;
            }
        }

        let center = rvec2(rl.get_screen_width(), rl.get_screen_height()) / 2.0;
        let mut d = rl.begin_drawing(&thread);
        // d.clear_background(Color::new(86, 86, 86, 255));
        d.draw_texture_ex(&assets.background, Vector2::zero(), 0.0, 3.0, Color::WHITE);
        {
            let mut d = d.begin_mode2D(player.camera(&level));
            level.draw(&assets, &mut d);
            if match state {
                State::Playing => true,
                State::LevelTransition { loaded, .. } => loaded,
                State::Paused => true,
            } {
                player.draw(&assets, &mut d);
            }
            for particle in &level.particles {
                particle.draw(&mut d);
            }
        }

        for overlay in &level.overlays {
            let font_size = (overlay.time.powi(3) * center.y * 14.0) as i32;
            let position = center - rvec2(measure_text(overlay.text(), font_size), font_size) / 2.0;
            d.draw_text(
                overlay.text(),
                position.x as _,
                position.y as _,
                font_size,
                Color::WHITE,
            );
        }

        if let State::LevelTransition { timer, .. } = &state {
            d.draw_rectangle_v(
                Vector2::zero(),
                center * 2.0,
                Color::new(0, 0, 0, 255 - (timer.abs().min(0.5) / 0.5 * 255.0) as u8),
            )
        }

        let mut text = format_time(time);
        if state == State::Paused {
            text.push_str(" (paused)");
        }
        d.draw_text(&text, 10, 10, 20, Color::WHITE);
    }

    let mut timer = 0.0;
    while !rl.window_should_close() {
        timer += rl.get_frame_time();

        fn center_text<D: RaylibDraw>(d: &mut D, text: &str, y: i32, size: i32, color: f32) {
            d.draw_text(
                text,
                400 - measure_text(text, size) / 2,
                y,
                size,
                Color::new(0, 0, 0, (color * 255.0) as u8),
            );
        }

        let mut d = rl.begin_drawing(&thread);
        let brightness = (timer / 2.0).min(1.0);
        d.draw_texture_ex(
            &assets.title_screen,
            Vector2::zero(),
            0.0,
            3.0,
            Color::color_from_normalized(rquat(brightness, brightness, brightness, 1.0)),
        );
        center_text(
            &mut d,
            "Thans for playing!",
            260,
            50,
            (timer - 2.0).clamp(0.0, 1.0),
        );
        center_text(
            &mut d,
            &format!("Your time: {}", format_time(time)),
            320,
            50,
            (timer - 3.0).clamp(0.0, 1.0),
        );
        center_text(
            &mut d,
            "Made for IcoJam 2023",
            380,
            30,
            (timer - 4.0).clamp(0.0, 1.0),
        );
        center_text(
            &mut d,
            "By InfiniteCoder",
            410,
            40,
            (timer - 5.0).clamp(0.0, 1.0),
        );
    }

    Ok(())
}
