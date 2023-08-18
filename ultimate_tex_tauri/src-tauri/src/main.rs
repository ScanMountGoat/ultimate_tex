// Prevents additional console window on Windows in release, DO NOT REMOVE!!
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    error::Error,
    path::{Path, PathBuf},
    sync::Mutex,
};

use image_dds::{ImageFormat, Mipmaps, Quality};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use tauri::{Manager, WindowEvent};
use ultimate_tex::{
    convert_to_bntx, convert_to_dds, convert_to_image, convert_to_nutexb, ImageFile, NutexbFile,
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
        let start = std::time::Instant::now();

        // Only the expensive file reading benefits from parallelism.
        let new_files: Vec<_> = files
            .par_iter()
            .map(|file| ImageFile::read(file).unwrap())
            .collect();
        for (file, image) in files.iter().zip(new_files.iter()) {
            self.settings
                .file_settings
                .push(ImageFileSettings::from_image(file.clone(), image));
        }
        self.files.extend(new_files);

        println!("Added {} files in {:?}", files.len(), start.elapsed());
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
            .par_iter()
            .zip(self.files.par_iter())
            .filter_map(|(settings, file)| {
                let output = if self.settings.save_in_same_folder {
                    settings.path.parent()
                } else {
                    self.settings.output_folder.as_deref()
                };

                output.and_then(|output| {
                    convert_and_save_file(output, settings, file, &self.settings.overrides).ok()
                })
            })
            .count();

        println!(
            "Converted {} files in {:?}",
            self.files.len(),
            start.elapsed()
        );

        Ok(count)
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
struct AppSettings {
    output_folder: Option<PathBuf>,
    save_in_same_folder: bool,
    overrides: FileSettingsOverrides,
    file_settings: Vec<ImageFileSettings>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
struct ImageFileSettings {
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
fn load_files(state: tauri::State<AppState>) -> Vec<ImageFileSettings> {
    // TODO: Is this the best way to pass to javascript?
    state.0.lock().unwrap().settings.file_settings.clone()
}

#[tauri::command(async)]
async fn export_files(
    settings: AppSettings,
    handle: tauri::AppHandle,
) -> Result<usize, tauri::Error> {
    // TODO: Return information to JS for displaying in a bottom bar?
    tauri::async_runtime::spawn_blocking(move || {
        // TODO: Is it worth storing settings in Rust if we get it from JS anyway?
        // TODO: Log errors.
        let state = handle.state::<AppState>();
        let app = &mut state.0.lock().unwrap();
        app.settings = settings;
        // TODO: Avoid unwrap?
        app.convert_and_export_files().unwrap()
    })
    .await
}

#[tauri::command]
fn add_files(handle: tauri::AppHandle) {
    tauri::api::dialog::FileDialogBuilder::default()
        .add_filter(
            "image files",
            &["png", "tiff", "nutexb", "bntx", "jpeg", "jpg", "dds"],
        )
        .pick_files(move |files| {
            if let Some(files) = files {
                let state = handle.state::<AppState>();

                let app = &mut state.0.lock().unwrap();

                app.add_files(&files);
                handle.emit_all("files_changed", "").unwrap();
            }
        });
}

#[tauri::command]
fn clear_files(handle: tauri::AppHandle) {
    let state = handle.state::<AppState>();
    let app = &mut state.0.lock().unwrap();

    app.clear_files();
    handle.emit_all("files_changed", "").unwrap();
}

#[tauri::command]
fn optimize_nutexb() {
    tauri::api::dialog::FileDialogBuilder::default()
        .set_title("Select Nutexb Root Folder")
        .pick_folder(|folder| {
            if let Some(folder) = folder {
                optimize_nutexb_files_recursive(&folder);
            }
        });
}

#[tauri::command]
fn select_output_folder(handle: tauri::AppHandle) -> Option<PathBuf> {
    // TODO: how to return the selected folder to JS?
    // TODO: Just emit an event to indicate the value changed?
    tauri::api::dialog::FileDialogBuilder::default()
        .set_title("Select Output Folder")
        .pick_folder(move |folder| {
            if let Some(folder) = folder {
                let state = handle.state::<AppState>();
                let app = &mut state.0.lock().unwrap();
                app.settings.output_folder = Some(folder);
            }
        });

    None
}

#[tauri::command]
fn open_wiki() {
    if let Err(_) = open::that("https://github.com/ScanMountGoat/ultimate_tex/wiki") {
        // TODO: log errors
    }
}

fn optimize_nutexb_files_recursive(root: &Path) {
    for entry in globwalk::GlobWalkerBuilder::from_patterns(root, &["*.{nutexb}"])
        .build()
        .unwrap()
        .filter_map(Result::ok)
    {
        if let Ok(mut nutexb) = NutexbFile::read_from_file(entry.path()) {
            nutexb.optimize_size();
            // TODO: Avoid unwrap.
            if let Err(_) = nutexb.write_to_file(entry.path()) {
                // TODO: log errors
            }
        }
    }
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
                if let Some(index) = event
                    .payload()
                    .and_then(|s| serde_json::from_str::<usize>(s).ok())
                {
                    let state = handle.state::<AppState>();
                    let app = &mut state.0.lock().unwrap();

                    app.files.remove(index);
                    app.settings.file_settings.remove(index);
                    handle.emit_to("main", "files_changed", "").unwrap();
                }
            });

            Ok(())
        })
        .on_window_event(|event| {
            match event.event() {
                WindowEvent::Resized(_) => {
                    // Workaround for slow Chromium window resizing.
                    // https://github.com/tauri-apps/tauri/issues/6322
                    std::thread::sleep(std::time::Duration::from_nanos(1));
                }
                WindowEvent::FileDrop(tauri::FileDropEvent::Dropped(new_files)) => {
                    let state = event.window().state::<AppState>();
                    let app = &mut state.0.lock().unwrap();

                    app.add_files(new_files);
                    event.window().emit("files_changed", "").unwrap();
                }
                _ => (),
            }
        })
        .invoke_handler(tauri::generate_handler![
            add_files,
            clear_files,
            export_files,
            load_files,
            open_wiki,
            optimize_nutexb,
            select_output_folder
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
