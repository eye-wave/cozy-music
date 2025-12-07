use std::sync::Arc;

use iced::widget::svg::Handle;
use iced::widget::{Column, Row, Svg, TextInput, button, column, row, svg};
use iced::{Center, Task};

use crate::player::event::AtomicEvent;
use crate::player::{self, AudioController, SharedAudioBuffer, decode_samples};

pub fn run() -> Result<(), iced::Error> {
    iced::application("Cozy music", CozyApp::update, CozyApp::view)
        .window_size((1200.0, 640.0))
        .run()
}

pub struct CozyApp {
    text_input: Arc<str>,
    player: Option<player::AudioController>,
}

impl Default for CozyApp {
    fn default() -> Self {
        Self {
            text_input: Arc::from(""),
            player: AudioController::create().ok(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    TextInput(Arc<str>),
    Load,
    Loaded(SharedAudioBuffer),
    Error,
    Play,
    Pause,
}

impl CozyApp {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match (message, self.player.as_ref()) {
            (Message::TextInput(text), _) => {
                self.text_input = Arc::clone(&text);
                Task::none()
            }
            (Message::Load, Some(_)) => {
                let path = self.text_input.clone();

                Task::perform(
                    async move {
                        decode_samples(path.as_ref())
                            .map(SharedAudioBuffer::from)
                            .map_err(|e| e.to_string())
                    },
                    |res| match res {
                        Ok(buf) => Message::Loaded(buf),
                        Err(_) => Message::Error,
                    },
                )
            }
            (Message::Loaded(buf), Some(player)) => {
                player.shared_audio.swap(Arc::new(buf));
                println!("Ready!");
                Task::none()
            }
            (Message::Play, Some(player)) => {
                player.send_event(AtomicEvent::Play).ok();
                Task::none()
            }
            (Message::Pause, Some(player)) => {
                player.send_event(AtomicEvent::Pause).ok();
                Task::none()
            }

            _ => Task::none(),
        }
    }

    fn logo() -> Svg<'static> {
        let svg_data = include_bytes!("assets/logo.svg");
        let handle = Handle::from_memory(svg_data);
        svg(handle)
    }

    pub fn view(&self) -> Column<'_, Message> {
        let value = &self.text_input;
        let player: Option<Row<'_, Message>> = self.player.as_ref().map(|player| {
            let is_playing = player.get_is_playing();

            row![
                button("Load").on_press(Message::Load),
                if is_playing {
                    button("Pause").on_press(Message::Pause)
                } else {
                    button("Play").on_press(Message::Play)
                }
            ]
        });

        column![
            TextInput::new("enter path...", value)
                .on_input(|msg| Message::TextInput(Arc::from(msg))),
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
