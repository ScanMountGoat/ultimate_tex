// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    error::Error,
    path::{Path, PathBuf},
    sync::Mutex,
};

use image_dds::{ImageFormat, Mipmaps, Quality};
use serde::{Deserialize, Serialize};
use tauri::{CustomMenuItem, Manager, Menu, Submenu, WindowEvent};
use ultimate_tex::{
    convert_to_bntx, convert_to_dds, convert_to_image, convert_to_nutexb, ImageFile,
};

#[derive(Default)]
struct AppState(Mutex<App>);

// TODO: Add proper logging using events?
#[derive(Default)]
struct App {
    // Store settings separately to pass to and from javascript.
    // Image data should only ever be accessible from Rust.
    settings: AppSettings,
    files: Vec<ImageFile>,
}

impl App {
    fn add_files(&mut self, files: &[PathBuf]) {
        // TODO: par_iter?
        for file in files {
            let image = ImageFile::read(&file).unwrap();
            let image_settings = ImageFileSettings::from_image(file.clone(), &image);
            self.files.push(image);
            self.settings.file_settings.push(image_settings);
        }
    }

    fn clear_files(&mut self) {
        self.files.clear();
        self.settings.file_settings.clear();
    }

    fn convert_and_export_files(&self) -> Result<usize, Box<dyn Error>> {
        // TODO: Log an error if creating the output directory fails.
        if let Some(output_folder) = &self.settings.output_folder {
            std::fs::create_dir_all(output_folder)?;
        }

        let start = std::time::Instant::now();

        // TODO: report progress?
        let count = self
            .settings
            .file_settings
            .iter()
            .zip(self.files.iter())
            .map(|(settings, file)| {
                // TODO: find a simpler way to write this.
                if let Some(file_output_folder) = if self.settings.save_to_original_folder {
                    settings.path.parent().map(PathBuf::from)
                } else {
                    self.settings.output_folder.to_owned()
                } {
                    match convert_and_save_file(
                        &file_output_folder,
                        settings,
                        file,
                        &self.settings.overrides,
                    ) {
                        Ok(_) => 1,
                        Err(e) => {
                            // TODO: Log errors.
                            println!("Error converting {:?}: {e}", settings.path);
                            0
                        }
                    }
                } else {
                    0
                }
            })
            .sum();

        println!(
            "Converted {} files in {:?}",
            self.files.len(),
            start.elapsed()
        );

        Ok(count)
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct AppSettings {
    output_folder: Option<PathBuf>,
    save_to_original_folder: bool,
    overrides: FileSettingsOverrides,
    file_settings: Vec<ImageFileSettings>,
}

#[derive(Serialize, Deserialize, Clone)]
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

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy)]
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

#[derive(Serialize, Deserialize, Clone)]
struct ImageFileSettings {
    // TODO: Properly implement serialize?
    name: String,
    path: PathBuf,
    format: ImageFormat,
    dimensions: (u32, u32, u32),
    output_file_type: ImageFileType,
    output_format: ImageFormat,
    output_quality: Quality,
    output_mipmaps: Mipmaps,
}

impl ImageFileSettings {
    fn from_image(path: PathBuf, image: &ImageFile) -> Self {
        // Default to the input format to encourage lossless conversions.
        let format = image.image_format();
        ImageFileSettings {
            name: path.file_name().unwrap().to_string_lossy().to_string(),
            path,
            format,
            dimensions: image.dimensions(),
            output_file_type: ImageFileType::Nutexb,
            output_format: format,
            output_quality: Quality::Fast,
            output_mipmaps: Mipmaps::GeneratedAutomatic,
        }
    }

    fn file_name_no_extension(&self) -> String {
        // TODO: Avoid unwrap.
        self.path
            .with_extension("")
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }
}

#[tauri::command]
fn load_items(state: tauri::State<AppState>) -> Vec<ImageFileSettings> {
    // TODO: Is this the best way to pass to javascript?
    state.0.lock().unwrap().settings.file_settings.clone()
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

fn convert_and_save_file(
    output_folder: &Path,
    file: &ImageFileSettings,
    image_file: &ImageFile,
    overrides: &FileSettingsOverrides,
) -> Result<(), Box<dyn Error>> {
    // Global overrides take priority over file specific settings if enabled.
    let file_type = overrides.output_file_type.unwrap_or(file.output_file_type);
    let format = overrides.output_format.unwrap_or(file.output_format);
    let quality = overrides.compression_quality.unwrap_or(file.output_quality);
    let mipmaps = overrides.mipmaps.unwrap_or(file.output_mipmaps);

    let output = output_folder
        .join(file.file_name_no_extension())
        .with_extension(file_type.extension());

    match file_type {
        ImageFileType::Dds => convert_to_dds(image_file, &output, format, quality, mipmaps)?,
        ImageFileType::Png => convert_to_image(image_file, &output)?,
        ImageFileType::Tiff => convert_to_image(image_file, &output)?,
        ImageFileType::Nutexb => convert_to_nutexb(image_file, &output, format, quality, mipmaps)?,
        ImageFileType::Bntx => convert_to_bntx(image_file, &output, format, quality, mipmaps)?,
    }
    Ok(())
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
                    if let Some(index) = app
                        .settings
                        .file_settings
                        .iter()
                        .position(|s| s.name == name)
                    {
                        app.files.remove(index);
                        app.settings.file_settings.remove(index);

                        handle.emit_to("main", "items_changed", "").unwrap();
                    }
                }
            });

            let handle = app.handle();
            main_window.listen("export_items", move |event| {
                let state = handle.state::<AppState>();
                let app = &mut state.0.lock().unwrap();
                // TODO: Log errors.
                app.convert_and_export_files().unwrap();
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
                app.clear_files();

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
