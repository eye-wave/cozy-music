use crate::player::AudioController;

mod library;
mod player;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            crate::library::ipc::load_library,
            crate::player::ipc::player_load_song,
            crate::player::ipc::player_get_position,
            crate::player::ipc::player_set_position,
            crate::player::ipc::player_play,
            crate::player::ipc::player_pause,
            crate::player::ipc::player_stop,
            crate::player::ipc::player_set_volume,
            crate::player::ipc::player_set_playback_speed,
            crate::player::ipc::player_get_props,
        ])
        .manage(AudioController::create()?)
        .run(tauri::generate_context!())?;

    Ok(())
}
