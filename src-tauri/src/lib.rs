use crate::player::AudioController;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod player;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! You've been greeted from Rust!")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, player::ipc::load_song])
        .manage(AudioController::new())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
