use std::{
    error::Error,
    path::{Path, PathBuf},
    sync::Arc,
};

use gpui_kit::{RenderImage, SharedString};
use image_dds::{ImageFormat, Mipmaps, Quality, image::Frame};
use rayon::prelude::*;
use ultimate_tex_lib::{ImageFile, NutexbFile};

use crate::{File, overrides::FileSettingsOverrides};

// TODO: Add proper logging?

#[derive(Clone)]
pub struct ImageFileSettings {
    pub name: SharedString,
    pub path: PathBuf,
    pub format: ImageFormat,
    pub dimensions: (u32, u32, u32),
    pub output_file_type: ImageFileType,
    pub output_format: ImageFormat,
    pub output_quality: Quality,
    pub output_mipmaps: Mipmaps,
}

pub fn convert_and_export_files(
    files: &[File],
    output_folder: Option<&Path>,
    save_in_same_folder: bool,
    overrides: &FileSettingsOverrides,
) -> Result<Vec<String>, Box<dyn Error>> {
    // TODO: Log an error if creating the output directory fails.
    if let Some(output_folder) = output_folder {
        std::fs::create_dir_all(output_folder)?;
    }

    // TODO: report progress?
    let settings_to_export: Vec<_> = files.iter().map(|f| &f.settings).collect();
    dbg!(overrides);
    let mut messages: Vec<_> = settings_to_export
        .par_iter()
        .filter_map(|f| {
            let output = if save_in_same_folder {
                f.path.parent()
            } else {
                output_folder
            };

            output.and_then(|output| {
                // Collect error messages to display to the user.
                let file = ImageFile::from_file(&f.path).ok()?;
                match convert_and_save_file(output, &f, &file, overrides) {
                    Ok(_) => None,
                    Err(e) => Some(format!("Error converting {}: {e}", f.name)),
                }
            })
        })
        .collect();

    // Always show basic results for the first message.
    messages.insert(
        0,
        format!(
            "Successfully converted {} of {} file(s)",
            files.len() - messages.len(),
            files.len(),
        ),
    );

    Ok(messages)
}

pub fn load_files(files: &[PathBuf]) -> (Vec<Arc<RenderImage>>, Vec<ImageFileSettings>) {
    let start = std::time::Instant::now();

    // Only the expensive file reading benefits from parallelism.
    // TODO: Store or log errors?
    let new_files: Vec<_> = files
        .par_iter()
        .filter_map(|file| Some((file, ImageFile::from_file(file).ok()?)))
        .collect();
    // TODO: rework thumbnails to use images supported by gpui
    let new_thumbnails: Vec<_> = new_files
        .par_iter()
        .map(|(_, image)| create_render_image(image))
        .collect();

    let new_settings = new_files
        .into_iter()
        .map(|(file, image)| ImageFileSettings::from_image(file.clone(), &image))
        .collect();

    println!("Loaded {} files in {:?}", files.len(), start.elapsed());
    (new_thumbnails, new_settings)
}

fn create_render_image(f: &ImageFile) -> Arc<RenderImage> {
    // Convert to a gpui compatible format.
    let mut image = f.to_image().unwrap();
    image.pixels_mut().for_each(|p| {
        // gpui expects BGRA instead of RGBA.
        p.0.swap(0, 2);
        // Disable alpha for better display of PRM and NOR.
        p[3] = 255u8;
    });

    let frame = Frame::new(image);
    Arc::new(RenderImage::new([frame]))
}

#[derive(
    Debug, PartialEq, Clone, Copy, strum::Display, strum::EnumIter, strum::EnumString, Default,
)]
pub enum ImageFileType {
    Dds,
    #[default]
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

impl ImageFileSettings {
    fn from_image(path: PathBuf, image: &ImageFile) -> Self {
        // Default to the input format to encourage lossless conversions.
        let format = image.image_format();
        ImageFileSettings {
            name: path.file_name().unwrap().to_string_lossy().into(),
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

pub fn optimize_nutexb_files(folder: &Path) {
    globwalk::GlobWalkerBuilder::from_patterns(folder, &["*.{nutexb}"])
        .build()
        .unwrap()
        .filter_map(Result::ok)
        .par_bridge()
        .for_each(|entry| {
            if let Ok(mut nutexb) = NutexbFile::read_from_file(entry.path()) {
                nutexb.optimize_size();
                // TODO: log errors
                let _ = nutexb.write_to_file(entry.path());
                // TODO: return how many files were processed
            }
        });
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
        ImageFileType::Dds => image_file.save_dds(&output, format, quality, mipmaps)?,
        ImageFileType::Png => image_file.save_image(&output)?,
        ImageFileType::Tiff => image_file.save_image(&output)?,
        ImageFileType::Nutexb => image_file.save_nutexb(&output, format, quality, mipmaps)?,
        ImageFileType::Bntx => image_file.save_bntx(&output, format, quality, mipmaps)?,
    }
    Ok(())
}
