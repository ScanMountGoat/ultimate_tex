#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use serde::Serialize;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[derive(Serialize, Clone)]
struct Item {
    name: String,
    dimensions: (u32, u32, u32),
    format: String,
    file_type: String,
    quality: String,
    mipmaps: String,
}

#[tauri::command]
fn load_items() -> Vec<Item> {
    vec![
        Item {
            name: "def_mario_00.tiff".into(),
            dimensions: (64, 64, 1),
            format: "Bc7Unorm".to_string(),
            file_type: "Nutexb".to_string(),
            quality: "Fast".to_string(),
            mipmaps: "None".to_string(),
        };
        30
    ]
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![load_items])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
