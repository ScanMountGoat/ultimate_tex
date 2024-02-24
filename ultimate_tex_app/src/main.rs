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
    let app = use_ref(cx, App::default);
    let messages = use_ref(cx, Vec::new);
    let is_file_open = use_state(cx, || false);
    let is_batch_open = use_state(cx, || false);
    let is_help_open = use_state(cx, || false);
    let is_exporting = use_state(cx, || false);

    // TODO: Clean up into more components?
    // Reduced options for global presets.
    let preset_file_types = [
        ImageFileType::Png,
        ImageFileType::Dds,
        ImageFileType::Nutexb,
        ImageFileType::Bntx,
    ];
    let preset_format_types = [
        (ImageFormat::BC7Srgb, "Color (sRGB) + Alpha"),
        (ImageFormat::BC7Unorm, "Color (Linear) + Alpha"),
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

    let disable_export =
        (app.read().settings.output_folder.is_none() && !save_in_same_folder) || **is_exporting;

    cx.render(rsx! {
        style { {include_str!("./pico.min.css")} }
        style { {include_str!("./app.css")} }

        nav {
            ul {
                li {
                    details {
                        role: "list",
                        dir: "ltr",
                        open: "{is_file_open}",
                        summary {
                            aria_haspopup: "listbox",
                            role: "link",
                            onclick: move |_| {
                                is_file_open.set(true);
                            },
                            "File"
                        }
                        ul {
                            role: "listbox",
                            li {
                                a {
                                    onclick: move |_| {
                                        app.with_mut(|a| a.add_files());
                                        is_file_open.set(false);
                                    },
                                    "Add Files..."
                                }
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
                    details {
                        role: "list",
                        dir: "ltr",
                        open: "{is_batch_open}",
                        summary {
                            aria_haspopup: "listbox",
                            role: "link",
                            onclick: move |_| {
                                is_batch_open.set(true);
                            },
                            "Batch"
                        }
                        ul {
                            role: "listbox",
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
                    details {
                        role: "list",
                        dir: "ltr",
                        open: "{is_help_open}",
                        summary {
                            aria_haspopup: "listbox",
                            role: "link",
                            onclick: move |_| {
                                is_help_open.set(true);
                            },
                            "Help"
                        }
                        ul {
                            role: "listbox",
                            li {
                                a {
                                    onclick: move |_| {
                                        is_help_open.set(false);
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
                div {
                    class: "file-container",
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
                    div {
                        class: "file-text",
                        "{output_folder_text}"
                    }
                }
            }
        }
        button {
            style: "width: 150px;",
            disabled: disable_export,
            onclick: move |_| {
                // TODO: make this async.
                is_exporting.set(true);
                *messages.write() = app.with(|a| a.convert_and_export_files().unwrap());
                is_exporting.set(false);
            },
            "Export"
        }
        hr {}

        // TODO: select appropriate option by default.
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

            if show_compressed_options {
                rsx! {
                    fieldset {
                        legend { strong { "Output Format" } }
                        for (option, option_name) in preset_format_types {
                            label {
                                r#for: "outputFormat{option}",
                                input {
                                    r#type: "radio",
                                    id: "outputFormat{option}",
                                    name: "outputFormat",
                                    value: "{option}",
                                    oninput: move |e| {
                                        app.with_mut(|a| a.settings.overrides.output_format = Some(e.value.parse().unwrap()));
                                    }
                                }
                                option_name
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
                        for (option, option_name) in preset_mipmap_types {
                            label {
                                r#for: "mipmaps{option}",
                                input {
                                    r#type: "radio",
                                    id: "mipmaps{option}",
                                    name: "mipmaps",
                                    value: "{option}",
                                    oninput: move |e| {
                                        app.with_mut(|a| a.settings.overrides.mipmaps = Some(e.value.parse().unwrap()));
                                    }
                                }
                                option_name
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
                                        app.with_mut(|a| a.settings.overrides.output_quality = Some(e.value.parse().unwrap()));
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
                                    app.with_mut(|a| a.settings.overrides.output_quality = None);
                                }
                            }
                            "Custom..."
                        }
                    }
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
                                match app.with(|a| a.settings.overrides.output_file_type) {
                                    Some(ty) => rsx! { "{ty}" },
                                    None => rsx!{
                                        select {
                                            onchange: move |e| {
                                                app.with_mut(|a| a.settings.file_settings[i].output_file_type = e.value.parse().unwrap());
                                            },
                                            for variant in ImageFileType::iter() {
                                                option { value: "{item.output_file_type}", "{variant}" }
                                            }
                                        }
                                    }
                                }
                            }
                            th {
                                match app.with(|a| a.settings.overrides.output_format) {
                                    Some(ty) => rsx! { "{ty}" },
                                    None => rsx!{
                                        select {
                                            onchange: move |e| {
                                                app.with_mut(|a| a.settings.file_settings[i].output_format = e.value.parse().unwrap());
                                            },
                                            for variant in ImageFormat::iter() {
                                                option { value: "{item.output_format}", "{variant}" }
                                            }
                                        }
                                    }
                                }
                            }
                            th {
                                match app.with(|a| a.settings.overrides.output_quality) {
                                    Some(ty) => rsx! { "{ty}" },
                                    None => rsx!{
                                        select {
                                            onchange: move |e| {
                                                app.with_mut(|a| a.settings.file_settings[i].output_quality = e.value.parse().unwrap());
                                            },
                                            for variant in Quality::iter() {
                                                option { value: "{item.output_quality}", "{variant}" }
                                            }
                                        }
                                    }
                                }
                            }
                            th {
                                match app.with(|a| a.settings.overrides.mipmaps) {
                                    Some(ty) => rsx! { "{ty}" },
                                    None => rsx!{
                                        select {
                                            onchange: move |e| {
                                                app.with_mut(|a| a.settings.file_settings[i].output_mipmaps = e.value.parse().unwrap());
                                            },
                                            for variant in Mipmaps::iter() {
                                                option { value: "{item.output_mipmaps}", "{variant}" }
                                            }
                                        }
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
        if app.read().settings.file_settings.is_empty() {
            rsx! {
                div {
                    class: "centered-text",
                    "Drag and drop image files onto the window or add files using File > Add Files..."
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
