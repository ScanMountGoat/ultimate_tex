use std::{path::PathBuf, sync::Arc};

use gpui_kit::{
    App, Bounds, Entity, ObjectFit, Pixels, RenderImage, SharedString, Subscription, Window,
    base::{IndexPath, h_flex, v_flex},
    component::{
        button::{Button, ButtonVariants},
        select::{Select, SelectEvent, SelectItem, SelectState},
        table::{Column, TableDelegate, TableState},
        tooltip::Tooltip,
    },
    div, img,
    prelude::*,
    px,
};
use strum::IntoEnumIterator;

use crate::{
    Application,
    app::{ImageFileSettings, ImageFileType, load_files},
    overrides::FileSettingsOverrides,
};

#[derive(Clone)]
struct EnumSelectItem<T>(T);

impl<T> SelectItem for EnumSelectItem<T>
where
    T: PartialEq + Clone + std::fmt::Display,
{
    type Value = T;

    fn title(&self) -> SharedString {
        self.0.to_string().into()
    }

    fn value(&self) -> &Self::Value {
        &self.0
    }
}

pub struct File {
    pub settings: ImageFileSettings,
    thumbnail: Arc<RenderImage>,
    output_type_select: Entity<SelectState<Vec<EnumSelectItem<ImageFileType>>>>,
    output_format_select: Entity<SelectState<Vec<EnumSelectItem<image_dds::ImageFormat>>>>,
    compression_select: Entity<SelectState<Vec<EnumSelectItem<image_dds::Quality>>>>,
    mipmaps_select: Entity<SelectState<Vec<EnumSelectItem<image_dds::Mipmaps>>>>,
    _subscriptions: Vec<Subscription>,
}

pub struct FilesTableDelegate {
    pub files: Vec<File>,
    columns: Vec<Column>,
    overrides: Entity<FileSettingsOverrides>,
}

impl FilesTableDelegate {
    pub fn new(overrides: Entity<FileSettingsOverrides>) -> Self {
        Self {
            files: Vec::new(),
            columns: vec![
                Column::new("image", "Image"),
                Column::new("output-type", "Type").width(px(120.0)),
                Column::new("output-format", "Format").width(px(200.0)),
                Column::new("compression", "Compression").width(px(120.0)),
                Column::new("mipmaps", "Mipmaps").width(px(150.0)),
                Column::new("remove", ""),
            ],
            overrides,
        }
    }

    pub fn add_files(
        &mut self,
        paths: &[PathBuf],
        window: &mut Window,
        cx: &mut Context<TableState<FilesTableDelegate>>,
    ) {
        let (thumbnails, settings) = load_files(paths);

        for (settings, thumbnail) in settings.into_iter().zip(thumbnails) {
            // Don't add duplicate files.
            if self.files.iter().any(|f| f.settings.name == settings.name) {
                continue;
            }
            let new_index = self.files.len();

            let output_type_select = cx.new(|cx| {
                let variants = vec![
                    EnumSelectItem(ImageFileType::Png),
                    EnumSelectItem(ImageFileType::Dds),
                    EnumSelectItem(ImageFileType::Nutexb),
                    EnumSelectItem(ImageFileType::Bntx),
                ];
                SelectState::new(variants, Some(IndexPath::new(0)), window, cx)
            });
            let output_type_select_sub = cx.subscribe_in(
                &output_type_select,
                window,
                move |this, _select, event, _window, _cx| {
                    if let SelectEvent::Confirm(Some(value)) = event {
                        this.delegate_mut().files[new_index]
                            .settings
                            .output_file_type = *value;
                    }
                },
            );

            let output_format_select =
                cx.new(|cx| enum_select_state(settings.output_format, window, cx));
            let format_select_sub = cx.subscribe_in(
                &output_format_select,
                window,
                move |this, _select, event, _window, _cx| {
                    if let SelectEvent::Confirm(Some(value)) = event {
                        this.delegate_mut().files[new_index].settings.output_format = *value;
                    }
                },
            );

            let compression_select =
                cx.new(|cx| enum_select_state(settings.output_quality, window, cx));
            let compression_select_sub = cx.subscribe_in(
                &compression_select,
                window,
                move |this, _select, event, _window, _cx| {
                    if let SelectEvent::Confirm(Some(value)) = event {
                        this.delegate_mut().files[new_index].settings.output_quality = *value;
                    }
                },
            );

            // TODO: generated exact doesn't make sense if there's no option to set the number.
            let mipmaps_select =
                cx.new(|cx| enum_select_state(settings.output_mipmaps, window, cx));
            let mipmaps_select_sub = cx.subscribe_in(
                &mipmaps_select,
                window,
                move |this, _select, event, _window, _cx| {
                    if let SelectEvent::Confirm(Some(value)) = event {
                        this.delegate_mut().files[new_index].settings.output_mipmaps = *value;
                    }
                },
            );

            self.files.push(File {
                settings,
                thumbnail,
                output_type_select,
                output_format_select,
                compression_select,
                mipmaps_select,
                _subscriptions: vec![
                    output_type_select_sub,
                    format_select_sub,
                    compression_select_sub,
                    mipmaps_select_sub,
                ],
            });
        }
    }
}

fn enum_select_state<T>(
    initial_value: T,
    window: &mut Window,
    cx: &mut Context<'_, SelectState<Vec<EnumSelectItem<T>>>>,
) -> SelectState<Vec<EnumSelectItem<T>>>
where
    T: PartialEq + Clone + IntoEnumIterator + std::fmt::Display,
{
    let variants = T::iter().map(EnumSelectItem).collect();
    let index = T::iter()
        .position(|f| f == initial_value)
        .unwrap_or_default();
    SelectState::new(variants, Some(IndexPath::new(index)), window, cx)
}

impl TableDelegate for FilesTableDelegate {
    fn columns_count(&self, _: &App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _: &App) -> usize {
        self.files.len()
    }

    fn column(&self, col_ix: usize, _: &App) -> Column {
        self.columns[col_ix].clone()
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let row = &self.files[row_ix];
        let col = &self.columns[col_ix];

        // See if the final file type is compressed to disable irrelevant settings.
        let is_compressed = !matches!(
            self.overrides
                .read(cx)
                .output_file_type
                .unwrap_or(row.settings.output_file_type),
            ImageFileType::Png | ImageFileType::Tiff
        );

        let thumbnail = row.thumbnail.clone();
        let full_path = row.settings.path.to_string_lossy().to_string();
        let size = format!(
            "{} x {} x {}",
            row.settings.dimensions.0, row.settings.dimensions.1, row.settings.dimensions.2
        );
        let img_format = row.settings.format.to_string();

        let child =
            match col.key.as_ref() {
                "image" => h_flex()
                    .size_full()
                    .gap(px(10.0))
                    .child(
                        div()
                            .h_full()
                            .aspect_square()
                            .overflow_hidden()
                            .child(
                                img(thumbnail.clone())
                                    .size_full()
                                    .object_fit(ObjectFit::Fill), // TODO: fix for cube or non square
                            )
                            .id(format!("thumbnail{row_ix}"))
                            .tooltip(move |window, cx| {
                                let thumbnail = thumbnail.clone();
                                let img_format = img_format.clone();
                                let size = size.clone();
                                Tooltip::element(move |_, _| {
                                    v_flex()
                                        .gap(px(10.0))
                                        .child(img_format.clone())
                                        .child(size.clone())
                                        .child(img(thumbnail.clone()).h(px(256.0)))
                                })
                                .build(window, cx)
                            }),
                    )
                    .child(
                        div()
                            .id(format!("name{row_ix}"))
                            .tooltip(move |window, cx| {
                                Tooltip::new(full_path.clone()).build(window, cx)
                            })
                            .child(row.settings.name.clone())
                            .truncate(),
                    )
                    .into_any_element(),
                // TODO: replace these selects with the override value text if the overrides are not none
                "output-type" => div()
                    .w_full()
                    .child(
                        Select::new(&row.output_type_select)
                            .disabled(self.overrides.read(cx).output_file_type.is_some()),
                    )
                    .into_any_element(),
                "output-format" => div()
                    .w_full()
                    .child(Select::new(&row.output_format_select).disabled(
                        self.overrides.read(cx).output_format.is_some() || !is_compressed,
                    ))
                    .into_any_element(),
                "compression" => div()
                    .w_full()
                    .child(Select::new(&row.compression_select).disabled(
                        self.overrides.read(cx).output_quality.is_some() || !is_compressed,
                    ))
                    .into_any_element(),
                "mipmaps" => div()
                    .w_full()
                    .child(
                        Select::new(&row.mipmaps_select)
                            .disabled(self.overrides.read(cx).mipmaps.is_some() || !is_compressed),
                    )
                    .into_any_element(),
                "remove" => div()
                    .w_full()
                    .child(
                        Button::new(format!("remove{row_ix}"))
                            .label("Remove")
                            .secondary()
                            .on_click(cx.listener(move |this, _, _window, cx| {
                                this.delegate_mut().files.remove(row_ix);
                                cx.notify();
                            })),
                    )
                    .into_any_element(),
                _ => div().into_any_element(),
            };
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_start()
            .child(child)
    }

    fn render_empty(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        div()
            .flex()
            .size_full()
            .items_center()
            .justify_center()
            .child(
                "Drag and drop image files onto the window or add files using File > Add Files...",
            )
    }
}

pub fn update_table_column_sizes(
    this: &mut Application,
    child_bounds: Vec<Bounds<Pixels>>,
    cx: &mut Context<Application>,
) {
    // Compute the size of the table before it is rendered.
    let table_width = child_bounds[0].size.width;
    let delegate = this.table_state.read(cx).delegate();

    // Filter out the name column.
    let name_column_index = 0;
    let column_widths: Pixels = delegate
        .columns
        .iter()
        .enumerate()
        .filter_map(|(i, col)| {
            if i != name_column_index {
                Some(col.width)
            } else {
                None
            }
        })
        .sum();
    let remaining = table_width - column_widths;

    // Datatable doesn't support responsive layouts like table.
    // Make the name column fill the remaining space.
    this.table_state.update(cx, |state, cx| {
        state.delegate_mut().columns[name_column_index].width = remaining.max(px(100.0));
        state.refresh(cx);
    })
}
