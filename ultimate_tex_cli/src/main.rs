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

    let format = args
        .format
        .map(|s| image_dds::ImageFormat::from_str(&s).unwrap())
        .unwrap_or(image_dds::ImageFormat::BC7Unorm);

    let quality = image_dds::Quality::Fast;

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
        "nutexb" => input_image
            .save_nutexb(output, format, quality, mipmaps)
            .unwrap(),
        "bntx" => input_image
            .save_bntx(output, format, quality, mipmaps)
            .unwrap(),
        "dds" => input_image
            .save_dds(output, format, quality, mipmaps)
            .unwrap(),
        // Assume the other formats are image formats.
        _ => input_image.save_image(output).unwrap(),
    }
}
