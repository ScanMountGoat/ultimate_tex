use std::{
    error::Error,
    path::{Path, PathBuf},
};

use image_dds::{ImageFormat, Mipmaps, Quality};
use rayon::prelude::*;
use rfd::FileDialog;
use ultimate_tex::{
    convert_to_bntx, convert_to_dds, convert_to_image, convert_to_nutexb, ImageFile, NutexbFile,
};

// TODO: Add proper logging using events?
#[derive(Default)]
pub struct App {
    // Store settings separately to pass to and from javascript.
    // Image data should only ever be accessible from Rust.
    pub settings: AppSettings,
    pub files: Vec<ImageFile>,
}

#[derive(Clone, Default)]
pub struct AppSettings {
    pub output_folder: Option<PathBuf>,
    pub save_in_same_folder: bool,
    pub overrides: FileSettingsOverrides,
    pub file_settings: Vec<ImageFileSettings>,
}

#[derive(Clone)]
pub struct FileSettingsOverrides {
    pub output_file_type: Option<ImageFileType>,
    pub output_format: Option<ImageFormat>,
    pub mipmaps: Option<Mipmaps>,
    pub output_quality: Option<Quality>,
}

#[derive(Clone)]
pub struct ImageFileSettings {
    pub name: String,
    pub path: PathBuf,
    pub format: ImageFormat,
    pub dimensions: (u32, u32, u32),
    pub output_file_type: ImageFileType,
    pub output_format: ImageFormat,
    pub output_quality: Quality,
    pub output_mipmaps: Mipmaps,
}

impl App {
    pub fn add_files(&mut self) {
        // TODO: make this async to not block?
        if let Some(files) = FileDialog::new()
            .add_filter(
                "image files",
                &["png", "tiff", "nutexb", "bntx", "jpeg", "jpg", "dds"],
            )
            .pick_files()
        {
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
    }

    pub fn remove_file(&mut self, index: usize) {
        self.files.remove(index);
        self.settings.file_settings.remove(index);
    }

    pub fn clear_files(&mut self) {
        self.files.clear();
        self.settings.file_settings.clear();
    }

    pub fn convert_and_export_files(&self) -> Result<Vec<String>, Box<dyn Error>> {
        // TODO: Log an error if creating the output directory fails.
        if let Some(output_folder) = &self.settings.output_folder {
            std::fs::create_dir_all(output_folder)?;
        }

        // TODO: report progress?
        let mut messages: Vec<_> = self
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
                    // Collect error messages to display to the user.
                    match convert_and_save_file(output, settings, file, &self.settings.overrides) {
                        Ok(_) => None,
                        Err(e) => Some(format!("Error converting {}: {e}", settings.name)),
                    }
                })
            })
            .collect();

        // Always show basic results for the first message.
        messages.insert(
            0,
            format!(
                "Successfully converted {} of {} file(s)",
                self.files.len() - messages.len(),
                self.files.len(),
            ),
        );

        Ok(messages)
    }
}

impl Default for FileSettingsOverrides {
    fn default() -> Self {
        // Default to a custom output format to encourage lossless conversions.
        Self {
            output_file_type: Some(ImageFileType::Png),
            output_format: None,
            mipmaps: Some(Mipmaps::GeneratedAutomatic),
            output_quality: Some(Quality::Fast),
        }
    }
}

#[derive(PartialEq, Clone, Copy, strum::Display, strum::EnumIter, strum::EnumString)]
pub enum ImageFileType {
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

pub fn optimize_nutexb_files() {
    if let Some(folder) = FileDialog::new()
        .set_title("Select Nutexb Root Folder")
        .pick_folder()
    {
        for entry in globwalk::GlobWalkerBuilder::from_patterns(folder, &["*.{nutexb}"])
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
    let quality = overrides.output_quality.unwrap_or(file.output_quality);
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
