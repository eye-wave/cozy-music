use crate::player::{decode_samples, AudioControler};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod player;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! You've been greeted from Rust!")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let audio = decode_samples(&"src/jonkler.mp3").unwrap();

    println!("{}", audio.sample_rate);

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .manage(AudioControler::new(audio.samples, audio.sample_rate))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
