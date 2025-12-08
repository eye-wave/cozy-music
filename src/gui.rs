use iced::widget::svg::Handle;
use iced::widget::{Column, Svg, Text, column, svg};
use iced::{Center, Element, Subscription, Task};

mod events;
mod widgets;

use crate::gui::events::AppEvent;
use crate::gui::widgets::player::PlayerWidget;
use crate::player::AudioController;

pub fn run() -> Result<(), iced::Error> {
    tracing_subscriber::fmt::init();

    iced::application("Cozy music", CozyApp::update, CozyApp::view)
        .subscription(CozyApp::subscription)
        .window_size((1200.0, 640.0))
        .run()
}

pub struct CozyApp {
    player: Option<AudioController>,
    player_widget: PlayerWidget,
}

impl Default for CozyApp {
    fn default() -> Self {
        Self {
            player: AudioController::create().ok(),
            player_widget: PlayerWidget::default(),
        }
    }
}

impl CozyApp {
    pub fn update(&mut self, event: AppEvent) -> Task<AppEvent> {
        match event {
            AppEvent::Player(event) => {
                if let Some(player) = self.player.as_ref() {
                    return self
                        .player_widget
                        .update(player, event)
                        .map(AppEvent::Player);
                }
            }
        }

        Task::none()
    }

    fn subscription(&self) -> Subscription<AppEvent> {
        PlayerWidget::subscription().map(AppEvent::Player)
    }

    fn logo(&self) -> Svg<'static> {
        let svg_data = include_bytes!("assets/logo.svg");
        let handle = Handle::from_memory(svg_data);
        svg(handle)
    }

    pub fn view(&self) -> Column<'_, AppEvent> {
        let player_view: Element<_> = self
            .player
            .as_ref()
            .map(|p| self.player_widget.view(p).map(AppEvent::Player))
            .unwrap_or_else(|| Text::new("").into());

        column![player_view, self.logo()]
            .padding(20)
            .align_x(Center)
    }
}
