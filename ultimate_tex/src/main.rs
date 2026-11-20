use gpui_kit::{
    base::{Disableable, h_flex, v_flex},
    component::{
        IconName, Sizable, Theme, ThemeRegistry,
        button::{Button, ButtonVariants},
        checkbox::Checkbox,
        menu::DropdownMenu,
        table::{DataTable, TableState},
    },
    prelude::FluentBuilder,
    *,
};
use rfd::AsyncFileDialog;

use crate::{
    app::{convert_and_export_files, optimize_nutexb_files},
    overrides::FileSettingsOverrides,
    table::{FilesTableDelegate, update_table_column_sizes},
};

mod app;
mod overrides;
mod table;

actions!(
    ultimate_tex,
    [AddFiles, ClearFiles, OptimizeNutexbPadding, OpenWiki, Quit]
);
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
    Theme::update(cx, |current| {
        current.apply_config(&theme);
        current.font_size = px(14.0);
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
