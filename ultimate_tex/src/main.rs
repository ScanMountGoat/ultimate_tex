#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use gpui_kit::{
    base::{Disableable, h_flex, v_flex},
    component::{
        ActiveTheme, IconName, Sizable, Theme, ThemeRegistry, TitleBar,
        button::{Button, ButtonVariants},
        checkbox::Checkbox,
        list::{List, ListState},
        menu::DropdownMenu,
        popover::Popover,
        progress::Progress,
        status_bar::StatusBar,
        table::{DataTable, TableState},
    },
    prelude::FluentBuilder,
    *,
};
use rfd::AsyncFileDialog;

use crate::{
    app::{convert_and_export_files, optimize_nutexb_files},
    log::LogListDelegate,
    overrides::FileSettingsOverrides,
    table::{FilesTableDelegate, update_table_column_sizes},
};

mod app;
mod log;
mod overrides;
mod table;

actions!(
    ultimate_tex,
    [AddFiles, ClearFiles, OptimizeNutexbPadding, OpenWiki, Quit]
);
struct Application {
    export_folder: SharedString,
    save_to_original_folder: bool,
    table_state: Entity<TableState<FilesTableDelegate>>,
    overrides: Entity<FileSettingsOverrides>,
    is_processing: bool,
    status_message: SharedString,
    log_list: Entity<ListState<LogListDelegate>>,
    application_logo: Arc<Image>,
}

impl Application {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let overrides = cx.new(|_cx| FileSettingsOverrides::default());

        let delegate = FilesTableDelegate::new(overrides.clone());
        let table_state = cx.new(|cx| {
            // Changing the overrides should update the table rendering.
            cx.observe(&overrides, |_this, _target, cx| cx.notify())
                .detach();

            TableState::new(delegate, window, cx)
                .row_selectable(false)
                .col_selectable(false)
                .col_movable(false)
                .col_resizable(false)
        });

        let log_list = cx.new(|cx| ListState::new(LogListDelegate::default(), window, cx));

        let application_logo = Arc::new(Image::from_bytes(
            ImageFormat::Svg,
            include_bytes!("../icons/UltimateTexIcon.svg").to_vec(),
        ));

        Self {
            export_folder: SharedString::default(),
            table_state,
            save_to_original_folder: false,
            overrides,
            status_message: SharedString::default(),
            is_processing: false,
            log_list,
            application_logo,
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
        cx.spawn(async |this, cx| {
            // Await the native OS file picker without blocking the UI thread
            let file_handle = AsyncFileDialog::new()
                .set_title("Select Nutexb Root Folder")
                .pick_folder()
                .await;

            if let Some(handle) = file_handle {
                this.update(cx, |view, cx| {
                    view.status_message = "Optimizing nutexb padding...".into();
                    view.is_processing = true;
                    cx.notify();
                })
                .unwrap();

                let start = std::time::Instant::now();
                let optimized_count = cx
                    .background_spawn(async move { optimize_nutexb_files(handle.path()) })
                    .await;
                let elapsed = start.elapsed();

                this.update(cx, |view, cx| {
                    view.status_message =
                        format!("Optimized {optimized_count} nutexb files").into();
                    view.is_processing = false;
                    cx.notify();

                    // TODO: connect this to tracing-subscriber instead.
                    view.log_list.update(cx, |state, cx| {
                        state.delegate_mut().items.push(format!(
                            "Optimized {optimized_count} nutexb files: {:?}",
                            elapsed
                        ));
                        cx.notify();
                    });
                })
                .unwrap();
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_export_disabled = (!self.save_to_original_folder && self.export_folder.is_empty())
            || self.table_state.read(cx).delegate().files.is_empty()
            || self.is_processing;
        v_flex()
            .size_full()
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
            .child(
                TitleBar::new()
                    .text_color(if window.is_window_active() {
                        cx.theme().accent_foreground
                    } else {
                        cx.theme().muted_foreground
                    })
                    .text_sm()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .when(cfg!(windows), |this| {
                                this.child(img(self.application_logo.clone()).size(px(16.0)))
                            })
                            .child(concat!("Ultimate Tex ", env!("CARGO_PKG_VERSION")).to_string()),
                    ),
            )
            .child(
                v_flex()
                    .size_full()
                    .gap(px(10.0))
                    .p(px(10.0))
                    .text_size(px(14.0))
                    .child(application_menu(self.is_processing, cx))
                    .child(
                        Checkbox::new("save-original-folder")
                            .label("Save to original folder")
                            .text_size(px(14.0))
                            .checked(self.save_to_original_folder)
                            .on_change(cx.listener(|this, checked, _, cx| {
                                this.save_to_original_folder = *checked;
                                cx.notify();
                            })),
                    )
                    .when(!self.save_to_original_folder, |this| {
                        this.child(
                            h_flex()
                                .gap(px(10.0))
                                .child(
                                    Button::new("select-folder")
                                        .label("Select Folder...")
                                        .secondary()
                                        .w_40()
                                        .on_click(cx.listener(|_this, _, window, cx| {
                                            select_output_folder(window, cx);
                                        })),
                                )
                                .child(self.export_folder.clone()),
                        )
                    })
                    .child(
                        Button::new("export")
                            .label("Export")
                            .primary()
                            .disabled(is_export_disabled)
                            .w_40()
                            .on_click(cx.listener(|this, _, _window, cx| {
                                let files: Vec<_> = this
                                    .table_state
                                    .read(cx)
                                    .delegate()
                                    .files
                                    .iter()
                                    .map(|f| f.settings.clone())
                                    .collect();

                                let export_folder = this.export_folder.clone();
                                let export_folder = (!export_folder.is_empty())
                                    .then_some(std::path::PathBuf::from(export_folder.as_str()));

                                let save_to_original_folder = this.save_to_original_folder;
                                let overrides = this.overrides.read(cx).clone();

                                this.status_message =
                                    format!("Exporting {} files...", files.len()).into();
                                this.is_processing = true;
                                cx.spawn(async move |this, cx| {
                                    let start = std::time::Instant::now();
                                    let result = cx
                                        .background_spawn(async move {
                                            convert_and_export_files(
                                                files,
                                                export_folder,
                                                save_to_original_folder,
                                                overrides,
                                            )
                                        })
                                        .await;
                                    let elapsed = start.elapsed();

                                    let file_count =
                                        result.as_ref().map(|f| f.len()).unwrap_or_default();
                                    this.update(cx, |view, cx| {
                                        view.status_message =
                                            format!("Exported {file_count} files").into();
                                        view.is_processing = false;
                                        cx.notify();

                                        // TODO: connect this to tracing-subscriber instead.
                                        view.log_list.update(cx, |state, cx| {
                                            state.delegate_mut().items.push(format!(
                                                "Exported {file_count} files: {:?}",
                                                elapsed
                                            ));
                                            if let Ok(result) = result {
                                                for item in result.into_iter().flatten() {
                                                    state.delegate_mut().items.push(item);
                                                }
                                            }
                                            cx.notify();
                                        });
                                    })
                                })
                                .detach();
                            })),
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
                            .child(DataTable::new(&self.table_state).large()),
                    )
                    .child(
                        StatusBar::new()
                            .text_base()
                            .bg(cx.theme().background)
                            .left(self.status_message.clone())
                            .when(self.is_processing, |this| {
                                this.child(Progress::new("progress").loading(true))
                            })
                            .right(
                                Popover::new("log-popover")
                                    .trigger(
                                        Button::new("open-logs")
                                            .label("Logs")
                                            .icon(IconName::TriangleAlert),
                                    )
                                    .child(
                                        div()
                                            .w(px(400.0))
                                            .h(px(400.0))
                                            .child(List::new(&self.log_list)),
                                    ),
                            ),
                    ),
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
                view.export_folder = path.to_string_lossy().into();
                cx.notify();
            })
            .ok();
        }
    })
    .detach();
}

fn application_menu(is_processing: bool, _cx: &mut Context<Application>) -> impl IntoElement {
    // TODO: use foreground text color and accent foreground when hovered or active?
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
        .child(Button::new("batch").ghost().label("Batch").dropdown_menu(
            move |menu, _window, _cx| {
                menu.menu_with_enable(
                    "Optimize Nutexb Padding...",
                    Box::new(OptimizeNutexbPadding),
                    !is_processing,
                )
            },
        ))
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
    Theme::update(cx, |current| {
        current.apply_config(&theme);
        // This fully disables hover colors since the table isn't striped.
        current.table_hover = Hsla::transparent_black();
    });
}

fn main() {
    application().with_assets(assets::Assets).run(|cx| {
        init(cx);

        cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        cx.on_action(|_: &Quit, cx| {
            cx.quit();
        });

        initialize_themes(cx);

        let options = TitleBar::window_options();
        open_window(options, cx, |window, cx| {
            cx.new(|cx| Application::new(window, cx))
        })
        .unwrap();
    });
}
