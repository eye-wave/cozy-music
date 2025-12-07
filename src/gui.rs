use std::sync::Arc;
use std::time::Duration;

use iced::widget::svg::Handle;
use iced::widget::{Column, Row, Svg, Text, button, column, row, svg};
use iced::{Center, Subscription, Task, time};

mod events;
mod widgets;

use crate::gui::events::{AppEvent, PlayerEvent, UiEvent};
use crate::player::event::AtomicEvent;
use crate::player::{self, AudioController, SharedAudioBuffer, decode_samples};

pub fn run() -> Result<(), iced::Error> {
    tracing_subscriber::fmt::init();

    iced::application("Cozy music", CozyApp::update, CozyApp::view)
        .subscription(CozyApp::subscription)
        .window_size((1200.0, 640.0))
        .run()
}

pub struct CozyApp {
    player: Option<player::AudioController>,
}

impl Default for CozyApp {
    fn default() -> Self {
        Self {
            player: AudioController::create().ok(),
        }
    }
}

impl CozyApp {
    pub fn update(&mut self, event: AppEvent) -> Task<AppEvent> {
        match event {
            AppEvent::Player(event) => {
                if let Some(player) = self.player.as_ref() {
                    match event {
                        PlayerEvent::Loaded(buf) => {
                            player.shared_audio.swap(Arc::new(buf));
                        }
                        PlayerEvent::Play => {
                            player.send_event(AtomicEvent::Play).ok();
                        }
                        PlayerEvent::Pause => {
                            player.send_event(AtomicEvent::Pause).ok();
                        }
                        PlayerEvent::Error(err) => eprintln!("{err:?}"),
                    }
                }

                Task::none()
            }

            AppEvent::Ui(UiEvent::LoadSong) => Task::perform(
                async move {
                    decode_samples("/home/eyewave/Music/my-ids/400_eyewave.mp3")
                        .map(SharedAudioBuffer::from)
                },
                |res| match res {
                    Ok(res) => PlayerEvent::Loaded(res).into(),
                    Err(err) => PlayerEvent::Error(Arc::new(err.into())).into(),
                },
            ),

            _ => Task::none(),
        }
    }

    fn subscription(&self) -> Subscription<AppEvent> {
        time::every(Duration::from_millis(100)).map(|_| UiEvent::SongTick.into())
    }

    fn logo() -> Svg<'static> {
        let svg_data = include_bytes!("assets/logo.svg");
        let handle = Handle::from_memory(svg_data);
        svg(handle)
    }

    pub fn view(&self) -> Column<'_, AppEvent> {
        let player: Option<Row<'_, AppEvent>> = self.player.as_ref().map(|player| {
            let is_playing = player.get_is_playing();
            let time = player.get_song_position_string();

            row![
                Text::new(time),
                button("Load").on_press(UiEvent::LoadSong.into()),
                if is_playing {
                    button("Pause").on_press(PlayerEvent::Pause.into())
                } else {
                    button("Play").on_press(PlayerEvent::Play.into())
                }
            ]
        });

        column![
            if let Some(player) = player {
                player
            } else {
                row![]
            },
            Self::logo()
        ]
        .padding(20)
        .align_x(Center)
    }
}
