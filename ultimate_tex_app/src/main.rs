#![allow(non_snake_case)]
// Prevents additional console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use dioxus::prelude::*;
use dioxus_desktop::WindowBuilder;
use image_dds::{ImageFormat, Mipmaps, Quality};
use rfd::FileDialog;
use strum::IntoEnumIterator;

mod app;
use app::{optimize_nutexb_files, App, ImageFileType};

fn main() {
    dioxus_desktop::launch_cfg(
        App,
        dioxus_desktop::Config::new()
            .with_window(
                WindowBuilder::new()
                    .with_title(concat!("Ultimate Tex ", env!("CARGO_PKG_VERSION"))),
            )
            .with_disable_context_menu(true),
    );
}

#[component]
fn App(cx: Scope) -> Element {
    // TODO: Is there a better way of managing this state?
    let app = use_ref(cx, || App::default());
    let messages = use_ref(cx, || Vec::new());

    // TODO: Clean up into more components?
    // Reduced options for global presets.
    let preset_file_types = [
        ImageFileType::Png,
        ImageFileType::Dds,
        ImageFileType::Nutexb,
        ImageFileType::Bntx,
    ];
    let preset_format_types = ["Color (sRGB) + Alpha", "Color (Linear) + Alpha"];
    let preset_mipmap_types = ["Enabled", "Disabled"];

    let save_in_same_folder = app.read().settings.save_in_same_folder;

    let is_override_type_compressed = app
        .read()
        .settings
        .overrides
        .output_file_type
        .map(is_compressed_type)
        .unwrap_or_default();

    cx.render(rsx! {
        style { {include_str!("./pico.min.css")} }
        style { {include_str!("./app.css")} }
        // TODO: menus don't close on click
        nav {
            ul {
                li {
                    details {
                        role: "list",
                        dir: "ltr",
                        summary {
                            aria_haspopup: "listbox",
                            role: "link",
                            "File"
                        }
                        ul {
                            role: "listbox",
                            li {
                                a {
                                    onclick: move |_| {
                                        app.with_mut(|a| a.add_files());
                                    },
                                    "Add Files..."
                                }
                            }
                            li {
                                a {
                                    onclick: move |_| {
                                        app.with_mut(|a| a.clear_files());
                                    },
                                    "Clear Files..."
                                }
                            }
                        }
                    }
                }
                li {
                    details {
                        role: "list",
                        dir: "ltr",
                        summary {
                            aria_haspopup: "listbox",
                            role: "link",
                            "Batch"
                        }
                        ul {
                            role: "listbox",
                            li {
                                a {
                                    onclick: move |_| { optimize_nutexb_files() },
                                    "Optimize Nutexb Padding..."
                                }
                            }
                        }
                    }
                }
                li {
                    details {
                        role: "list",
                        dir: "ltr",
                        summary {
                            aria_haspopup: "listbox",
                            role: "link",
                            "Help"
                        }
                        ul {
                            role: "listbox",
                            li {
                                a {
                                    onclick: move |_| {
                                        if let Err(_) = open::that("https://github.com/ScanMountGoat/ultimate_tex/wiki") {
                                            // TODO: log errors
                                        }
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
                    app.with_mut(|a| a.settings.save_in_same_folder = e.value.parse().unwrap());
                }

            }
            "Save to original folder"
        }
        if !save_in_same_folder {
            // TODO: use an input with type directory instead?
            // TODO: display the folder value
            rsx! {
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
            }
        }
        button {
            style: "width: 150px;",
            onclick: move |_| {
                *messages.write() = app.with(|a| a.convert_and_export_files().unwrap());
            },
            "Export"
        }
        hr {}

        // TODO: Modify override values.
        // TODO: Disable appropriate elements if not compressed
        div {
            class: "flex-container",
            fieldset {
                legend { strong { "Output Type" } }
                for option in preset_file_types {
                    label {
                        r#for: "outputType{option}",
                        input {
                            r#type: "radio",
                            id: "outputType{option}",
                            name: "outputType",
                            value: "{option}",
                            oninput: move |e| {
                                app.with_mut(|a| a.settings.overrides.output_file_type = Some(e.value.parse().unwrap()));
                            }
                        }
                        "{option}"
                    }
                }
                label {
                    r#for: "outputTypeNull",
                    input {
                        r#type: "radio",
                        id: "outputTypeNull",
                        name: "outputType",
                        value: "",
                        oninput: move |_| {
                            app.with_mut(|a| a.settings.overrides.output_file_type = None);
                        }
                    }
                    "Custom..."
                }
            }
            fieldset {
                legend { strong { "Output Format" } }
                for option in preset_format_types {
                    label {
                        r#for: "outputFormat{option}",
                        input {
                            r#type: "radio",
                            id: "outputFormat{option}",
                            name: "outputFormat",
                            value: option,
                            oninput: move |e| {
                                app.with_mut(|a| a.settings.overrides.output_format = Some(e.value.parse().unwrap()));
                            }
                        }
                        option
                    }
                }
                label {
                    r#for: "outputFormatNull",
                    input {
                        r#type: "radio",
                        id: "outputFormatNull",
                        name: "outputFormat",
                        value: "",
                        oninput: move |_| {
                            app.with_mut(|a| a.settings.overrides.output_file_type = None);
                        }
                    }
                    "Custom..."
                }
            }
            fieldset {
                legend { strong { "Mipmaps" } }
                for option in preset_mipmap_types {
                    label {
                        r#for: "mipmaps{option}",
                        input {
                            r#type: "radio",
                            id: "mipmaps{option}",
                            name: "mipmaps",
                            value: option,
                            oninput: move |e| {
                                app.with_mut(|a| a.settings.overrides.mipmaps = Some(e.value.parse().unwrap()));
                            }
                        }
                        option
                    }
                }
                label {
                    r#for: "mipmapsNull",
                    input {
                        r#type: "radio",
                        id: "mipmapsNull",
                        name: "mipmaps",
                        value: "",
                        oninput: move |_| {
                            app.with_mut(|a| a.settings.overrides.mipmaps = None);
                        }
                    }
                    "Custom..."
                }
            }
            fieldset {
                legend { strong { "Compression" } }
                for option in Quality::iter() {
                    label {
                        r#for: "compression{option}",
                        input {
                            r#type: "radio",
                            id: "compression{option}",
                            name: "compression",
                            value: "{option}",
                            oninput: move |e| {
                                app.with_mut(|a| a.settings.overrides.compression_quality = Some(e.value.parse().unwrap()));
                            }
                        }
                        "{option}"
                    }
                }
                label {
                    r#for: "compressionNull",
                    input {
                        r#type: "radio",
                        id: "compressionNull",
                        name: "compression",
                        value: "",
                        oninput: move |_| {
                            app.with_mut(|a| a.settings.overrides.compression_quality = None);
                        }
                    }
                    "Custom..."
                }
            }
        }

        figure {
            table {
                role: "grid",
                thead {
                    tr {
                        th { scope: "col", strong { "Name" } }
                        th { scope: "col", strong { "Format" } }
                        th { scope: "col", strong { "Size" } }
                        th { scope: "col", strong { "Output Type" } }
                        th { scope: "col", strong { "Output Format" } }
                        th { scope: "col", strong { "Compression" } }
                        th { scope: "col", strong { "Mipmaps" } }
                        th {}
                    }
                }
                tbody {
                    for (i, item) in app.read().settings.file_settings.iter().enumerate() {
                        tr {
                            key: "{item.name}",
                            th { scope: "row", "{item.name}" }
                            th { "{item.format}" }
                            th { "{item.dimensions.0}x{item.dimensions.1}x{item.dimensions.2}" }
                            th {
                                select {
                                    onchange: move |e| {
                                        app.with_mut(|a| a.settings.file_settings[i].output_file_type = e.value.parse().unwrap());
                                    },
                                    for variant in ImageFileType::iter() {
                                        option { value: "{item.output_file_type}", "{variant}" }
                                    }
                                }
                            }
                            th {
                                select {
                                    onchange: move |e| {
                                        app.with_mut(|a| a.settings.file_settings[i].output_format = e.value.parse().unwrap());
                                    },
                                    for variant in ImageFormat::iter() {
                                        option { value: "{item.output_format}", "{variant}" }
                                    }
                                }
                            }
                            th {
                                select {
                                    onchange: move |e| {
                                        app.with_mut(|a| a.settings.file_settings[i].output_quality = e.value.parse().unwrap());
                                    },
                                    for variant in Quality::iter() {
                                        option { value: "{item.output_quality}", "{variant}" }
                                    }
                                }
                            }
                            th {
                                select {
                                    onchange: move |e| {
                                        app.with_mut(|a| a.settings.file_settings[i].output_mipmaps = e.value.parse().unwrap());
                                    },
                                    for variant in Mipmaps::iter() {
                                        option { value: "{item.output_mipmaps}", "{variant}" }
                                    }
                                }
                            }
                            th {
                                button {
                                    class: "secondary",
                                    onclick: move |_| { app.with_mut(|a| a.remove_file(i)); },
                                    "Remove"
                                }
                            }
                        }
                    }

                }
            }
        }

        footer {
            hr {}
            for message in messages.read().iter() {
                "{message}"
            }
        }
    })
}

fn is_compressed_type(ty: ImageFileType) -> bool {
    ty != ImageFileType::Png && ty != ImageFileType::Tiff
}
