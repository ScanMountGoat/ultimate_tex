use gpui_kit::{
    SharedString,
    base::h_flex,
    component::{form::field, radio::RadioGroup},
    prelude::*,
};
use image_dds::{ImageFormat, Mipmaps, Quality};

use crate::app::ImageFileType;

#[derive(Debug, Clone)]
pub struct FileSettingsOverrides {
    pub output_file_type: Option<ImageFileType>,
    pub output_format: Option<ImageFormat>,
    pub mipmaps: Option<Mipmaps>,
    pub output_quality: Option<Quality>,
}

impl FileSettingsOverrides {
    pub const OUTPUT_TYPES: [Option<ImageFileType>; 5] = [
        Some(ImageFileType::Png),
        Some(ImageFileType::Dds),
        Some(ImageFileType::Nutexb),
        Some(ImageFileType::Bntx),
        None,
    ];

    pub const OUTPUT_FORMATS: [Option<ImageFormat>; 3] = [
        Some(ImageFormat::BC7RgbaUnormSrgb),
        Some(ImageFormat::BC7RgbaUnorm),
        None,
    ];

    pub const OUTPUT_FORMAT_LABELS: [SharedString; 3] = [
        SharedString::new_static("Color (sRGB) + Alpha"),
        SharedString::new_static("Color (Linear) + Alpha"),
        SharedString::new_static("Custom..."),
    ];

    pub const MIPMAPS: [Option<Mipmaps>; 3] = [
        Some(Mipmaps::GeneratedAutomatic),
        Some(Mipmaps::Disabled),
        None,
    ];

    pub const MIPMAP_LABELS: [SharedString; 3] = [
        SharedString::new_static("Enabled"),
        SharedString::new_static("Disabled"),
        SharedString::new_static("Custom..."),
    ];

    pub const QUALITIES: [Option<Quality>; 4] = [
        Some(Quality::Fast),
        Some(Quality::Normal),
        Some(Quality::Slow),
        None,
    ];
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

impl Render for FileSettingsOverrides {
    fn render(
        &mut self,
        _window: &mut gpui_kit::Window,
        cx: &mut gpui_kit::prelude::Context<Self>,
    ) -> impl gpui_kit::prelude::IntoElement {
        // Show options that override individual file settings.
        // Setting a non compressed output type like PNG disables non relevant settings.
        h_flex()
            .items_start()
            .child(
                // TODO: div instead of field?
                field().label("Output Type").child(
                    RadioGroup::vertical("output-type")
                        .children(
                            FileSettingsOverrides::OUTPUT_TYPES.iter().map(|v| {
                                v.map(|v| v.to_string()).unwrap_or("Custom...".to_string())
                            }),
                        )
                        .selected_index(
                            FileSettingsOverrides::OUTPUT_TYPES
                                .iter()
                                .position(|v| v == &self.output_file_type),
                        )
                        .on_change(cx.listener(|this, index, _window, _cx| {
                            if let Some(new_value) = FileSettingsOverrides::OUTPUT_TYPES.get(*index)
                            {
                                this.output_file_type = *new_value;
                            }
                        })),
                ),
            )
            .when(self.output_file_type != Some(ImageFileType::Png), |this| {
                this.child(
                    field().label("Output Format").child(
                        RadioGroup::vertical("output-format")
                            .children(FileSettingsOverrides::OUTPUT_FORMAT_LABELS)
                            .selected_index(
                                FileSettingsOverrides::OUTPUT_FORMATS
                                    .iter()
                                    .position(|v| v == &self.output_format),
                            )
                            .on_change(cx.listener(|this, index, _window, _cx| {
                                if let Some(new_value) =
                                    FileSettingsOverrides::OUTPUT_FORMATS.get(*index)
                                {
                                    this.output_format = *new_value;
                                }
                            })),
                    ),
                )
                .child(
                    field().label("Compression").child(
                        RadioGroup::vertical("compression")
                            .children(FileSettingsOverrides::QUALITIES.iter().map(|v| {
                                v.map(|v| v.to_string()).unwrap_or("Custom...".to_string())
                            }))
                            .selected_index(
                                FileSettingsOverrides::QUALITIES
                                    .iter()
                                    .position(|v| v == &self.output_quality),
                            )
                            .on_change(cx.listener(|this, index, _window, _cx| {
                                if let Some(new_value) =
                                    FileSettingsOverrides::QUALITIES.get(*index)
                                {
                                    this.output_quality = *new_value;
                                }
                            })),
                    ),
                )
                .child(
                    field().label("Mipmaps").child(
                        RadioGroup::vertical("mipmaps")
                            .children(FileSettingsOverrides::MIPMAP_LABELS)
                            .selected_index(
                                FileSettingsOverrides::MIPMAPS
                                    .iter()
                                    .position(|v| v == &self.mipmaps),
                            )
                            .on_change(cx.listener(|this, index, _window, _cx| {
                                if let Some(new_value) = FileSettingsOverrides::MIPMAPS.get(*index)
                                {
                                    this.mipmaps = *new_value;
                                }
                            })),
                    ),
                )
            })
    }
}
