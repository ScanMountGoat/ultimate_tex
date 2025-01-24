// Prevents additional console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::Arc;

use dioxus::prelude::*;
use dioxus::{html::HasFileData, prelude::dioxus_elements::FileEngine};
use dioxus_desktop::{tao::window::Icon, Config, WindowBuilder};
use image_dds::{ImageFormat, Mipmaps, Quality};
use rfd::FileDialog;
use strum::IntoEnumIterator;

mod app;
use app::{optimize_nutexb_files, App, ImageFileType};

use crate::app::{load_files, pick_files};

fn main() {
    let image = image_dds::image::load_from_memory(include_bytes!("../icons/32x32.png")).unwrap();
    let icon = Icon::from_rgba(image.into_rgba8().into_raw(), 32, 32).unwrap();

    LaunchBuilder::desktop()
        .with_cfg(
            Config::new()
                .with_window(
                    WindowBuilder::new()
                        .with_title(concat!("Ultimate Tex ", env!("CARGO_PKG_VERSION")))
                        .with_window_icon(Some(icon)),
                )
                .with_disable_context_menu(true),
        )
        .launch(app);
}

fn app() -> Element {
    // TODO: Is there a better way of managing this state?
    let mut app = use_signal(App::default);
    let mut messages = use_signal(Vec::<String>::new);
    let mut is_file_open = use_signal(|| false);
    let mut is_batch_open = use_signal(|| false);
    let mut is_help_open = use_signal(|| false);
    let mut is_exporting = use_signal(|| false);

    // TODO: Clean up into more components?
    // Reduced options for global presets.
    let preset_file_types = [
        ImageFileType::Png,
        ImageFileType::Dds,
        ImageFileType::Nutexb,
        ImageFileType::Bntx,
    ];
    let preset_format_types = [
        (ImageFormat::BC7RgbaUnormSrgb, "Color (sRGB) + Alpha"),
        (ImageFormat::BC7RgbaUnorm, "Color (Linear) + Alpha"),
    ];
    let preset_mipmap_types = [
        (Mipmaps::GeneratedAutomatic, "Enabled"),
        (Mipmaps::Disabled, "Disabled"),
    ];

    let save_in_same_folder = app.read().settings.save_in_same_folder;

    let show_compressed_options = app
        .read()
        .settings
        .overrides
        .output_file_type
        .map(is_compressed_type)
        .unwrap_or(true);

    let output_folder_text = app
        .read()
        .settings
        .output_folder
        .as_ref()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or("No folder selected".to_string());

    let disable_export = (app.read().settings.output_folder.is_none() && !save_in_same_folder)
        || *is_exporting.read();

    let add_files = move |_| {
        is_file_open.set(false);

        spawn({
            async move {
                if let Some((new_thumbnails, new_settings)) =
                    tokio::task::spawn_blocking(pick_files).await.unwrap()
                {
                    app.with_mut(|a| {
                        add_image_files(a, new_thumbnails, new_settings);
                    });
                }
            }
        });
    };

    let add_dropped_files = move |file_engine: Arc<dyn FileEngine>| async move {
        let files = file_engine.files();
        let paths = files.iter().map(PathBuf::from).collect();
        let (new_thumbnails, new_settings) = tokio::task::spawn_blocking(move || load_files(paths))
            .await
            .unwrap();
        app.with_mut(|a| {
            add_image_files(a, new_thumbnails, new_settings);
        });
    };

    use_effect(|| {
        document::eval(
            r#"
                window.addEventListener("dragover", function(e)
                {
                    document.getElementById("drop-zone").style.visibility = "";
                    document.getElementById("drop-zone").style.pointerEvents = "all";

                });

                window.addEventListener("dragleave", function(e)
                {
                    document.getElementById("drop-zone").style.visibility = "hidden";
                    document.getElementById("drop-zone").style.pointerEvents = "none";
                });
            "#,
        );
    });

    let export_files = move |_| {
        spawn({
            async move {
                is_exporting.set(true);

                // The app doesn't store image data, so this clone is cheap.
                let app = app.read().clone();
                let new_messages =
                    tokio::task::spawn_blocking(move || app.convert_and_export_files().unwrap())
                        .await
                        .unwrap();

                *messages.write() = new_messages;
                is_exporting.set(false);
            }
        });
    };

    rsx! {
        style { {include_str!("./pico.min.css")} }
        style { {include_str!("./app.css")} }

        nav {
            ul {
                li {
                    details { role: "list", dir: "ltr", open: "{is_file_open}",
                        summary {
                            aria_haspopup: "listbox",
                            role: "link",
                            onclick: move |_| {
                                is_file_open.set(true);
                            },
                            "File"
                        }
                        ul { role: "listbox",
                            li {
                                a { onclick: add_files, "Add Files..." }
                            }
                            li {
                                a {
                                    onclick: move |_| {
                                        app.with_mut(|a| a.clear_files());
                                        is_file_open.set(false);
                                    },
                                    "Clear Files..."
                                }
                            }
                        }
                    }
                }
                li {
                    details { role: "list", dir: "ltr", open: "{is_batch_open}",
                        summary {
                            aria_haspopup: "listbox",
                            role: "link",
                            onclick: move |_| {
                                is_batch_open.set(true);
                            },
                            "Batch"
                        }
                        ul { role: "listbox",
                            li {
                                a {
                                    onclick: move |_| {
                                        optimize_nutexb_files();
                                        is_batch_open.set(false);
                                    },
                                    "Optimize Nutexb Padding..."
                                }
                            }
                        }
                    }
                }
                li {
                    details { role: "list", dir: "ltr", open: "{is_help_open}",
                        summary {
                            aria_haspopup: "listbox",
                            role: "link",
                            onclick: move |_| {
                                is_help_open.set(true);
                            },
                            "Help"
                        }
                        ul { role: "listbox",
                            li {
                                a {
                                    onclick: move |_| {
                                        is_help_open.set(false);
                                        let _ = open::that("https://github.com/ScanMountGoat/ultimate_tex/wiki");
                                    },
                                    "Wiki"
                                }
                            }
                        }
                    }
                }
            }
        }
        hr {}

        label {
            input {
                r#type: "checkbox",
                checked: "{save_in_same_folder}",
                onchange: move |e| {
                    app.with_mut(|a| a.settings.save_in_same_folder = e.value().parse().unwrap());
                },
            }
            "Save to original folder"
        }
        if !save_in_same_folder {
            // TODO: use an input with type directory instead?
            // TODO: display the folder value
            div { class: "grid-horizontal",
                button {
                    style: "width: auto;",
                    class: "secondary",
                    onclick: move |_| {
                        if let Some(folder) = FileDialog::new()
                            .set_title("Select Nutexb Root Folder")
                            .pick_folder()
                        {
                            app.with_mut(|a| a.settings.output_folder = Some(folder));
                        }
                    },
                    "Select Folder..."
                }
                div { class: "message-text", "{output_folder_text}" }
            }
        }
        div { class: "grid-horizontal",
            button {
                style: "width: 150px;",
                disabled: disable_export,
                onclick: export_files,
                "Export"
            }
            for message in messages.read().iter() {
                div { class: "message-text", "{message}" }
            }
        }
        hr {}

        div { class: "flex-container",
            fieldset {
                legend {
                    strong { "Output Type" }
                }
                for option in preset_file_types {
                    label { r#for: "outputType{option}",
                        input {
                            r#type: "radio",
                            id: "outputType{option}",
                            name: "outputType",
                            value: "{option}",
                            checked: app.read().settings.overrides.output_file_type == Some(option),
                            oninput: move |e| {
                                app.with_mut(|a| {
                                    a.settings.overrides.output_file_type = Some(e.value().parse().unwrap());
                                });
                            },
                        }
                        "{option}"
                    }
                }
                label { r#for: "outputTypeNull",
                    input {
                        r#type: "radio",
                        id: "outputTypeNull",
                        name: "outputType",
                        value: "",
                        checked: app.read().settings.overrides.output_file_type.is_none(),
                        oninput: move |_| {
                            app.with_mut(|a| a.settings.overrides.output_file_type = None);
                        },
                    }
                    "Custom..."
                }
            }

            if show_compressed_options {
                fieldset {
                    legend {
                        strong { "Output Format" }
                    }
                    for (option , option_name) in preset_format_types {
                        label { r#for: "outputFormat{option}",
                            input {
                                r#type: "radio",
                                id: "outputFormat{option}",
                                name: "outputFormat",
                                value: "{option}",
                                checked: app.read().settings.overrides.output_format == Some(option),
                                oninput: move |e| {
                                    app.with_mut(|a| {
                                        a.settings.overrides.output_format = Some(e.value().parse().unwrap());
                                    });
                                },
                            }
                            {option_name}
                        }
                    }
                    label { r#for: "outputFormatNull",
                        input {
                            r#type: "radio",
                            id: "outputFormatNull",
                            name: "outputFormat",
                            value: "",
                            checked: app.read().settings.overrides.output_format.is_none(),
                            oninput: move |_| {
                                app.with_mut(|a| a.settings.overrides.output_format = None);
                            },
                        }
                        "Custom..."
                    }
                }
                fieldset {
                    legend {
                        strong { "Mipmaps" }
                    }
                    for (option , option_name) in preset_mipmap_types {
                        label { r#for: "mipmaps{option}",
                            input {
                                r#type: "radio",
                                id: "mipmaps{option}",
                                name: "mipmaps",
                                value: "{option}",
                                checked: app.read().settings.overrides.mipmaps == Some(option),
                                oninput: move |e| {
                                    app.with_mut(|a| {
                                        a.settings.overrides.mipmaps = Some(e.value().parse().unwrap());
                                    });
                                },
                            }
                            {option_name}
                        }
                    }
                    label { r#for: "mipmapsNull",
                        input {
                            r#type: "radio",
                            id: "mipmapsNull",
                            name: "mipmaps",
                            value: "",
                            checked: app.read().settings.overrides.mipmaps.is_none(),
                            oninput: move |_| {
                                app.with_mut(|a| a.settings.overrides.mipmaps = None);
                            },
                        }
                        "Custom..."
                    }
                }
                fieldset {
                    legend {
                        strong { "Compression" }
                    }
                    for option in Quality::iter() {
                        label { r#for: "compression{option}",
                            input {
                                r#type: "radio",
                                id: "compression{option}",
                                name: "compression",
                                value: "{option}",
                                checked: app.read().settings.overrides.output_quality == Some(option),
                                oninput: move |e| {
                                    app.with_mut(|a| {
                                        a.settings.overrides.output_quality = Some(e.value().parse().unwrap());
                                    });
                                },
                            }
                            "{option}"
                        }
                    }
                    label { r#for: "compressionNull",
                        input {
                            r#type: "radio",
                            id: "compressionNull",
                            name: "compression",
                            value: "",
                            checked: app.read().settings.overrides.output_quality.is_none(),
                            oninput: move |_| {
                                app.with_mut(|a| a.settings.overrides.output_quality = None);
                            },
                        }
                        "Custom..."
                    }
                }
            }
        }

        figure {
            table { role: "grid",
                thead {
                    tr {
                        th { scope: "col",
                            strong { "Image" }
                        }
                        th { scope: "col",
                            strong { "Name" }
                        }
                        th { scope: "col",
                            strong { "Format" }
                        }
                        th { scope: "col",
                            strong { "Size" }
                        }
                        th { scope: "col",
                            strong { "Output Type" }
                        }
                        th { scope: "col",
                            strong { "Output Format" }
                        }
                        th { scope: "col",
                            strong { "Compression" }
                        }
                        th { scope: "col",
                            strong { "Mipmaps" }
                        }
                        th {}
                    }
                }
                tbody {
                    for (i , item) in app.read().settings.file_settings.iter().enumerate() {
                        tr { key: "{item.name}",
                            td {
                                img { src: "{app.read().png_thumbnails[i]}" }
                            }
                            td { "{item.name}" }
                            td { "{item.format}" }
                            td { "{item.dimensions.0}x{item.dimensions.1}x{item.dimensions.2}" }
                            td {
                                match app.read().settings.overrides.output_file_type {
                                    Some(ty) => rsx! {
                                    "{ty}"
                                    },
                                    None => rsx! {
                                        select {
                                            onchange: move |e| {
                                                app.with_mut(|a| {
                                                    a.settings.file_settings[i].output_file_type = e.value().parse().unwrap();
                                                });
                                            },
                                            for variant in ImageFileType::iter() {
                                                option { selected: item.output_file_type == variant, value: "{variant}", "{variant}" }
                                            }
                                        }
                                    },
                                }
                            }
                            td {
                                match app.read().settings.overrides.output_format {
                                    Some(ty) => rsx! {
                                    "{ty}"
                                    },
                                    None => rsx! {
                                        select {
                                            onchange: move |e| {
                                                app.with_mut(|a| {
                                                    a.settings.file_settings[i].output_format = e.value().parse().unwrap();
                                                });
                                            },
                                            for variant in ImageFormat::iter() {
                                                option { selected: item.output_format == variant, value: "{variant}", "{variant}" }
                                            }
                                        }
                                    },
                                }
                            }
                            td {
                                match app.read().settings.overrides.output_quality {
                                    Some(ty) => rsx! {
                                    "{ty}"
                                    },
                                    None => rsx! {
                                        select {
                                            onchange: move |e| {
                                                app.with_mut(|a| {
                                                    a.settings.file_settings[i].output_quality = e.value().parse().unwrap();
                                                });
                                            },
                                            for variant in Quality::iter() {
                                                option { selected: item.output_quality == variant, value: "{variant}", "{variant}" }
                                            }
                                        }
                                    },
                                }
                            }
                            td {
                                match app.read().settings.overrides.mipmaps {
                                    Some(ty) => rsx! {
                                    "{ty}"
                                    },
                                    None => rsx! {
                                        select {
                                            onchange: move |e| {
                                                app.with_mut(|a| {
                                                    a.settings.file_settings[i].output_mipmaps = e.value().parse().unwrap();
                                                });
                                            },
                                            for variant in Mipmaps::iter() {
                                                option { selected: item.output_mipmaps == variant, value: "{variant}", "{variant}" }
                                            }
                                        }
                                    },
                                }
                            }
                            td {
                                button {
                                    class: "secondary",
                                    onclick: move |_| {
                                        app.with_mut(|a| a.remove_file(i));
                                    },
                                    "Remove"
                                }
                            }
                        }
                    }
                }
            }
        }
        if app.read().settings.file_settings.is_empty() {
            div { class: "centered-text",
                "Drag and drop image files onto the window or add files using File > Add Files..."
            }
        }
        div {
            id: "drop-zone",
            class: "drop-zone",
            ondrop: move |e| async move {
                if let Some(file_engine) = e.files() {
                    add_dropped_files(file_engine).await;
                }
            },
        }
    }
}

fn add_image_files(
    a: &mut App,
    new_thumbnails: Vec<String>,
    new_settings: Vec<app::ImageFileSettings>,
) {
    // Prevent adding duplicate paths.
    for (thumbnail, settings) in new_thumbnails.into_iter().zip(new_settings) {
        if !a
            .settings
            .file_settings
            .iter()
            .any(|t| t.name == settings.name)
        {
            a.png_thumbnails.push(thumbnail);
            a.settings.file_settings.push(settings);
        }
    }
}

fn is_compressed_type(ty: ImageFileType) -> bool {
    ty != ImageFileType::Png && ty != ImageFileType::Tiff
}
