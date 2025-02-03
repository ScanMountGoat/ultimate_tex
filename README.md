# ultimate_tex [![GitHub release (latest by date including pre-releases)](https://img.shields.io/github/v/release/ScanMountGoat/ultimate_tex?include_prereleases)](https://github.com/ScanMountGoat/ultimate_tex/releases/latest)
Texture conversion utilities for Smash Ultimate. Report any bugs or request new features in [issues](https://github.com/ScanMountGoat/ultimate_tex/issues).

## ultimate_tex
![image](https://github.com/user-attachments/assets/791c08c0-f5b9-47f2-a47d-8686d403bd85)

A desktop application for batch converting texture files. Download the program in [releases](https://github.com/ScanMountGoat/ultimate_tex/releases).

Drag files onto the application window or add them with File > Add File(s), select the export settings, select the export folder, and click the export button to convert. See the [wiki](https://github.com/ScanMountGoat/ultimate_tex/wiki) for detailed usage instructions. 

## ultimate_tex_cli
A commandline program for converting a single texture file.

### Examples
DDS:  
`ultimate_tex_cli cat.jpeg cat.dds --format BC1RgbaUnormSrgb`  
`ultimate_tex_cli cat.dds cat.tiff`  

Nutexb:  
`ultimate_tex_cli def_mario_001_col.nutexb img.dds`  
`ultimate_tex_cli img.dds def_mario_001_col.nutexb --format BC7RgbaUnormSrgb`  

Bntx:  
`ultimate_tex_cli chara_0_captain_01.bntx img.png`  
`ultimate_tex_cli img.png chara_0_captain_01.bntx --format BC7RgbaUnorm --no-mipmaps`  

## ultimate_tex_lib
A library for conversion functionality shared between the GUI and CLI programs.

## Building
With a newer version of the Rust toolchain installed, run `cargo build --release`. Don't forget the `--release` since debug builds in Rust will run slowly. Build and run the application with `cargo run --release -p ultimate_tex`. See the GitHub Actions scripts for installing the necessary Linux packages. image_dds uses precompiled kernels for encoding BCN image formats, so not all platforms and architectures are supported. 

## Credits
- [nutexb](https://github.com/jam1garner/nutexb)
- [bntx](https://github.com/ScanMountGoat/bntx)
- [image_dds](https://github.com/ScanMountGoat/image_dds)
- [image](https://crates.io/crates/image)
- [ddsfile](https://crates.io/crates/ddsfile)
