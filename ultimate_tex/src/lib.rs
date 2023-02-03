use std::{error::Error, path::Path};

pub use bntx::BntxFile;
pub use ddsfile::Dds;
pub use image::RgbaImage;
pub use nutexb::NutexbFile;

use image_dds::ImageFormat;

pub enum ImageFile {
    Image(RgbaImage),
    Dds(Dds),
    Nutexb(NutexbFile),
    Bntx(BntxFile),
}

// TODO: Add a save method?
// TODO: Add methods for dimensions, format, etc?
impl ImageFile {
    pub fn read<P: AsRef<Path>>(input: P) -> Result<Self, Box<dyn Error>> {
        match input
            .as_ref()
            .extension()
            .unwrap()
            .to_str()
            .unwrap()
            .to_lowercase()
            .as_str()
        {
            "nutexb" => Ok(ImageFile::Nutexb(NutexbFile::read_from_file(input)?)),
            "bntx" => Ok(ImageFile::Bntx(BntxFile::from_file(input)?)),
            "dds" => {
                let mut reader = std::io::BufReader::new(std::fs::File::open(input)?);
                Ok(ImageFile::Dds(Dds::read(&mut reader)?))
            }
            // Assume the other formats are image formats.
            _ => Ok(ImageFile::Image(image::open(input)?.to_rgba8())),
        }
    }

    pub fn image_format(&self) -> ImageFormat {
        // TODO: How to handle this?
        match self {
            ImageFile::Image(_) => ImageFormat::R8G8B8A8Unorm, // TODO: Should this be srgb?
            ImageFile::Dds(dds) => image_dds::dds_image_format(dds).unwrap(), // TODO: make this part of image_dds
            ImageFile::Nutexb(nutexb) => nutexb_image_format(nutexb), // TODO: impl From<NutexbFormat>?
            ImageFile::Bntx(bntx) => bntx_image_format(bntx),
        }
    }

    pub fn dimensions(&self) -> (u32, u32, u32) {
        match self {
            ImageFile::Image(image) => (image.width(), image.height(), 1),
            ImageFile::Dds(dds) => (dds.get_width(), dds.get_height(), dds.get_depth()),
            ImageFile::Nutexb(nutexb) => (
                nutexb.footer.width,
                nutexb.footer.height,
                nutexb.footer.depth,
            ),
            ImageFile::Bntx(bntx) => (bntx.width(), bntx.height(), bntx.depth()),
        }
    }
}

fn bntx_image_format(bntx: &BntxFile) -> ImageFormat {
    match bntx.image_format() {
        bntx::SurfaceFormat::R8G8B8A8Srgb => ImageFormat::R8G8B8A8Srgb,
        bntx::SurfaceFormat::BC7Unorm => ImageFormat::BC7Unorm,
    }
}

fn nutexb_image_format(nutexb: &NutexbFile) -> ImageFormat {
    match nutexb.footer.image_format {
        nutexb::NutexbFormat::R8Unorm => ImageFormat::R8Unorm,
        nutexb::NutexbFormat::R8G8B8A8Unorm => ImageFormat::R8G8B8A8Unorm,
        nutexb::NutexbFormat::R8G8B8A8Srgb => ImageFormat::R8G8B8A8Srgb,
        nutexb::NutexbFormat::R32G32B32A32Float => ImageFormat::R32G32B32A32Float,
        nutexb::NutexbFormat::B8G8R8A8Unorm => ImageFormat::B8G8R8A8Unorm,
        nutexb::NutexbFormat::B8G8R8A8Srgb => ImageFormat::B8G8R8A8Srgb,
        nutexb::NutexbFormat::BC1Unorm => ImageFormat::BC1Unorm,
        nutexb::NutexbFormat::BC1Srgb => ImageFormat::BC1Srgb,
        nutexb::NutexbFormat::BC2Unorm => ImageFormat::BC2Unorm,
        nutexb::NutexbFormat::BC2Srgb => ImageFormat::BC2Srgb,
        nutexb::NutexbFormat::BC3Unorm => ImageFormat::BC3Unorm,
        nutexb::NutexbFormat::BC3Srgb => ImageFormat::BC3Srgb,
        nutexb::NutexbFormat::BC4Unorm => ImageFormat::BC4Unorm,
        nutexb::NutexbFormat::BC4Snorm => ImageFormat::BC4Snorm,
        nutexb::NutexbFormat::BC5Unorm => ImageFormat::BC5Unorm,
        nutexb::NutexbFormat::BC5Snorm => ImageFormat::BC5Snorm,
        nutexb::NutexbFormat::BC6Ufloat => ImageFormat::BC6Ufloat,
        nutexb::NutexbFormat::BC6Sfloat => ImageFormat::BC6Sfloat,
        nutexb::NutexbFormat::BC7Unorm => ImageFormat::BC7Unorm,
        nutexb::NutexbFormat::BC7Srgb => ImageFormat::BC7Srgb,
    }
}

pub fn convert_to_image(input_image: &ImageFile, output: &Path) {
    // TODO: EXR support for BC6H?

    match input_image {
        ImageFile::Image(image) => image.save(output).unwrap(),
        ImageFile::Dds(dds) => {
            let image = image_dds::image_from_dds(dds, 0).unwrap();
            image.save(output).unwrap();
        }
        ImageFile::Nutexb(nutexb) => {
            // Use DDS as an intermediate format to handle swizzling.
            let dds = nutexb::create_dds(nutexb).unwrap();
            let image = image_dds::image_from_dds(&dds, 0).unwrap();
            image.save(output).unwrap();
        }
        ImageFile::Bntx(bntx) => {
            // Use DDS as an intermediate format to handle swizzling.
            let dds = bntx::dds::create_dds(bntx).unwrap();
            let image = image_dds::image_from_dds(&dds, 0).unwrap();
            image.save(output).unwrap();
        }
    }
}

pub fn convert_to_nutexb(
    input_image: &ImageFile,
    output: &Path,
    image_format: image_dds::ImageFormat,
    mipmaps: image_dds::Mipmaps,
) {
    // Nutexb files use the file name as the internal name.
    let name = output
        .with_extension("")
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();

    match input_image {
        ImageFile::Image(image) => {
            // TODO: use args for quality and mipmaps
            let dds =
                image_dds::dds_from_image(image, image_format, image_dds::Quality::Fast, mipmaps)
                    .unwrap();
            let nutexb = NutexbFile::create(&dds, name).unwrap();
            nutexb.write_to_file(output).unwrap();
        }
        ImageFile::Dds(dds) => {
            // TODO: Decode and encode to new format?
            // TODO: Check the mipmaps option here.
            let nutexb = NutexbFile::create(dds, name).unwrap();
            nutexb.write_to_file(output).unwrap();
        }
        ImageFile::Nutexb(nutexb) => {
            // TODO: Decode and encode to new format?
            nutexb.write_to_file(output).unwrap();
        }
        ImageFile::Bntx(bntx) => {
            let dds = bntx::dds::create_dds(bntx).unwrap();
            let nutexb = NutexbFile::create(&dds, name).unwrap();
            nutexb.write_to_file(output).unwrap();
        }
    };
}

pub fn convert_to_bntx(
    input_image: &ImageFile,
    output: &Path,
    image_format: image_dds::ImageFormat,
    mipmaps: image_dds::Mipmaps,
) {
    // Nutexb files use the file name as the internal name.
    let name = output
        .with_extension("")
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();

    match input_image {
        ImageFile::Image(image) => {
            // TODO: use args for quality and mipmaps
            let dds =
                image_dds::dds_from_image(image, image_format, image_dds::Quality::Fast, mipmaps)
                    .unwrap();
            let bntx = bntx::dds::create_bntx(&name, &dds).unwrap();
            bntx.save(output).unwrap();
        }
        ImageFile::Dds(dds) => {
            // TODO: Decode and encode to new format?
            // TODO: Check the mipmaps option here.
            let bntx = bntx::dds::create_bntx(&name, dds).unwrap();
            bntx.save(output).unwrap();
        }
        ImageFile::Nutexb(nutexb) => {
            let dds = nutexb::create_dds(nutexb).unwrap();
            // TODO: Decode and encode to new format?
            let bntx = bntx::dds::create_bntx(&name, &dds).unwrap();
            bntx.save(output).unwrap();
        }
        ImageFile::Bntx(bntx) => {
            // TODO: Decode and encode to new format?
            bntx.save(output).unwrap();
        }
    };
}

pub fn convert_to_dds(
    input_image: &ImageFile,
    output: &Path,
    image_format: image_dds::ImageFormat,
    mipmaps: image_dds::Mipmaps,
) {
    match input_image {
        ImageFile::Image(image) => {
            // TODO: use args for format, quality, and mipmaps
            let dds =
                image_dds::dds_from_image(image, image_format, image_dds::Quality::Fast, mipmaps)
                    .unwrap();
            write_dds(output, &dds);
        }
        ImageFile::Dds(dds) => {
            // TODO: Decode and encode to new format?
            // Only encode again if the format is different?
            // TODO: Check mipmaps here.
            write_dds(output, dds);
        }
        ImageFile::Nutexb(nutexb) => {
            // TODO: Decode and encode to new format?
            // TODO: Check mipmaps here.
            let dds = nutexb::create_dds(nutexb).unwrap();
            write_dds(output, &dds);
        }
        ImageFile::Bntx(bntx) => {
            let dds = bntx::dds::create_dds(bntx).unwrap();
            write_dds(output, &dds);
        }
    };
}

fn write_dds(output: &Path, dds: &Dds) {
    let mut writer = std::io::BufWriter::new(std::fs::File::create(output).unwrap());
    dds.write(&mut writer).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2, 1 + 1);
    }
}
