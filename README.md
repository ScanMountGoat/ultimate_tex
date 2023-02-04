# ultimate_tex
Texture conversion utilities for Smash Ultimate.

## ultimate_tex_app
A desktop application for batch converting texture files.

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
Don't forget the `--release` since debug builds in Rust will perform poorly!

## Credits
- [nutexb](https://github.com/jam1garner/nutexb)
- [bntx](https://github.com/jam1garner/bntx)
- [image_dds](https://github.com/ScanMountGoat/image_dds)
- [image](https://github.com/image-rs/image)
- [ddsfile](https://github.com/SiegeEngine/ddsfile)
