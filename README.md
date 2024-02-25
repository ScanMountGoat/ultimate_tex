# ultimate_tex [![GitHub release (latest by date including pre-releases)](https://img.shields.io/github/v/release/ScanMountGoat/ultimate_tex?include_prereleases)](https://github.com/ScanMountGoat/ultimate_tex/releases/latest)
Texture conversion utilities for Smash Ultimate. Report any bugs or request new features in [issues](https://github.com/ScanMountGoat/ultimate_tex/issues).

## ultimate_tex_app
![image](https://github.com/ScanMountGoat/ultimate_tex/assets/23301691/0c38e6de-6800-42c3-b250-efcf4d3cb490)

A desktop application for batch converting texture files. Download the program in [releases](https://github.com/ScanMountGoat/ultimate_tex/releases).

Drag files onto the application window or add them with File > Add File(s), select the export settings, select the export folder, and click the export button to convert. See the [wiki](https://github.com/ScanMountGoat/ultimate_tex/wiki) for detailed usage instructions. 

## ultimate_tex_cli
A commandline program for converting a single texture file.

### Examples
DDS:  
`ultimate_tex_cli cat.jpeg cat.dds --format BC1Srgb`  
`ultimate_tex_cli cat.dds cat.tiff`  

Nutexb:  
`ultimate_tex_cli def_mario_001_col.nutexb img.dds`  
`ultimate_tex_cli img.dds def_mario_001_col.nutexb --format BC7Srgb`  

Bntx:  
`ultimate_tex_cli chara_0_captain_01.bntx img.png`  
`ultimate_tex_cli img.png chara_0_captain_01.bntx --format BC7Unorm --no-mipmaps`  

## ultimate_tex
A library for conversion functionality shared between the GUI and CLI programs.

## Building
With a newer version of the Rust toolchain installed, run `cargo build --release`. Don't forget the `--release` since debug builds in Rust will run slowly. The image_dds crate compiles C/C++ code for the encoders and decoders and requires C/C++ tooling installed. image_dds uses precompiled kernels for some image encoding operations, so not all platforms and architectures are supported. Build and run the application with `cargo run --release -p ultimate_tex_app`. See the GitHub Actions scripts for installing the necessary Linux packages.

## Credits
- [nutexb](https://github.com/jam1garner/nutexb)
- [bntx](https://github.com/jam1garner/bntx)
- [image_dds](https://github.com/ScanMountGoat/image_dds)
- [image](https://github.com/image-rs/image)
- [ddsfile](https://github.com/SiegeEngine/ddsfile)
