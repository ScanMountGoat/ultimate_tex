// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{error::Error, path::PathBuf, sync::Mutex};

use image_dds::{ImageFormat, Mipmaps, Quality};
use serde::Serialize;
use tauri::{CustomMenuItem, Manager, Menu, Submenu, WindowEvent};
use ultimate_tex::{
    convert_to_bntx, convert_to_dds, convert_to_image, convert_to_nutexb, ImageFile, NutexbFile,
};

#[derive(Default)]
struct AppState(Mutex<App>);

#[derive(Default)]
struct App {
    output_folder: Option<PathBuf>,
    save_to_original_folder: bool,
    overrides: FileSettingsOverrides,
    // TODO: Add proper logging.
    message_text: String,
    // Store settings separately to pass to and from javascript.
    // Image data should only ever be accessible from Rust.
    files: Vec<ImageFile>,
    file_settings: Vec<ImageFileSettings>,
}

impl App {
    fn add_files(&mut self, files: &[PathBuf]) {
        // TODO: par_iter?
        for file in files {
            let image = ImageFile::read(&file).unwrap();
            let image_settings = ImageFileSettings::from_image(file.clone(), &image);
            self.files.push(image);
            self.file_settings.push(image_settings);
        }
    }
}

struct FileSettingsOverrides {
    output_file_type: Option<ImageFileType>,
    output_format: Option<ImageFormat>,
    mipmaps: Option<Mipmaps>,
    compression_quality: Option<Quality>,
}

impl Default for FileSettingsOverrides {
    fn default() -> Self {
        // Default to a custom output format to encourage lossless conversions.
        Self {
            output_file_type: Some(ImageFileType::Png),
            output_format: None,
            mipmaps: Some(Mipmaps::GeneratedAutomatic),
            compression_quality: Some(Quality::Fast),
        }
    }
}

#[derive(Debug, Serialize, PartialEq, Clone, Copy)]
enum ImageFileType {
    Dds,
    Png,
    Tiff,
    Nutexb,
    Bntx,
}

impl Default for ImageFileType {
    fn default() -> Self {
        Self::Png
    }
}

impl ImageFileType {
    fn extension(&self) -> &'static str {
        match self {
            ImageFileType::Dds => "dds",
            ImageFileType::Png => "png",
            ImageFileType::Tiff => "tiff",
            ImageFileType::Nutexb => "nutexb",
            ImageFileType::Bntx => "bntx",
        }
    }
}

#[derive(Serialize, Clone)]
struct ImageFileSettings {
    // TODO: Properly implement serialize?
    name: String,
    path: PathBuf,
    output_file_type: ImageFileType,
    #[serde(skip)]
    output_format: ImageFormat,
    #[serde(skip)]
    compression_quality: Quality,
    #[serde(skip)]
    mipmaps: Mipmaps,
}

impl ImageFileSettings {
    fn from_image(path: PathBuf, image: &ImageFile) -> Self {
        // Default to the input format to encourage lossless conversions.
        let output_format = image.image_format();
        ImageFileSettings {
            name: path.file_name().unwrap().to_string_lossy().to_string(),
            path,
            output_file_type: ImageFileType::Nutexb,
            output_format,
            compression_quality: Quality::Fast,
            mipmaps: Mipmaps::GeneratedAutomatic,
        }
    }
}

#[tauri::command]
fn load_items(state: tauri::State<AppState>) -> Vec<ImageFileSettings> {
    // TODO: Is this the best way to pass to javascript?
    state.0.lock().unwrap().file_settings.clone()
}

// TODO: Just do this in svelte for consistency?
fn main_menu() -> Menu {
    let add_files = CustomMenuItem::new("file_add_files", "Add Files...");
    let clear_files = CustomMenuItem::new("file_clear_files", "Clear Files");
    let file = Submenu::new(
        "File",
        Menu::new().add_item(add_files).add_item(clear_files),
    );
    Menu::new().add_submenu(file)
}

fn main() {
    tauri::Builder::default()
        .manage(AppState::default())
        .setup(|app| {
            let main_window = app.get_window("main").unwrap();
            main_window
                .set_title(concat!("Ultimate Tex ", env!("CARGO_PKG_VERSION")))
                .unwrap();

            let handle = app.handle();
            main_window.listen("remove_item", move |event| {
                if let Some(name) = event
                    .payload()
                    .and_then(|s| serde_json::from_str::<String>(s).ok())
                {
                    // TODO: Simpler way to remove the item with the given name?
                    let state = handle.state::<AppState>();
                    let app = &mut state.0.lock().unwrap();
                    if let Some(index) = app.file_settings.iter().position(|s| s.name == name) {
                        app.files.remove(index);
                        app.file_settings.remove(index);

                        handle.emit_to("main", "items_changed", "").unwrap();
                    }
                }
            });

            Ok(())
        })
        .menu(main_menu())
        .on_menu_event(|event| match event.menu_item_id() {
            "file_add_files" => {
                tauri::api::dialog::FileDialogBuilder::default()
                    .add_filter(
                        "image files",
                        &["png", "tiff", "nutexb", "bntx", "jpeg", "jpg", "dds"],
                    )
                    .pick_files(move |files| {
                        if let Some(files) = files {
                            let state = event.window().state::<AppState>();

                            let app = &mut state.0.lock().unwrap();
                            app.add_files(&files);
                            event.window().emit("items_changed", "").unwrap();
                        }
                    });
            }
            "file_clear_files" => {
                let state = event.window().state::<AppState>();
                let mut app = state.0.lock().unwrap();
                app.file_settings.clear();
                app.files.clear();

                event.window().emit("items_changed", "").unwrap();
            }
            _ => (),
        })
        .on_window_event(|event| {
            let state = event.window().state::<AppState>();
            let mut app = state.0.lock().unwrap();

            match event.event() {
                WindowEvent::Resized(_) => {
                    // Workaround for slow Chromium window resizing.
                    // https://github.com/tauri-apps/tauri/issues/6322
                    std::thread::sleep(std::time::Duration::from_nanos(1));
                }
                WindowEvent::FileDrop(e) => match e {
                    tauri::FileDropEvent::Dropped(files) => {
                        app.add_files(files);
                        event.window().emit("items_changed", "").unwrap();
                    }
                    _ => (),
                },
                _ => (),
            }
        })
        .invoke_handler(tauri::generate_handler![load_items])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
