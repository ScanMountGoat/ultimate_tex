// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Serialize;
use tauri::{CustomMenuItem, Manager, Menu, Submenu, WindowEvent};

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
            format: "BC7Unorm".to_string(),
            file_type: "Nutexb".to_string(),
            quality: "Fast".to_string(),
            mipmaps: "Disabled".to_string(),
        };
        30
    ]
}

// TODO: Just do this in svelte for consistency?
fn main_menu() -> Menu {
    let add_files = CustomMenuItem::new("add_files", "Add Files...");
    let clear_files = CustomMenuItem::new("clear_files", "Clear Files");
    let file = Submenu::new(
        "File",
        Menu::new().add_item(add_files).add_item(clear_files),
    );
    Menu::new().add_submenu(file)
}

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            app.get_window("main")
                .unwrap()
                .set_title(concat!("Ultimate Tex ", env!("CARGO_PKG_VERSION")))
                .unwrap();
            Ok(())
        })
        .menu(main_menu())
        .on_window_event(|e| {
            // Workaround for slow Chromium window resizing.
            // https://github.com/tauri-apps/tauri/issues/6322
            if let WindowEvent::Resized(_) = e.event() {
                std::thread::sleep(std::time::Duration::from_nanos(1));
            }
        })
        .invoke_handler(tauri::generate_handler![load_items])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
