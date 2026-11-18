use std::path::PathBuf;

use gpui_kit::{
    base::{IndexPath, h_flex, v_flex},
    component::{
        IconName, Sizable, Theme, ThemeRegistry,
        button::{Button, ButtonVariants},
        checkbox::Checkbox,
        form::field,
        menu::DropdownMenu,
        radio::RadioGroup,
        select::{Select, SelectItem, SelectState},
        table::{Column, DataTable, TableDelegate, TableState},
    },
    prelude::FluentBuilder,
    *,
};
use rfd::AsyncFileDialog;
use strum::IntoEnumIterator;

use crate::app::{ImageFileSettings, ImageFileType, load_files};

// TODO: add app.rs from ultimate_tex (add dependencies)
mod app;

actions!(
    ultimate_tex,
    [AddFiles, ClearFiles, OptimizeNutexbPadding, OpenWiki]
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
    output_type_select: Entity<SelectState<Vec<EnumSelectItem<ImageFileType>>>>,
    output_format_select: Entity<SelectState<Vec<EnumSelectItem<image_dds::ImageFormat>>>>,
    compression_select: Entity<SelectState<Vec<EnumSelectItem<image_dds::Quality>>>>,
    mipmaps_select: Entity<SelectState<Vec<EnumSelectItem<image_dds::Mipmaps>>>>,
}

struct FilesTableDelegate {
    files: Vec<File>,
    columns: Vec<Column>,
}

impl FilesTableDelegate {
    fn new() -> Self {
        Self {
            files: Vec::new(),
            columns: vec![
                Column::new("image", "Image"),
                Column::new("name", "Name"),
                Column::new("format", "Format"),
                Column::new("size", "Size"),
                Column::new("output-type", "Output Type").width(px(150.0)),
                Column::new("output-format", "Output Format").width(px(200.0)),
                Column::new("compression", "Compression").width(px(150.0)),
                Column::new("mipmaps", "Mipmaps").width(px(210.0)),
                Column::new("remove", ""),
            ],
        }
    }

    fn add_files<T>(&mut self, paths: &[PathBuf], window: &mut Window, cx: &mut Context<T>) {
        // TODO: par_extend once this actually loads from disk
        // TODO: don't add duplicate files

        let (thumbnails, settings) = load_files(paths);

        // TODO: set the selects to update the settings.
        self.files.extend(settings.into_iter().map(|settings| {
            let output_format_select =
                cx.new(|cx| enum_select_state(settings.output_format, window, cx));
            let compression_select =
                cx.new(|cx| enum_select_state(settings.output_quality, window, cx));
            let mipmaps_select =
                cx.new(|cx| enum_select_state(settings.output_mipmaps, window, cx));
            File {
                settings,
                output_type_select: cx.new(|cx| {
                    let variants = vec![
                        EnumSelectItem(ImageFileType::Png),
                        EnumSelectItem(ImageFileType::Dds),
                        EnumSelectItem(ImageFileType::Nutexb),
                        EnumSelectItem(ImageFileType::Bntx),
                    ];
                    SelectState::new(variants, Some(IndexPath::new(0)), window, cx)
                }),
                output_format_select,
                compression_select,
                mipmaps_select,
            }
        }));
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

        // TODO: where to handle size formatting?
        // TODO: why does the striping change colors when clicking elements?
        // TODO: make the columns fill all available space?
        let child = match col.key.as_ref() {
            "name" => div()
                .child(row.settings.name.clone())
                .truncate()
                .into_any_element(),
            // TODO: handle remaining columns.
            "output-type" => div()
                .w_full()
                .child(Select::new(&row.output_type_select))
                .into_any_element(),
            "output-format" => div()
                .w_full()
                .child(Select::new(&row.output_format_select))
                .into_any_element(),
            "compression" => div()
                .w_full()
                .child(Select::new(&row.compression_select))
                .into_any_element(),
            "mipmaps" => div()
                .w_full()
                .child(Select::new(&row.mipmaps_select))
                .into_any_element(),
            "remove" => div()
                .w_full()
                .child(
                    Button::new(format!("remove{row_ix}"))
                        .label("Remove")
                        .secondary()
                        .on_click(cx.listener(move |this, _, window, cx| {
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
}

impl Application {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let export_folder = cx.new(|cx| String::new());
        let delegate = FilesTableDelegate::new();
        let table_state = cx.new(|cx| {
            TableState::new(delegate, window, cx)
                .col_selectable(false)
                .col_movable(false)
                .col_resizable(false)
        });

        Self {
            export_folder,
            table_state,
            save_to_original_folder: false,
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
        window: &mut Window,
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
                // TODO: batch process the nutexb files in parallel
            }
        })
        .detach();
    }

    fn on_action_open_wiki(&mut self, _: &OpenWiki, _window: &mut Window, _cx: &mut Context<Self>) {
        let _ = open::that("https://github.com/ScanMountGoat/ultimate_tex/wiki");
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
                        .on_click(cx.listener(|this, _, window, cx| {
                            select_output_folder(window, cx);
                        })),
                )
            })
            .child(Button::new("export").label("Export").primary().w_40())
            .child(
                h_flex()
                    .items_start()
                    .child(field().label("Output Type").child(
                        RadioGroup::vertical("output-type").children([
                            "Png",
                            "Dds",
                            "Nutexb",
                            "Bntx",
                            "Custom...",
                        ]),
                    ))
                    .child(field().label("Output Format").child(
                        RadioGroup::vertical("output-format").children([
                            "Color (sRGB) + Alpha",
                            "Color (Linear) + Alpha",
                            "Custom...",
                        ]),
                    ))
                    .child(field().label("Mipmaps").child(
                        RadioGroup::vertical("mipmaps").children([
                            "Enabled",
                            "Disabled",
                            "Custom...",
                        ]),
                    ))
                    .child(field().label("Compression").child(
                        RadioGroup::vertical("compression").children([
                            "Fast",
                            "Normal",
                            "Slow",
                            "Custom...",
                        ]),
                    )),
            )
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
    let name_column_index = 1;
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
    // TODO: choose a different background for list even and list active
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
