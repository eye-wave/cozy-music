use std::borrow::Cow;
use std::{sync::Arc, time::Duration};

use iced::Alignment::Center;
use iced::widget::{Text, button, column, row, slider};
use iced::{Element, Subscription, Task, time};

use crate::gui::events::AppEvent;
use crate::gui::widgets::gen_svg_icon;
use crate::player::event::{AtomicEvent, AudioEvent};
use crate::player::{AudioController, AudioError, SharedAudioBuffer, decode_samples};

pub struct PlayerWidget {
    song_dur: [u8; 5],
    song_pos: [u8; 5],
}

impl Default for PlayerWidget {
    fn default() -> Self {
        Self {
            song_dur: *b"00:00",
            song_pos: *b"00:00",
        }
    }
}

#[derive(Debug, Clone)]
pub enum PlayerWidgetEvent {
    LoadSong,
    Loaded(SharedAudioBuffer),
    Error(Arc<AudioError>),
    Play,
    Pause,
    Stop,
    Volume(f32),
    Speed(f64),
    Seek(f64),
    SongTick,
}

impl From<PlayerWidgetEvent> for AppEvent {
    fn from(val: PlayerWidgetEvent) -> Self {
        AppEvent::Player(val)
    }
}

impl PlayerWidget {
    const PLAY_ICON: &[u8] = include_bytes!("../../assets/icon-play.svg");
    const PAUSE_ICON: &[u8] = include_bytes!("../../assets/icon-pause.svg");
    const STOP_ICON: &[u8] = include_bytes!("../../assets/icon-stop.svg");

    pub fn update(
        &mut self,
        player: &AudioController,
        event: PlayerWidgetEvent,
    ) -> Task<PlayerWidgetEvent> {
        match event {
            PlayerWidgetEvent::LoadSong => {
                return Task::perform(
                    async move {
                        decode_samples("/home/eyewave/Music/my-ids/400_eyewave.mp3")
                            .map(SharedAudioBuffer::from)
                    },
                    |res| match res {
                        Ok(res) => PlayerWidgetEvent::Loaded(res),
                        Err(err) => PlayerWidgetEvent::Error(Arc::new(err.into())),
                    },
                );
            }
            PlayerWidgetEvent::Loaded(buf) => {
                player.shared_audio.swap(Arc::new(buf));
                self.song_dur = get_song_duration_pretty(player);
            }
            PlayerWidgetEvent::Play => {
                player.send_event(AtomicEvent::Play);

                if player.get_song_duration() < 1 {
                    return Task::perform(async {}, |_| PlayerWidgetEvent::LoadSong);
                }
            }
            PlayerWidgetEvent::Pause => {
                player.send_event(AtomicEvent::Pause);
            }
            PlayerWidgetEvent::Stop => {
                player.send_event(AudioEvent::Stop);
            }
            PlayerWidgetEvent::Volume(vol) => {
                player.send_event(AtomicEvent::SetVolume(vol));
            }
            PlayerWidgetEvent::Speed(s) => {
                player.send_event(AtomicEvent::SetSpeed(s));
            }
            PlayerWidgetEvent::Seek(pos) => player.set_position(pos),
            PlayerWidgetEvent::SongTick => self.song_pos = get_song_position_pretty(player),
            PlayerWidgetEvent::Error(err) => eprintln!("{err:?}"),
        }

        Task::none()
    }

    pub fn subscription() -> Subscription<PlayerWidgetEvent> {
        time::every(Duration::from_millis(100)).map(|_| PlayerWidgetEvent::SongTick)
    }

    fn get_time(&self) -> (Cow<'_, str>, Cow<'_, str>) {
        (
            Cow::Borrowed(std::str::from_utf8(&self.song_dur).unwrap()),
            Cow::Borrowed(std::str::from_utf8(&self.song_pos).unwrap()),
        )
    }

    pub fn view(&self, player: &AudioController) -> Element<'_, PlayerWidgetEvent> {
        let time = player.get_song_position_percent() * 100.0;
        let volume = player.get_volume() * 100.0;
        let speed = player.get_speed();

        let (duration, position) = self.get_time();

        column![
            row![
                match player.get_is_playing() {
                    true => button(gen_svg_icon(Self::PAUSE_ICON))
                        .on_press(PlayerWidgetEvent::Pause)
                        .width(40),
                    false => button(gen_svg_icon(Self::PLAY_ICON))
                        .on_press(PlayerWidgetEvent::Play)
                        .width(40),
                },
                button(gen_svg_icon(Self::STOP_ICON))
                    .on_press(PlayerWidgetEvent::Stop)
                    .width(40),
                column![
                    row![
                        slider(0.0..=100.0, volume, |v| PlayerWidgetEvent::Volume(v * 0.01))
                            .step(1.0)
                            .width(80),
                        Text::new(format!("{volume:.0}%"))
                    ],
                    row![
                        slider(0.5..=2.0, speed, PlayerWidgetEvent::Speed)
                            .step(0.01)
                            .width(80),
                        Text::new(format!("x{speed:.2}"))
                    ],
                ]
            ]
            .align_y(Center)
            .spacing(12),
            row![
                Text::new(position),
                slider(0.0..=100.0, time, |v| PlayerWidgetEvent::Seek(v * 0.01)),
                Text::new(duration),
            ]
            .spacing(12)
            .align_y(Center)
        ]
        .align_x(Center)
        .max_width(800)
        .into()
    }
}

fn get_song_position_pretty(player: &AudioController) -> [u8; 5] {
    let pos = player.get_song_position();
    let (pm, ps) = format_sample_time(pos, player.sample_rate());

    let mut buffer = *b"00:00";

    buffer[0..2].copy_from_slice(&pad_start(pm));
    buffer[3..5].copy_from_slice(&pad_start(ps));

    buffer
}

fn get_song_duration_pretty(player: &AudioController) -> [u8; 5] {
    let buffer = player.shared_audio.load();
    let duration = buffer.duration();

    let (dm, ds) = format_sample_time(duration as f64, buffer.sample_rate);

    let mut buffer = *b"00:00";

    buffer[0..2].copy_from_slice(&pad_start(dm));
    buffer[3..5].copy_from_slice(&pad_start(ds));

    buffer
}

fn format_sample_time(samples: f64, sample_rate: u32) -> (u8, u8) {
    let total_secs = samples / sample_rate as f64;
    let min = (total_secs / 60.0).floor() as u8;
    let s = total_secs - (min as f64 * 60.0);

    (min, s as u8)
}

fn pad_start(num: u8) -> [u8; 2] {
    [((num / 10) % 10) + 48, (num % 10) + 48]
}
