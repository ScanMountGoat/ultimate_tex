// Hide the console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    error::Error,
    path::{Path, PathBuf},
};

use eframe::egui::{self, Button, ComboBox, Grid, Id, ScrollArea, Style, Visuals};
use egui_extras::{Column, TableBuilder, TableRow};
use image_dds::{ImageFormat, Mipmaps, Quality};
use rayon::prelude::*;
// use rfd::FileDialog;
use strum::IntoEnumIterator;
use theme::widgets_dark;
use ultimate_tex::{
    convert_to_bntx, convert_to_dds, convert_to_image, convert_to_nutexb, ImageFile, NutexbFile,
};

mod theme;

#[derive(Default)]
struct App {
    output_folder: Option<PathBuf>,
    save_to_original_folder: bool,
    files: Vec<ImageFileSettings>,
    overrides: FileSettingsOverrides,
    // TODO: Add proper logging.
    message_text: String,
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

#[derive(PartialEq, Clone, Copy, strum::Display, strum::EnumIter)]
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

struct ImageFileSettings {
    path: PathBuf,
    image_file: ImageFile,
    output_file_type: ImageFileType,
    output_format: ImageFormat,
    compression_quality: Quality,
    mipmaps: image_dds::Mipmaps,
}

impl ImageFileSettings {
    fn from_path(path: PathBuf) -> Result<Self, Box<dyn Error>> {
        let image_file = ImageFile::read(&path)?;
        // Default to the input format to encourage lossless conversions.
        let output_format = image_file.image_format();
        Ok(ImageFileSettings {
            path,
            image_file,
            output_file_type: ImageFileType::Nutexb,
            output_format,
            compression_quality: Quality::Fast,
            mipmaps: Mipmaps::GeneratedAutomatic,
        })
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

    fn extension(&self) -> &str {
        self.path
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Allow loading files by dragging and dropping onto the window.
        // TODO: use rayon here as well?
        ctx.input(|i| {
            for file in &i.raw.dropped_files {
                if let Some(path) = &file.path {
                    if let Ok(new_file) = ImageFileSettings::from_path(path.clone()) {
                        self.files.push(new_file);
                    }
                }
            }
        });

        egui::TopBottomPanel::top("menu_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Add Files...").clicked() {
                        ui.close_menu();

                        // if let Some(files) = FileDialog::new()
                        //     .add_filter(
                        //         "image files",
                        //         &["png", "tiff", "nutexb", "bntx", "jpeg", "jpg", "dds"],
                        //     )
                        //     .pick_files()
                        // {
                        //     let start = std::time::Instant::now();

                        //     let new_files = files.into_par_iter().filter_map(|f| ImageFileSettings::from_path(f).ok());
                        //     self.files.par_extend(new_files);

                        //     println!("Loaded files in {:?}", start.elapsed());
                        // }
                    }

                    if ui.button("Clear Files").clicked() {
                        ui.close_menu();

                        self.files.clear();
                    }
                });

                ui.menu_button("Batch", |ui| {
                    if ui
                        .add(Button::new("Optimize Nutexb Padding...").wrap(false))
                        .on_hover_text(
                            "Optimize nutexb surface size for all folders and subfolders recursively",
                        )
                        .clicked()
                    {
                        ui.close_menu();

                        // if let Some(folder) = FileDialog::new().pick_folder() {
                        //     optimize_nutexb_files_recursive(&folder);
                        //     // TODO: Show how many files were optimized in the bottom bar?
                        //     // TODO: Log errors to the bottom bar?
                        // }
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("Wiki").clicked() {
                        ui.close_menu();

                        if let Err(_) = open::that("https://github.com/ScanMountGoat/ultimate_tex/wiki") {
                            // TODO: log errors
                        }
                    }
                });
            })
        });

        egui::TopBottomPanel::top("output_panel").show(ctx, |ui| {
            ui.heading("Output");

            ui.checkbox(&mut self.save_to_original_folder, "Save to Original Folder");

            if !self.save_to_original_folder {
                ui.horizontal(|ui| {
                    ui.label("Output Location");
                    if let Some(output_folder) = &self.output_folder {
                        ui.label(output_folder.to_string_lossy());
                    }
                    if ui.button("Open...").clicked() {
                        // if let Some(folder) = FileDialog::new().pick_folder() {
                        //     self.output_folder = Some(folder);
                        // }
                    }
                });
            }

            // Exporting should only be enabled once an export folder is selected.
            let can_export = self.output_folder.is_some() || self.save_to_original_folder;
            if ui
                .add_enabled_ui(can_export, |ui| {
                    // Make the button larger and easier to click.
                    ui.add_sized(egui::vec2(80.0, 30.0), Button::new("Export"))
                })
                .inner
                .on_disabled_hover_text("Select an output folder.")
                .clicked()
            {
                // TODO: Spawn a thread to process the files.
                // TODO: Update progress using a callback?
                if let Ok(count) = convert_and_export_files(
                    &self.files,
                    &self.output_folder,
                    &self.overrides,
                    self.save_to_original_folder,
                ) {
                    self.message_text = format!(
                        "Successfully converted {count} of {} file(s)",
                        self.files.len()
                    );
                }
                // TODO: Use log for showing the messages?
            }
            horizontal_separator_empty(ui);
        });

        egui::TopBottomPanel::bottom("progress_panel").show(ctx, |ui| {
            ui.label(&self.message_text);
            // TODO: Track progress for exporting.
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Files");
            horizontal_separator_empty(ui);

            if !self.files.is_empty() {
                settings_presets(ui, &mut self.overrides);
                horizontal_separator_empty(ui);
            }

            ScrollArea::horizontal()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    if !self.files.is_empty() {
                        files_table(ui, &mut self.files, &self.overrides);
                    } else {
                        ui.label("Drag and drop image files onto the window or add files using File > Add Files...");
                    }
                });
        });
    }
}

fn settings_presets(ui: &mut egui::Ui, overrides: &mut FileSettingsOverrides) {
    // Pad the first column to visually separate the labels.
    Grid::new("presets_grid")
        .min_col_width(150.0)
        .show(ui, |ui| {
            ui.label("Output Type");
            ui.horizontal(|ui| {
                ui.radio_value(
                    &mut overrides.output_file_type,
                    Some(ImageFileType::Png),
                    "PNG",
                );
                ui.radio_value(
                    &mut overrides.output_file_type,
                    Some(ImageFileType::Dds),
                    "DDS",
                );
                ui.radio_value(
                    &mut overrides.output_file_type,
                    Some(ImageFileType::Nutexb),
                    "Nutexb",
                );
                ui.radio_value(
                    &mut overrides.output_file_type,
                    Some(ImageFileType::Bntx),
                    "Bntx",
                );
                ui.radio_value(&mut overrides.output_file_type, None, "Custom...");
            });
            ui.end_row();

            // Uncompressed formats don't need encoding settings.
            // Allow these settings when selecting formats manually.
            let show_settings = overrides
                .output_file_type
                .map(file_supports_compression)
                .unwrap_or(true);

            if show_settings {
                ui.label("Output Format");
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut overrides.output_format,
                        Some(ImageFormat::BC7Srgb),
                        "Color (sRGB) + Alpha",
                    )
                    .on_hover_text(
                        "Recommended for most color textures like col, emi, or diffuse.",
                    );

                    ui.radio_value(
                        &mut overrides.output_format,
                        Some(ImageFormat::BC7Unorm),
                        "Color (Linear) + Alpha",
                    )
                    .on_hover_text("Recommended for nor and prm maps.");

                    ui.radio_value(&mut overrides.output_format, None, "Custom...");
                });
                ui.end_row();

                ui.label("Mipmaps");
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut overrides.mipmaps,
                        Some(Mipmaps::GeneratedAutomatic),
                        "Enabled",
                    );
                    ui.radio_value(&mut overrides.mipmaps, Some(Mipmaps::Disabled), "Disabled");
                    ui.radio_value(&mut overrides.mipmaps, None, "Custom...");
                });
                ui.end_row();

                ui.label("Compression");
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut overrides.compression_quality,
                        Some(Quality::Fast),
                        "Fast",
                    );
                    ui.radio_value(
                        &mut overrides.compression_quality,
                        Some(Quality::Normal),
                        "Normal",
                    );
                    ui.radio_value(
                        &mut overrides.compression_quality,
                        Some(Quality::Slow),
                        "Slow",
                    );
                    ui.radio_value(&mut overrides.compression_quality, None, "Custom...");
                });
                ui.end_row();
            }

            ui.end_row();
        });
}

fn optimize_nutexb_files_recursive(root: &Path) {
    // TODO: recursively search folders and call nutexb.optimize
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

fn convert_and_export_files(
    files: &[ImageFileSettings],
    output_folder: &Option<PathBuf>,
    overrides: &FileSettingsOverrides,
    save_to_original_folder: bool,
) -> Result<usize, Box<dyn Error>> {
    // TODO: Log an error if creating the output directory fails.
    if let Some(output_folder) = output_folder {
        std::fs::create_dir_all(output_folder)?;
    }

    let start = std::time::Instant::now();

    // TODO: report progress?
    let count = files
        .iter()
        .map(|file| {
            // TODO: find a simpler way to write this.
            if let Some(file_output_folder) = if save_to_original_folder {
                file.path.parent().map(PathBuf::from)
            } else {
                output_folder.to_owned()
            } {
                match convert_and_save_file(&file_output_folder, file, overrides) {
                    Ok(_) => 1,
                    Err(e) => {
                        // TODO: Log errors.
                        println!("Error converting {:?}: {e}", file.path);
                        0
                    }
                }
            } else {
                0
            }
        })
        .sum();

    println!("Converted {} files in {:?}", files.len(), start.elapsed());

    Ok(count)
}

fn convert_and_save_file(
    output_folder: &Path,
    file: &ImageFileSettings,
    overrides: &FileSettingsOverrides,
) -> Result<(), Box<dyn Error>> {
    // Global overrides take priority over file specific settings if enabled.
    let file_type = overrides.output_file_type.unwrap_or(file.output_file_type);
    let format = overrides.output_format.unwrap_or(file.output_format);
    let quality = overrides
        .compression_quality
        .unwrap_or(file.compression_quality);
    let mipmaps = overrides.mipmaps.unwrap_or(file.mipmaps);

    let output = output_folder
        .join(file.file_name_no_extension())
        .with_extension(file_type.extension());

    match file_type {
        ImageFileType::Dds => convert_to_dds(&file.image_file, &output, format, quality, mipmaps)?,
        ImageFileType::Png => convert_to_image(&file.image_file, &output)?,
        ImageFileType::Tiff => convert_to_image(&file.image_file, &output)?,
        ImageFileType::Nutexb => {
            convert_to_nutexb(&file.image_file, &output, format, quality, mipmaps)?
        }
        ImageFileType::Bntx => {
            convert_to_bntx(&file.image_file, &output, format, quality, mipmaps)?
        }
    }
    Ok(())
}

fn files_table(
    ui: &mut egui::Ui,
    files: &mut Vec<ImageFileSettings>,
    overrides: &FileSettingsOverrides,
) {
    let header_column = |header: &mut TableRow, name| {
        header.col(|ui| {
            ui.heading(name);
        })
    };

    let mut file_to_remove = None;

    TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto())
        .column(Column::auto())
        .column(Column::auto())
        .column(Column::auto())
        .column(Column::auto())
        .column(Column::auto())
        .column(Column::auto())
        .column(Column::auto())
        .column(Column::auto())
        .column(Column::auto())
        .min_scrolled_height(0.0)
        .header(20.0, |mut header| {
            header_column(&mut header, "Name");
            header_column(&mut header, "Type");
            header_column(&mut header, "Format");
            header_column(&mut header, "Dimensions");
            header_column(&mut header, "Output Name");
            header_column(&mut header, "Output Type");
            header_column(&mut header, "Output Format");
            header_column(&mut header, "Compression");
            header_column(&mut header, "Mipmaps");
        })
        .body(|body| {
            files_table_body(files, body, &mut file_to_remove, overrides);
        });

    if let Some(i) = file_to_remove {
        files.remove(i);
    }
}

fn files_table_body(
    files: &mut [ImageFileSettings],
    mut body: egui_extras::TableBody,
    file_to_remove: &mut Option<usize>,
    overrides: &FileSettingsOverrides,
) {
    for (i, file) in files.iter_mut().enumerate() {
        body.row(20.0, |mut row| {
            row.col(|ui| {
                ui.label(file.file_name_no_extension());
            });
            row.col(|ui| {
                ui.label(file.extension());
            });
            row.col(|ui| {
                ui.label(file.image_file.image_format().to_string());
            });

            let (width, height, depth) = file.image_file.dimensions();
            row.col(|ui| {
                ui.label(format!("{width}x{height}x{depth}"));
            });

            row.col(|ui| {
                // TODO: make this editable.
                ui.label(file.file_name_no_extension());
            });

            row.col(|ui| {
                if let Some(file_type) = overrides.output_file_type {
                    ui.label(file_type.to_string());
                } else {
                    edit_image_file_type(ui, i, &mut file.output_file_type);
                }
            });

            // The format can't be changed for uncompressed data.
            let output_file_type = overrides.output_file_type.unwrap_or(file.output_file_type);
            let supports_compression = file_supports_compression(output_file_type);

            row.col(|ui| {
                if !supports_compression {
                    // TODO: Allow floating point formats?
                    ui.label(ImageFormat::R8G8B8A8Unorm.to_string());
                } else if let Some(output_format) = overrides.output_format {
                    ui.label(output_format.to_string());
                } else {
                    edit_format(i, ui, &mut file.output_format);
                }
            });

            // These settings only make sense for files supporting compressed data.
            row.col(|ui| {
                ui.add_enabled_ui(supports_compression, |ui| {
                    if let Some(quality) = overrides.compression_quality {
                        ui.label(quality_text(quality));
                    } else {
                        edit_quality(ui, i, &mut file.compression_quality);
                    }
                });
            });

            row.col(|ui| {
                ui.add_enabled_ui(supports_compression, |ui| {
                    if let Some(mipmaps) = overrides.mipmaps {
                        ui.label(mipmaps_text(mipmaps));
                    } else {
                        edit_mipmaps(ui, i, &mut file.mipmaps);
                    }
                });
            });

            row.col(|ui| {
                if ui.button("Remove").clicked() {
                    *file_to_remove = Some(i);
                }
            });
        });
    }
}

fn file_supports_compression(file_type: ImageFileType) -> bool {
    matches!(
        file_type,
        ImageFileType::Dds | ImageFileType::Nutexb | ImageFileType::Bntx
    )
}

fn edit_image_file_type(ui: &mut egui::Ui, i: usize, file_type: &mut ImageFileType) {
    ComboBox::from_id_source(Id::new("type").with(i))
        .width(ui.available_width())
        .selected_text(file_type.to_string())
        .show_ui(ui, |ui| {
            for variant in ImageFileType::iter() {
                ui.selectable_value(file_type, variant, variant.to_string());
            }
        });
}

fn edit_format(i: usize, ui: &mut egui::Ui, format: &mut ImageFormat) {
    ComboBox::from_id_source(Id::new("format").with(i))
        .width(ui.available_width())
        .selected_text(format.to_string())
        .show_ui(ui, |ui| {
            for variant in ImageFormat::iter() {
                ui.selectable_value(format, variant, variant.to_string());
            }
        });
}

fn edit_quality(ui: &mut egui::Ui, i: usize, compression_quality: &mut Quality) {
    ComboBox::from_id_source(Id::new("quality").with(i))
        .width(ui.available_width())
        .selected_text(quality_text(*compression_quality))
        .show_ui(ui, |ui| {
            for variant in [Quality::Fast, Quality::Normal, Quality::Slow] {
                ui.selectable_value(compression_quality, variant, quality_text(variant));
            }
        });
}

fn edit_mipmaps(ui: &mut egui::Ui, i: usize, mipmaps: &mut Mipmaps) {
    ComboBox::from_id_source(Id::new("mipmaps").with(i))
        .width(ui.available_width())
        .selected_text(mipmaps_text(*mipmaps))
        .show_ui(ui, |ui| {
            // TODO: Also support generating a specific number of mipmaps.
            // TODO: Loading from surface won't work properly for uncompressed images.
            for variant in [
                Mipmaps::Disabled,
                Mipmaps::FromSurface,
                Mipmaps::GeneratedAutomatic,
            ] {
                ui.selectable_value(mipmaps, variant, mipmaps_text(variant));
            }
        });
}

fn quality_text(quality: Quality) -> &'static str {
    match quality {
        Quality::Fast => "Fast",
        Quality::Normal => "Normal",
        Quality::Slow => "Slow",
    }
}

fn mipmaps_text(mipmaps: Mipmaps) -> &'static str {
    match mipmaps {
        Mipmaps::Disabled => "Disabled",
        Mipmaps::FromSurface => "FromSurface",
        Mipmaps::GeneratedExact(_) => "GeneratedExact",
        Mipmaps::GeneratedAutomatic => "GeneratedAutomatic",
    }
}

fn horizontal_separator_empty(ui: &mut egui::Ui) {
    let available_space = ui.available_size_before_wrap();
    ui.allocate_space(egui::vec2(available_space.x, 6.0));
}

fn main() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1200.0, 800.0)),
        centered: true,
        drag_and_drop_support: true,
        ..Default::default()
    };

    eframe::run_native(
        concat!("Ultimate Tex ", env!("CARGO_PKG_VERSION")),
        options,
        Box::new(|creation_context| {
            // Use the dark theme with increased text contrast.
            let style = Style {
                visuals: Visuals {
                    widgets: widgets_dark(),
                    ..Default::default()
                },
                ..Default::default()
            };
            creation_context.egui_ctx.set_style(style);
            Box::<App>::default()
        }),
    )
    .unwrap();
}
