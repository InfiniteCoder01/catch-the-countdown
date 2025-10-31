pub use anyhow::*;
use ldtk_easy::project::Project;
pub use raylib::prelude::*;

pub fn tuple2<T1: misc::AsF32, T2: misc::AsF32>(tuple: (T1, T2)) -> Vector2 {
    rvec2(tuple.0, tuple.1)
}

pub struct Assets {
    pub world: Project,
    pub player: Texture2D,
    pub tileset: Texture2D,
    pub numbers: Texture2D,
    pub spider: Texture2D,
    pub web: Texture2D,
    pub door: Texture2D,
    pub title_screen: Texture2D,
    pub button_play: Texture2D,
    pub button_music: Texture2D,
    pub button_nomusic: Texture2D,
    pub background: Texture2D,

    pub audio: RaylibAudio,
    pub jump_sound: Sound,
    pub number_sound: Sound,
    pub game_over_sound: Sound,
    pub next_level_sound: Sound,
    pub button_hover_sound: Sound,
    pub button_click_sound: Sound,
    pub song: Sound,
}

impl Assets {
    pub fn load(rl: &mut RaylibHandle, thread: &RaylibThread) -> Result<Self> {
        let world =
            Project::new(include_str!("../levels.ldtk")).map_err(|err| anyhow!(err.message))?;

        Ok(Self {
            world,
            player: rl
                .load_texture(thread, "Assets/Player.png")
                .map_err(|err| anyhow!(err))?,
            tileset: rl
                .load_texture(thread, "Assets/Tileset.png")
                .map_err(|err| anyhow!(err))?,
            numbers: rl
                .load_texture(thread, "Assets/Numbers.png")
                .map_err(|err| anyhow!(err))?,
            spider: rl
                .load_texture(thread, "Assets/Spider.png")
                .map_err(|err| anyhow!(err))?,
            web: rl
                .load_texture(thread, "Assets/Web.png")
                .map_err(|err| anyhow!(err))?,
            door: rl
                .load_texture(thread, "Assets/Door.png")
                .map_err(|err| anyhow!(err))?,
            title_screen: rl
                .load_texture(thread, "Assets/TitleScreen.png")
                .map_err(|err| anyhow!(err))?,
            button_play: rl
                .load_texture(thread, "Assets/ButtonPlay.png")
                .map_err(|err| anyhow!(err))?,
            button_music: rl
                .load_texture(thread, "Assets/ButtonMusic.png")
                .map_err(|err| anyhow!(err))?,
            button_nomusic: rl
                .load_texture(thread, "Assets/ButtonNoMusic.png")
                .map_err(|err| anyhow!(err))?,
            background: rl
                .load_texture(thread, "Assets/Background.png")
                .map_err(|err| anyhow!(err))?,

            audio: RaylibAudio::init_audio_device(),
            jump_sound: Sound::load_sound("Assets/Jump.wav").map_err(|err| anyhow!(err))?,
            number_sound: Sound::load_sound("Assets/Number.wav").map_err(|err| anyhow!(err))?,
            game_over_sound: Sound::load_sound("Assets/GameOver.wav")
                .map_err(|err| anyhow!(err))?,
            next_level_sound: Sound::load_sound("Assets/NextLevel.wav")
                .map_err(|err| anyhow!(err))?,
            button_hover_sound: Sound::load_sound("Assets/ButtonHover.wav")
                .map_err(|err| anyhow!(err))?,
            button_click_sound: Sound::load_sound("Assets/ButtonClick.wav")
                .map_err(|err| anyhow!(err))?,
            song: Sound::load_sound("Assets/Song.wav").map_err(|err| anyhow!(err))?,
        })
    }
}

#[derive(PartialEq)]
pub enum State {
    Playing,
    LevelTransition {
        next_level: usize,
        timer: f32,
        sound_played: bool,
        loaded: bool,
    },
    Paused,
}

impl State {
    pub fn transition(next_level: usize) -> Self {
        Self::LevelTransition {
            next_level,
            timer: 1.0,
            sound_played: false,
            loaded: false,
        }
    }
}
