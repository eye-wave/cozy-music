use crate::player::AudioController;

mod player;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! You've been greeted from Rust!")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            greet,
            player::ipc::load_song,
            player::ipc::player_play,
            player::ipc::player_pause,
            player::ipc::player_stop,
        ])
        .manage(AudioController::create())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
