use std::{path::Path, str::FromStr};

use clap::Parser;
use image_dds::Mipmaps;
use ultimate_tex::ImageFile;

#[derive(Parser, Debug)]
#[command(author, version, about = "Smash Ultimate texture converter", long_about = None)]
struct Args {
    #[arg(help = "The input image file to convert")]
    input: String,

    #[arg(help = "The output converted image file")]
    output: String,

    // TODO: make this a value enum to show possible image formats?
    #[arg(
        short = 'f',
        long = "format",
        help = "The output image format for files supporting compression"
    )]
    format: Option<String>,

    #[arg(
        long = "no-mipmaps",
        help = "Disable mipmap generation and only include the base mip level"
    )]
    no_mipmaps: bool,
}

fn main() {
    let args = Args::parse();
    let input = Path::new(&args.input);
    let output = Path::new(&args.output);

    let input_image = ImageFile::read(input).unwrap();

    let image_format = args
        .format
        .map(|s| image_dds::ImageFormat::from_str(&s).unwrap())
        .unwrap_or(image_dds::ImageFormat::BC7Unorm);

    let mipmaps = if args.no_mipmaps {
        Mipmaps::Disabled
    } else {
        Mipmaps::GeneratedAutomatic
    };

    match output
        .extension()
        .unwrap()
        .to_str()
        .unwrap()
        .to_lowercase()
        .as_str()
    {
        "nutexb" => ultimate_tex::convert_to_nutexb(&input_image, output, image_format, mipmaps),
        "bntx" => ultimate_tex::convert_to_bntx(&input_image, output, image_format, mipmaps),
        "dds" => ultimate_tex::convert_to_dds(&input_image, output, image_format, mipmaps),
        // Assume the other formats are image formats.
        _ => ultimate_tex::convert_to_image(&input_image, output),
    }
}
