use std::{error::Error, path::Path};

pub use bntx::BntxFile;
pub use nutexb::NutexbFile;

use image_dds::{dds_image_format, ddsfile::Dds, image::RgbaImage, ImageFormat, Surface};

pub enum ImageFile {
    Image(RgbaImage),
    Dds(Dds),
    Nutexb(NutexbFile),
    Bntx(BntxFile),
}

impl ImageFile {
    pub fn read<P: AsRef<Path>>(input: P) -> Result<Self, Box<dyn Error>> {
        match input
            .as_ref()
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
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
            _ => Ok(ImageFile::Image(image_dds::image::open(input)?.to_rgba8())),
        }
    }

    pub fn image_format(&self) -> ImageFormat {
        // TODO: Avoid unwrap?
        match self {
            ImageFile::Image(_) => ImageFormat::R8G8B8A8Unorm,
            ImageFile::Dds(dds) => image_dds::dds_image_format(dds).unwrap(),
            ImageFile::Nutexb(nutexb) => nutexb_image_format(nutexb),
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
        bntx::SurfaceFormat::R8Unorm => ImageFormat::R8Unorm,
        bntx::SurfaceFormat::R8G8B8A8Unorm => ImageFormat::R8G8B8A8Unorm,
        bntx::SurfaceFormat::R8G8B8A8Srgb => ImageFormat::R8G8B8A8Srgb,
        bntx::SurfaceFormat::B8G8R8A8Unorm => ImageFormat::B8G8R8A8Unorm,
        bntx::SurfaceFormat::B8G8R8A8Srgb => ImageFormat::B8G8R8A8Srgb,
        bntx::SurfaceFormat::BC1Unorm => ImageFormat::BC1Unorm,
        bntx::SurfaceFormat::BC1Srgb => ImageFormat::BC1Srgb,
        bntx::SurfaceFormat::BC2Unorm => ImageFormat::BC2Unorm,
        bntx::SurfaceFormat::BC2Srgb => ImageFormat::BC2Srgb,
        bntx::SurfaceFormat::BC3Unorm => ImageFormat::BC3Unorm,
        bntx::SurfaceFormat::BC3Srgb => ImageFormat::BC3Srgb,
        bntx::SurfaceFormat::BC4Unorm => ImageFormat::BC4Unorm,
        bntx::SurfaceFormat::BC4Snorm => ImageFormat::BC4Snorm,
        bntx::SurfaceFormat::BC5Unorm => ImageFormat::BC5Unorm,
        bntx::SurfaceFormat::BC5Snorm => ImageFormat::BC5Snorm,
        bntx::SurfaceFormat::BC6Ufloat => ImageFormat::BC6Ufloat,
        bntx::SurfaceFormat::BC6Sfloat => ImageFormat::BC6Sfloat,
        bntx::SurfaceFormat::BC7Unorm => ImageFormat::BC7Unorm,
        bntx::SurfaceFormat::BC7Srgb => ImageFormat::BC7Srgb,
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

pub fn convert_to_image(input_image: &ImageFile, output: &Path) -> Result<(), Box<dyn Error>> {
    // TODO: EXR support for BC6H?

    match input_image {
        ImageFile::Image(image) => image.save(output)?,
        ImageFile::Dds(dds) => {
            let image = image_dds::image_from_dds(dds, 0)?;
            image.save(output)?;
        }
        ImageFile::Nutexb(nutexb) => {
            // Use DDS as an intermediate format to handle swizzling.
            let dds = nutexb.to_dds()?;
            let image = image_dds::image_from_dds(&dds, 0)?;
            image.save(output)?;
        }
        ImageFile::Bntx(bntx) => {
            // Use DDS as an intermediate format to handle swizzling.
            let dds = bntx::dds::create_dds(bntx)?;
            let image = image_dds::image_from_dds(&dds, 0)?;
            image.save(output)?;
        }
    }
    Ok(())
}

pub fn convert_to_nutexb(
    input_image: &ImageFile,
    output: &Path,
    image_format: image_dds::ImageFormat,
    quality: image_dds::Quality,
    mipmaps: image_dds::Mipmaps,
) -> Result<(), Box<dyn Error>> {
    // Nutexb files use the file name as the internal name.
    let name = output
        .with_extension("")
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Use image_dds to encode to a new format if necessary.
    match input_image {
        ImageFile::Image(image) => {
            let dds = image_dds::dds_from_image(image, image_format, quality, mipmaps)?;
            let nutexb = NutexbFile::from_dds(&dds, name)?;
            nutexb.write_to_file(output)?;
        }
        ImageFile::Dds(dds) => {
            let new_dds = encode_dds(dds, image_format, quality, mipmaps)?;
            let nutexb = NutexbFile::from_dds(&new_dds, name)?;
            nutexb.write_to_file(output)?;
        }
        ImageFile::Nutexb(nutexb) => {
            let dds = nutexb.to_dds()?;
            let new_dds = encode_dds(&dds, image_format, quality, mipmaps)?;
            let new_nutexb = NutexbFile::from_dds(&new_dds, nutexb.footer.string.to_string())?;
            new_nutexb.write_to_file(output)?;
        }
        ImageFile::Bntx(bntx) => {
            let dds = bntx::dds::create_dds(bntx)?;
            let new_dds = encode_dds(&dds, image_format, quality, mipmaps)?;
            let nutexb = NutexbFile::from_dds(&new_dds, name)?;
            nutexb.write_to_file(output)?;
        }
    }
    Ok(())
}

pub fn convert_to_bntx(
    input_image: &ImageFile,
    output: &Path,
    image_format: image_dds::ImageFormat,
    quality: image_dds::Quality,
    mipmaps: image_dds::Mipmaps,
) -> Result<(), Box<dyn Error>> {
    // Nutexb files use the file name as the internal name.
    let name = output
        .with_extension("")
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    match input_image {
        ImageFile::Image(image) => {
            let dds = image_dds::dds_from_image(image, image_format, quality, mipmaps)?;
            let bntx = bntx::dds::create_bntx(&name, &dds)?;
            bntx.write_to_file(output)?;
        }
        ImageFile::Dds(dds) => {
            let new_dds = encode_dds(dds, image_format, quality, mipmaps)?;
            let bntx = bntx::dds::create_bntx(&name, &new_dds)?;
            bntx.write_to_file(output)?;
        }
        ImageFile::Nutexb(nutexb) => {
            let dds = nutexb.to_dds()?;
            let new_dds = encode_dds(&dds, image_format, quality, mipmaps)?;
            let bntx = bntx::dds::create_bntx(&name, &new_dds)?;
            bntx.write_to_file(output)?;
        }
        ImageFile::Bntx(bntx) => {
            let dds = bntx::dds::create_dds(bntx)?;
            let new_dds = encode_dds(&dds, image_format, quality, mipmaps)?;
            let new_bntx = bntx::dds::create_bntx(&name, &new_dds)?;
            new_bntx.write_to_file(output)?;
        }
    }
    Ok(())
}

pub fn convert_to_dds(
    input_image: &ImageFile,
    output: &Path,
    image_format: image_dds::ImageFormat,
    quality: image_dds::Quality,
    mipmaps: image_dds::Mipmaps,
) -> Result<(), Box<dyn Error>> {
    match input_image {
        ImageFile::Image(image) => {
            let dds = image_dds::dds_from_image(image, image_format, quality, mipmaps)?;
            write_dds(output, &dds)?;
        }
        ImageFile::Dds(dds) => {
            let new_dds = encode_dds(dds, image_format, quality, mipmaps)?;
            write_dds(output, &new_dds)?;
        }
        ImageFile::Nutexb(nutexb) => {
            let dds = nutexb.to_dds()?;
            let new_dds = encode_dds(&dds, image_format, quality, mipmaps)?;
            write_dds(output, &new_dds)?;
        }
        ImageFile::Bntx(bntx) => {
            let dds = bntx::dds::create_dds(bntx)?;
            let new_dds = encode_dds(&dds, image_format, quality, mipmaps)?;
            write_dds(output, &new_dds)?;
        }
    };

    Ok(())
}

fn encode_dds(
    dds: &Dds,
    image_format: ImageFormat,
    quality: image_dds::Quality,
    mipmaps: image_dds::Mipmaps,
) -> Result<Dds, Box<dyn Error>> {
    if matches!(dds_image_format(dds), Ok(format) if format == image_format) {
        // Avoid lossy conversions if the format doesn't change.
        // TODO: Handle different mipmap counts.
        // Dds does not implement Clone, so we need to get creative.
        let mut writer = std::io::Cursor::new(Vec::new());
        dds.write(&mut writer)?;
        let mut reader = std::io::Cursor::new(writer.into_inner());
        Dds::read(&mut reader).map_err(Into::into)
    } else {
        // Decode and encode to the desired format.
        // This also handles adjusting the number of mipmaps.
        Surface::from_dds(dds)?
            .decode_rgba8()?
            .encode(image_format, quality, mipmaps)?
            .to_dds()
            .map_err(Into::into)
    }
}

fn write_dds(output: &Path, dds: &Dds) -> Result<(), Box<dyn Error>> {
    let mut writer = std::io::BufWriter::new(std::fs::File::create(output)?);
    dds.write(&mut writer)?;
    Ok(())
}
