use std::borrow::Cow;
use std::{sync::Arc, time::Duration};

use iced::Alignment::Center;
use iced::widget::{Text, button, column, row, slider, vertical_slider};
use iced::{Element, Subscription, Task, time};

use crate::gui::events::AppEvent;
use crate::player::event::AtomicEvent;
use crate::player::{AudioController, AudioError, SharedAudioBuffer, decode_samples};

pub struct PlayerWidget {
    time: [u8; 13],
}

impl Default for PlayerWidget {
    fn default() -> Self {
        Self {
            time: *b"00:00 / 00:00",
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
    Volume(f32),
    Seek(f64),
    SongTick,
}

impl From<PlayerWidgetEvent> for AppEvent {
    fn from(val: PlayerWidgetEvent) -> Self {
        AppEvent::Player(val)
    }
}

impl PlayerWidget {
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
            PlayerWidgetEvent::Volume(vol) => {
                player.send_event(AtomicEvent::SetVolume(vol));
            }
            PlayerWidgetEvent::Seek(pos) => player.set_position(pos),
            PlayerWidgetEvent::SongTick => self.time = player.get_song_position_pretty(),
            PlayerWidgetEvent::Error(err) => eprintln!("{err:?}"),
        }

        Task::none()
    }

    pub fn subscription() -> Subscription<PlayerWidgetEvent> {
        time::every(Duration::from_millis(100)).map(|_| PlayerWidgetEvent::SongTick)
    }

    fn get_time(&self) -> Cow<'_, str> {
        Cow::Borrowed(std::str::from_utf8(&self.time).unwrap())
    }

    pub fn view(&self, player: &AudioController) -> Element<'_, PlayerWidgetEvent> {
        let time = player.get_song_position_percent() * 100.0;
        let volume = player.get_volume() * 100.0;

        column![
            row![
                match player.get_is_playing() {
                    true => button("Pause").on_press(PlayerWidgetEvent::Pause),
                    false => button("Play").on_press(PlayerWidgetEvent::Play),
                },
                Text::new(self.get_time()),
                vertical_slider(0.0..=100.0, volume, |v| PlayerWidgetEvent::Volume(v * 0.01))
                    .height(30)
            ]
            .align_y(Center),
            row![slider(0.0..=100.0, time, |v| PlayerWidgetEvent::Seek(
                v * 0.01
            ))]
        ]
        .max_width(320)
        .into()
    }
}
