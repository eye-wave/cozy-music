use crate::player::AudioController;

mod player;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! You've been greeted from Rust!")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            greet,
            crate::player::ipc::load_song,
            crate::player::ipc::get_samplerate,
            crate::player::ipc::get_position,
            crate::player::ipc::player_play,
            crate::player::ipc::player_pause,
            crate::player::ipc::player_stop,
            crate::player::ipc::player_set_volume,
            crate::player::ipc::player_set_playback_speed,
        ])
        .manage(AudioController::create()?)
        .run(tauri::generate_context!())?;

    Ok(())
}
