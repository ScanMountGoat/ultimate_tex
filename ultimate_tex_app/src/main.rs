// Hide the console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};

use eframe::egui::{self, Button, ComboBox, Id, ProgressBar, ScrollArea};
use egui::Grid;
use egui_extras::{Column, RetainedImage, TableBuilder, TableRow};
use image_dds::{ImageFormat, Mipmaps, Quality};
use rfd::FileDialog;
use strum::IntoEnumIterator;
use ultimate_tex::{convert_to_dds, convert_to_image, convert_to_nutexb, ImageFile, NutexbFile};

#[derive(Default)]
struct App {
    use_multicore: bool, // TODO: Default to true?
    is_exporting: bool,
    output_folder: Option<PathBuf>,
    files: Vec<ImageFileSettings>,
}

// TODO: Move this to the library?
#[derive(PartialEq, Clone, Copy, strum::Display, strum::EnumIter)]
enum ImageFileType {
    Dds,
    Png,
    Tiff,
    Nutexb,
    Bntx,
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

// TODO: Store an ImageFile instead?
struct ImageFileSettings {
    path: PathBuf,
    image_file: ImageFile,
    output_file_type: ImageFileType, // TODO: Should this be a string for the extension?
    output_format: ImageFormat,
    compression_quality: Quality,
    mipmaps: image_dds::Mipmaps,
}

impl ImageFileSettings {
    fn from_path(path: PathBuf) -> Self {
        let image_file = ImageFile::read(&path).unwrap();
        ImageFileSettings {
            path,
            image_file,
            output_file_type: ImageFileType::Nutexb,
            output_format: ImageFormat::BC7Unorm,
            compression_quality: Quality::Fast,
            mipmaps: Mipmaps::GeneratedAutomatic,
        }
    }

    fn file_name_no_extension(&self) -> String {
        // TODO: Avoid unwrap.
        self.path
            .with_extension("")
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string()
    }

    fn extension(&self) -> &str {
        // TODO: Avoid unwrap.
        self.path.extension().unwrap().to_str().unwrap()
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Allow loading files by dragging and dropping onto the window.
        for file in &ctx.input().raw.dropped_files {
            if let Some(path) = &file.path {
                let new_file = ImageFileSettings::from_path(path.clone());
                self.files.push(new_file);
            }
        }

        egui::TopBottomPanel::top("menu_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Add File...").clicked() {
                        ui.close_menu();

                        if let Some(files) = FileDialog::new()
                            .add_filter(
                                "image files",
                                &["png", "tiff", "nutexb", "bntx", "jpeg", "jpg"],
                            )
                            .pick_files()
                        {
                            for file in files {
                                let new_file = ImageFileSettings::from_path(file);
                                self.files.push(new_file);
                            }
                        }
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

                        if let Some(folder) = FileDialog::new().pick_folder() {
                            optimize_nutexb_files_recursive(&folder);
                            // TODO: Show how many files were optimized in the bottom bar?
                            // TODO: Log errors to the bottom bar?
                        }
                    }
                });

                ui.menu_button("Help", |ui| {
                    if ui.button("Wiki").clicked() {
                        ui.close_menu();
                        // TODO: Open a wiki page?
                    }
                });
            })
        });

        egui::TopBottomPanel::top("output_panel").show(ctx, |ui| {
            ui.heading("Output");

            ui.checkbox(&mut self.use_multicore, "Multicore Processing");
            ui.horizontal(|ui| {
                ui.label("Output Folder");
                if let Some(output_folder) = &self.output_folder {
                    ui.label(output_folder.to_string_lossy());
                }
                if ui.button("Open...").clicked() {
                    if let Some(folder) = FileDialog::new().pick_folder() {
                        self.output_folder = Some(folder);
                    }
                }
            });

            // TODO: Add presets for common settings like UI, color, non color, etc?

            // Disable the button while exporting to prevent multiple batch jobs.
            // Exporting should only be enabled once an export folder is selected.
            // TODO: Use Option<PathBuf> instead?
            // TODO: Show on hover why the button is disabled.
            let can_export = !self.is_exporting && self.output_folder.is_some();
            if ui
                .add_enabled_ui(can_export, |ui| {
                    // Make the button larger and easier to click.
                    ui.add_sized(egui::vec2(80.0, 30.0), Button::new("Export"))
                })
                .inner
                .clicked()
            {
                // TODO: Spawn a thread to process the files.
                // TODO: Update progress using a callback?
                if let Some(output_folder) = &self.output_folder {
                    self.is_exporting = true;
                    convert_and_export_files(&self.files, output_folder, self.use_multicore);
                    self.is_exporting = false;
                }
            }
            horizontal_separator_empty(ui);
        });

        egui::TopBottomPanel::bottom("progress_panel").show(ctx, |ui| {
            // Only allow a single export operation in progress at a time.
            if self.is_exporting {
                // TODO: Track progress for exporting.
                ui.horizontal(|ui| {
                    ui.label("Processing 4 of 20 files...");
                    ui.add(ProgressBar::new(4.0 / 20.0));
                });
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Files");
            horizontal_separator_empty(ui);

            ScrollArea::horizontal()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    if !self.files.is_empty() {
                        files_table(ui, &mut self.files);
                    } else {
                        ui.label("Drag and drop image files onto the window or add files using File > Add File...");
                    }
                });
        });
    }
}

fn optimize_nutexb_files_recursive(root: &Path) {
    // TODO: recursively search folders and call nutexb.optimize
    for entry in globwalk::GlobWalkerBuilder::from_patterns(root, &["*.{nutexb}"])
        .build()
        .unwrap()
        .into_iter()
        .filter_map(Result::ok)
    {
        if let Ok(mut nutexb) = NutexbFile::read_from_file(entry.path()) {
            nutexb.optimize_size();
            // TODO: Avoid unwrap.
            nutexb.write_to_file(entry.path()).unwrap();
        }
    }
}

fn convert_and_export_files(
    files: &[ImageFileSettings],
    output_folder: &Path,
    use_multicore: bool,
) {
    // TODO: Avoid exporting if this fails?
    std::fs::create_dir_all(output_folder).unwrap();
    // TODO: report progress?
    // TODO: Use rayon if use_multicore is enabled.
    for file in files {
        let output = output_folder
            .join(file.file_name_no_extension())
            .with_extension(file.output_file_type.extension());

        match file.output_file_type {
            ImageFileType::Dds => convert_to_dds(&file.image_file, &output, file.output_format),
            ImageFileType::Png => convert_to_image(&file.image_file, &output),
            ImageFileType::Tiff => convert_to_image(&file.image_file, &output),
            ImageFileType::Nutexb => {
                convert_to_nutexb(&file.image_file, &output, file.output_format)
            }
            ImageFileType::Bntx => todo!(),
        }
    }
}

fn files_table(ui: &mut egui::Ui, files: &mut [ImageFileSettings]) {
    let header_column = |header: &mut TableRow, name| {
        header.col(|ui| {
            ui.heading(name);
        })
    };

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
        .body(|mut body| {
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
                        edit_image_file_type(ui, i, &mut file.output_file_type);
                    });

                    let supports_compression = matches!(
                        file.output_file_type,
                        ImageFileType::Dds | ImageFileType::Nutexb | ImageFileType::Bntx
                    );

                    // The format can't be changed for uncompressed data.
                    // TODO: Allow bgra or floating point formats?
                    row.col(|ui| {
                        if supports_compression {
                            edit_format(i, ui, &mut file.output_format);
                        } else {
                            ui.label(ImageFormat::R8G8B8A8Unorm.to_string());
                        }
                    });

                    // These settings only make sense for files supporting compressed data.
                    row.col(|ui| {
                        ui.add_enabled_ui(supports_compression, |ui| {
                            edit_quality(ui, i, &mut file.compression_quality);
                        });
                    });

                    row.col(|ui| {
                        ui.add_enabled_ui(supports_compression, |ui| {
                            edit_mipmaps(ui, i, &mut file.mipmaps);
                        });
                    });
                });
            }
        });
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

    // TODO: Modify the themes to be slightly higher contrast.
    eframe::run_native(
        "Ultimate Tex",
        options,
        Box::new(|_cc| Box::new(App::default())),
    );
}
