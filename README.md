# ultimate_tex [![GitHub release (latest by date including pre-releases)](https://img.shields.io/github/v/release/ScanMountGoat/ssbh_editor?include_prereleases)](https://github.com/ScanMountGoat/ssbh_editor/releases/latest)
Texture conversion utilities for Smash Ultimate. Report any bugs or request new features in [issues](https://github.com/ScanMountGoat/ultimate_tex/issues).

## ultimate_tex_app
A desktop application for batch converting texture files. Download the program in [releases](https://github.com/ScanMountGoat/ultimate_tex/releases).

![image](https://user-images.githubusercontent.com/23301691/216787389-93b1484e-1560-4f45-8e9a-b7b60b19cdf4.png)

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
Don't forget the `--release` since debug builds in Rust will run slowly. 
The image_dds crate compiles C/C++ code for the encoders and decoders and requires C/C++ tooling installed.

## Credits
- [nutexb](https://github.com/jam1garner/nutexb)
- [bntx](https://github.com/jam1garner/bntx)
- [image_dds](https://github.com/ScanMountGoat/image_dds)
- [image](https://github.com/image-rs/image)
- [ddsfile](https://github.com/SiegeEngine/ddsfile)
