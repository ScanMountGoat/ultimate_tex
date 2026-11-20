use std::{path::PathBuf, sync::Arc};

use gpui_kit::{
    base::{Disableable, IndexPath, h_flex, v_flex},
    component::{
        IconName, Sizable, Theme, ThemeRegistry,
        button::{Button, ButtonVariants},
        checkbox::Checkbox,
        menu::DropdownMenu,
        select::{Select, SelectEvent, SelectItem, SelectState},
        table::{Column, DataTable, TableDelegate, TableState},
        tooltip::Tooltip,
    },
    prelude::FluentBuilder,
    *,
};
use rfd::AsyncFileDialog;
use strum::IntoEnumIterator;

use crate::{
    app::{
        ImageFileSettings, ImageFileType, convert_and_export_files, load_files,
        optimize_nutexb_files,
    },
    overrides::FileSettingsOverrides,
};

mod app;
mod overrides;

actions!(
    ultimate_tex,
    [AddFiles, ClearFiles, OptimizeNutexbPadding, OpenWiki, Quit]
);

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

struct File {
    settings: ImageFileSettings,
    thumbnail: Arc<RenderImage>,
    output_type_select: Entity<SelectState<Vec<EnumSelectItem<ImageFileType>>>>,
    output_format_select: Entity<SelectState<Vec<EnumSelectItem<image_dds::ImageFormat>>>>,
    compression_select: Entity<SelectState<Vec<EnumSelectItem<image_dds::Quality>>>>,
    mipmaps_select: Entity<SelectState<Vec<EnumSelectItem<image_dds::Mipmaps>>>>,
    _subscriptions: Vec<Subscription>,
}

struct FilesTableDelegate {
    files: Vec<File>,
    columns: Vec<Column>,
    overrides: Entity<FileSettingsOverrides>,
}

impl FilesTableDelegate {
    fn new(overrides: Entity<FileSettingsOverrides>) -> Self {
        Self {
            files: Vec::new(),
            columns: vec![
                Column::new("image", "Image"),
                Column::new("format", "Format").width(px(200.0)),
                Column::new("size", "Size").width(px(150.0)),
                Column::new("output-type", "Output Type").width(px(150.0)),
                Column::new("output-format", "Output Format").width(px(200.0)),
                Column::new("compression", "Compression").width(px(150.0)),
                Column::new("mipmaps", "Mipmaps").width(px(210.0)),
                Column::new("remove", ""),
            ],
            overrides,
        }
    }

    fn add_files(
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
                                Tooltip::element(move |_, _| img(thumbnail.clone()).h(px(256.0)))
                                    .build(window, cx)
                            }),
                    )
                    .child(
                        div()
                            .id(format!("name{row_ix}"))
                            .tooltip(move |window, cx| {
                                Tooltip::new(full_path.clone()).build(window, cx)
                            })
                            .child(row.settings.name.clone()),
                    )
                    .truncate()
                    .into_any_element(),
                "format" => div()
                    .child(row.settings.format.to_string())
                    .into_any_element(),
                "size" => div()
                    .child(format!(
                        "{}x{}x{}",
                        row.settings.dimensions.0,
                        row.settings.dimensions.1,
                        row.settings.dimensions.2
                    ))
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

struct Application {
    export_folder: Entity<String>,
    save_to_original_folder: bool,
    table_state: Entity<TableState<FilesTableDelegate>>,
    overrides: Entity<FileSettingsOverrides>,
}

impl Application {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let export_folder = cx.new(|_cx| String::new());

        let overrides = cx.new(|_cx| FileSettingsOverrides::default());

        let delegate = FilesTableDelegate::new(overrides.clone());
        let table_state = cx.new(|cx| {
            // Changing the overrides should update the table rendering.
            cx.observe(&overrides, |_this, _target, cx| cx.notify())
                .detach();

            TableState::new(delegate, window, cx)
                .col_selectable(false)
                .col_movable(false)
                .col_resizable(false)
        });

        Self {
            export_folder,
            table_state,
            save_to_original_folder: false,
            overrides,
        }
    }

    fn on_action_add_files(&mut self, _: &AddFiles, window: &mut Window, cx: &mut Context<Self>) {
        cx.spawn_in(window, async move |this, cx| {
            // Await the native OS file picker without blocking the UI thread
            let file_handle = AsyncFileDialog::new()
                .set_title("Open File")
                .add_filter(
                    "image files",
                    &["png", "tiff", "nutexb", "bntx", "jpeg", "jpg", "dds"],
                )
                .pick_files()
                .await;

            if let Some(handles) = file_handle {
                let paths: Vec<_> = handles.iter().map(|h| h.path().to_owned()).collect();
                // Update view state back on the UI context and notify for re-render
                this.update_in(cx, |view, window, cx| {
                    view.table_state.update(cx, |state, cx| {
                        state.delegate_mut().add_files(&paths, window, cx);
                    });
                    cx.notify();
                })
                .ok();
            }
        })
        .detach();
    }

    fn on_action_clear_files(
        &mut self,
        _: &ClearFiles,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.table_state.update(cx, |state, cx| {
            state.delegate_mut().files.clear();
            state.clear_selection(cx);
            state.refresh(cx);
        });
        cx.notify();
    }

    fn on_action_optimize_nutexb_padding(
        &mut self,
        _: &OptimizeNutexbPadding,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.spawn(async |_, _| {
            // Await the native OS file picker without blocking the UI thread
            let file_handle = AsyncFileDialog::new()
                .set_title("Select Nutexb Root Folder")
                .pick_folder()
                .await;

            if let Some(handle) = file_handle {
                let path = handle.path();
                optimize_nutexb_files(path);
            }
        })
        .detach();
    }

    fn on_action_open_wiki(&mut self, _: &OpenWiki, _window: &mut Window, cx: &mut Context<Self>) {
        cx.open_url("https://github.com/ScanMountGoat/ultimate_tex/wiki");
    }
}

// TODO: match the layout from ultimate_tex
impl Render for Application {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // TODO: padding and spacing
        v_flex()
            .size_full()
            .gap(px(10.0))
            .p(px(10.0))
            .on_drop(cx.listener(|this, paths: &ExternalPaths, window, cx| {
                let files = paths.paths();
                // TODO: add files asynchronously
                this.table_state.update(cx, |state, cx| {
                    state.delegate_mut().add_files(files, window, cx);
                });
                cx.notify();
            }))
            .on_action(cx.listener(Self::on_action_add_files))
            .on_action(cx.listener(Self::on_action_clear_files))
            .on_action(cx.listener(Self::on_action_optimize_nutexb_padding))
            .on_action(cx.listener(Self::on_action_open_wiki))
            .child(application_menu())
            .child(
                Checkbox::new("save-original-folder")
                    .label("Save to original folder")
                    .checked(self.save_to_original_folder)
                    .on_change(cx.listener(|this, checked, _, cx| {
                        this.save_to_original_folder = *checked;
                        cx.notify();
                    })),
            )
            .when(!self.save_to_original_folder, |this| {
                this.child(
                    Button::new("select-folder")
                        .label("Select Folder...")
                        .secondary()
                        .w_40()
                        .on_click(cx.listener(|_this, _, window, cx| {
                            select_output_folder(window, cx);
                        })),
                )
            })
            .child(
                h_flex()
                    .gap(px(10.0))
                    .child(
                        Button::new("export")
                            .label("Export")
                            .primary()
                            .disabled(
                                !self.save_to_original_folder
                                    && self.export_folder.read(cx).is_empty(),
                            )
                            .w_40()
                            .on_click(cx.listener(|this, _, _window, cx| {
                                let files = &this.table_state.read(cx).delegate().files;
                                let export_folder = this.export_folder.read(cx);
                                let export_folder = (!export_folder.is_empty())
                                    .then_some(std::path::Path::new(export_folder));
                                // TODO: Log errors.
                                // TODO: set the button to show progress.
                                let _ = convert_and_export_files(
                                    files,
                                    export_folder,
                                    this.save_to_original_folder,
                                    this.overrides.read(cx),
                                );
                            })),
                    )
                    .child(self.export_folder.read(cx).clone()),
            )
            .child(self.overrides.clone())
            .child(
                div()
                    .size_full()
                    .on_children_prepainted(cx.processor(
                        |this, child_bounds: Vec<Bounds<Pixels>>, _, cx| {
                            // Use a workaround to compute a responsive layout.
                            update_table_column_sizes(this, child_bounds, cx);
                        },
                    ))
                    .child(DataTable::new(&self.table_state).large().stripe(true)),
            )
    }
}

fn select_output_folder(window: &mut Window, cx: &mut Context<Application>) {
    cx.spawn_in(window, async move |this, cx| {
        // Await the native OS file picker without blocking the UI thread
        let file_handle = AsyncFileDialog::new()
            .set_title("Select Output Folder")
            .pick_folder()
            .await;

        if let Some(handle) = file_handle {
            let path = handle.path().to_path_buf();

            // Update view state back on the UI context and notify for re-render
            this.update_in(cx, |view, _window, cx| {
                view.export_folder
                    .write(cx, path.to_string_lossy().to_string());
                cx.notify();
            })
            .ok();
        }
    })
    .detach();
}

fn update_table_column_sizes(
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
        state.delegate_mut().columns[name_column_index].width = remaining;
        state.refresh(cx);
    })
}

fn application_menu() -> impl IntoElement {
    h_flex()
        .child(
            Button::new("file")
                .ghost()
                .label("File")
                .dropdown_menu(|menu, _window, _cx| {
                    menu.menu("Add Files...", Box::new(AddFiles))
                        .menu("Clear Files", Box::new(ClearFiles))
                }),
        )
        .child(
            Button::new("batch")
                .ghost()
                .label("Batch")
                .dropdown_menu(|menu, _window, _cx| {
                    menu.menu(
                        "Optimize Nutexb Padding..,",
                        Box::new(OptimizeNutexbPadding),
                    )
                }),
        )
        .child(
            Button::new("help")
                .ghost()
                .label("Help")
                .dropdown_menu(|menu, _window, _cx| {
                    menu.menu_with_icon("Wiki", IconName::Github, Box::new(OpenWiki))
                }),
        )
}

fn initialize_themes(cx: &mut App) {
    // TODO: match theme to existing application
    ThemeRegistry::global_mut(cx)
        .load_themes_from_str(include_str!("themes.json"))
        .unwrap();

    let theme = ThemeRegistry::global(cx)
        .themes()
        .get(&SharedString::from("Ultimate Tex Dark"))
        .cloned()
        .unwrap();
    Theme::update(cx, |current| current.apply_config(&theme));
}

fn main() {
    application().with_assets(assets::Assets).run(|cx| {
        init(cx);

        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.on_action(|_: &Quit, cx| {
            cx.quit();
        });

        initialize_themes(cx);

        let options = WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some(concat!("Ultimate Tex ", env!("CARGO_PKG_VERSION")).into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        open_window(options, cx, |window, cx| {
            cx.new(|cx| Application::new(window, cx))
        })
        .unwrap();
    });
}
