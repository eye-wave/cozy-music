use super::widgets::player::PlayerWidgetEvent;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Player(PlayerWidgetEvent),
}
