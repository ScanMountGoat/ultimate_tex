# ultimate_tex [![GitHub release (latest by date including pre-releases)](https://img.shields.io/github/v/release/ScanMountGoat/ultimate_tex?include_prereleases)](https://github.com/ScanMountGoat/ultimate_tex/releases/latest)
Texture conversion utilities for Smash Ultimate. Report any bugs or request new features in [issues](https://github.com/ScanMountGoat/ultimate_tex/issues).

## ultimate_tex_app
![image](https://user-images.githubusercontent.com/23301691/227569380-849ef33c-06e0-4637-9178-a1efcfea5fff.png)

A desktop application for batch converting texture files. Download the program in [releases](https://github.com/ScanMountGoat/ultimate_tex/releases).

Drag files onto the application window or add them with File > Add File(s), select the export settings, select the export folder, and click the export button to convert. See the [wiki](https://github.com/ScanMountGoat/ultimate_tex/wiki) for detailed usage instructions. 

## ultimate_tex_cli
A commandline program for converting a single texture file.

### Examples
DDS:  
`cargo run -p ultimate_tex_cli --release -- cat.jpeg cat.dds --format BC1Srgb`  
`cargo run -p ultimate_tex_cli --release -- cat.dds cat.tiff`  

Nutexb:  
`cargo run -p ultimate_tex_cli --release -- def_mario_001_col.nutexb img.dds`  
`cargo run -p ultimate_tex_cli --release -- img.dds def_mario_001_col.nutexb --format BC7Srgb`  

Bntx:  
`cargo run -p ultimate_tex_cli --release -- chara_0_captain_01.bntx img.png`  
`cargo run -p ultimate_tex_cli --release -- img.png chara_0_captain_01.bntx --format BC7Unorm --no-mipmaps`  

## ultimate_tex
A library for conversion functionality shared between the GUI and CLI programs.

## Building
With a newer version of the Rust toolchain installed, run `cargo build --release`. 
Don't forget the `--release` since debug builds in Rust will run slowly. The image_dds crate compiles C/C++ code for the encoders and decoders and requires C/C++ tooling installed. image_dds uses precompiled kernels for some image encoding operations, so not all platforms and architectures are supported.

The ultimate_tex_tauri application requires additional tools to build and run. See the [getting started guide](https://tauri.app/v1/guides/getting-started/prerequisites) for details. The guide also covers installing necessary packages on Linux. The frontend uses Javascript and requires installing a newer version of [npm](https://docs.npmjs.com/downloading-and-installing-node-js-and-npm). Run `npm install` once from the `ultimate_tex_tauri` directory to install the necessary Javascript packages. The easiest way to run the app locally is to install the tauri CLI tools with `cargo install tauri-cli` and then run `cargo tauri dev` also from the `ultimate_tex_tauri` directory.

## Credits
- [nutexb](https://github.com/jam1garner/nutexb)
- [bntx](https://github.com/jam1garner/bntx)
- [image_dds](https://github.com/ScanMountGoat/image_dds)
- [image](https://github.com/image-rs/image)
- [ddsfile](https://github.com/SiegeEngine/ddsfile)
