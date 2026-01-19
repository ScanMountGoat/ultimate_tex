use std::path::Path;

pub use bntx::Bntx;
pub use nutexb::NutexbFile;

use image_dds::{ImageFormat, Surface, dds_image_format, ddsfile::Dds, image::RgbaImage};

pub enum ImageFile {
    Image(RgbaImage),
    Dds(Dds),
    Nutexb(NutexbFile),
    Bntx(Bntx),
}

impl ImageFile {
    pub fn from_file<P: AsRef<Path>>(input: P) -> anyhow::Result<Self> {
        match input
            .as_ref()
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase()
            .as_str()
        {
            "nutexb" => {
                let mut nutexb = NutexbFile::read_from_file(input)?;
                // Some nutexbs saved with older tools have too many mipmaps specified.
                // The image data is usually still valid.
                // Attempt to fix the mipmap count here to keep this as an error in image_dds.
                fix_mipmap_count(&mut nutexb);
                Ok(ImageFile::Nutexb(nutexb))
            }
            "bntx" => Ok(ImageFile::Bntx(Bntx::from_file(input)?)),
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
            ImageFile::Image(_) => ImageFormat::Rgba8Unorm,
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

    pub fn to_image(&self) -> anyhow::Result<RgbaImage> {
        // TODO: EXR support for BC6H?
        match self {
            ImageFile::Image(image) => Ok(image.clone()),
            ImageFile::Dds(dds) => image_dds::image_from_dds(dds, 0).map_err(Into::into),
            ImageFile::Nutexb(nutexb) => {
                // Use DDS as an intermediate format to handle swizzling.
                let dds = nutexb.to_dds()?;
                image_dds::image_from_dds(&dds, 0).map_err(Into::into)
            }
            ImageFile::Bntx(bntx) => {
                // Use DDS as an intermediate format to handle swizzling.
                let dds = bntx.to_dds()?;
                image_dds::image_from_dds(&dds, 0).map_err(Into::into)
            }
        }
    }

    pub fn save_image(&self, output: &Path) -> anyhow::Result<()> {
        self.to_image()?.save(output).map_err(Into::into)
    }

    pub fn save_nutexb(
        &self,
        output: &Path,
        image_format: image_dds::ImageFormat,
        quality: image_dds::Quality,
        mipmaps: image_dds::Mipmaps,
    ) -> anyhow::Result<()> {
        // Nutexb files use the file name as the internal name.
        let name = output
            .with_extension("")
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Use image_dds to encode to a new format if necessary.
        match self {
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
                let dds = bntx.to_dds()?;
                let new_dds = encode_dds(&dds, image_format, quality, mipmaps)?;
                let nutexb = NutexbFile::from_dds(&new_dds, name)?;
                nutexb.write_to_file(output)?;
            }
        }
        Ok(())
    }

    pub fn save_bntx(
        &self,
        output: &Path,
        image_format: image_dds::ImageFormat,
        quality: image_dds::Quality,
        mipmaps: image_dds::Mipmaps,
    ) -> anyhow::Result<()> {
        // Nutexb files use the file name as the internal name.
        let name = output
            .with_extension("")
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        match self {
            ImageFile::Image(image) => {
                let dds = image_dds::dds_from_image(image, image_format, quality, mipmaps)?;
                let bntx = Bntx::from_dds(&dds, &name)?;
                bntx.save(output)?;
            }
            ImageFile::Dds(dds) => {
                let new_dds = encode_dds(dds, image_format, quality, mipmaps)?;
                let bntx = Bntx::from_dds(&new_dds, &name)?;
                bntx.save(output)?;
            }
            ImageFile::Nutexb(nutexb) => {
                let dds = nutexb.to_dds()?;
                let new_dds = encode_dds(&dds, image_format, quality, mipmaps)?;
                let bntx = Bntx::from_dds(&new_dds, &name)?;
                bntx.save(output)?;
            }
            ImageFile::Bntx(bntx) => {
                let dds = bntx.to_dds()?;
                let new_dds = encode_dds(&dds, image_format, quality, mipmaps)?;
                let bntx = Bntx::from_dds(&new_dds, &name)?;
                bntx.save(output)?;
            }
        }
        Ok(())
    }

    pub fn save_dds(
        &self,
        output: &Path,
        image_format: image_dds::ImageFormat,
        quality: image_dds::Quality,
        mipmaps: image_dds::Mipmaps,
    ) -> anyhow::Result<()> {
        match self {
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
                let dds = bntx.to_dds()?;
                let new_dds = encode_dds(&dds, image_format, quality, mipmaps)?;
                write_dds(output, &new_dds)?;
            }
        };

        Ok(())
    }
}

fn fix_mipmap_count(nutexb: &mut NutexbFile) {
    let max_mipmaps = nutexb
        .footer
        .width
        .max(nutexb.footer.height)
        .max(nutexb.footer.depth)
        .ilog2();
    nutexb.footer.mipmap_count = nutexb.footer.mipmap_count.min(max_mipmaps);
}

fn bntx_image_format(bntx: &Bntx) -> ImageFormat {
    bntx.image_format().try_into().unwrap()
}

fn nutexb_image_format(nutexb: &NutexbFile) -> ImageFormat {
    match nutexb.footer.image_format {
        nutexb::NutexbFormat::R8Unorm => ImageFormat::R8Unorm,
        nutexb::NutexbFormat::R8G8B8A8Unorm => ImageFormat::Rgba8Unorm,
        nutexb::NutexbFormat::R8G8B8A8Srgb => ImageFormat::Rgba8UnormSrgb,
        nutexb::NutexbFormat::R32G32B32A32Float => ImageFormat::Rgba32Float,
        nutexb::NutexbFormat::B8G8R8A8Unorm => ImageFormat::Bgra8Unorm,
        nutexb::NutexbFormat::B8G8R8A8Srgb => ImageFormat::Bgra8UnormSrgb,
        nutexb::NutexbFormat::BC1Unorm => ImageFormat::BC1RgbaUnorm,
        nutexb::NutexbFormat::BC1Srgb => ImageFormat::BC1RgbaUnormSrgb,
        nutexb::NutexbFormat::BC2Unorm => ImageFormat::BC2RgbaUnorm,
        nutexb::NutexbFormat::BC2Srgb => ImageFormat::BC2RgbaUnormSrgb,
        nutexb::NutexbFormat::BC3Unorm => ImageFormat::BC3RgbaUnorm,
        nutexb::NutexbFormat::BC3Srgb => ImageFormat::BC3RgbaUnormSrgb,
        nutexb::NutexbFormat::BC4Unorm => ImageFormat::BC4RUnorm,
        nutexb::NutexbFormat::BC4Snorm => ImageFormat::BC4RSnorm,
        nutexb::NutexbFormat::BC5Unorm => ImageFormat::BC5RgUnorm,
        nutexb::NutexbFormat::BC5Snorm => ImageFormat::BC5RgSnorm,
        nutexb::NutexbFormat::BC6Ufloat => ImageFormat::BC6hRgbUfloat,
        nutexb::NutexbFormat::BC6Sfloat => ImageFormat::BC6hRgbSfloat,
        nutexb::NutexbFormat::BC7Unorm => ImageFormat::BC7RgbaUnorm,
        nutexb::NutexbFormat::BC7Srgb => ImageFormat::BC7RgbaUnormSrgb,
    }
}

fn encode_dds(
    dds: &Dds,
    image_format: ImageFormat,
    quality: image_dds::Quality,
    mipmaps: image_dds::Mipmaps,
) -> anyhow::Result<Dds> {
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

fn write_dds(output: &Path, dds: &Dds) -> anyhow::Result<()> {
    let mut writer = std::io::BufWriter::new(std::fs::File::create(output)?);
    dds.write(&mut writer)?;
    Ok(())
}
