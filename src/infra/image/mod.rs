use image::{ImageFormat, ImageReader};
use std::io::Cursor;

pub struct ProcessedImage {
    pub data: Vec<u8>,
    pub content_type: String,
}

pub struct ImageProcessor;

impl ImageProcessor {
    pub fn process_avatar(
        image_data: &[u8],
        max_size: u32,
    ) -> Result<ProcessedImage, anyhow::Error> {
        Self::process_to_webp(image_data, max_size, max_size)
    }

    pub fn process_chat_image(
        image_data: &[u8],
        max_size: u32,
    ) -> Result<ProcessedImage, anyhow::Error> {
        Self::process_to_webp(image_data, max_size, max_size)
    }

    fn process_to_webp(
        image_data: &[u8],
        max_width: u32,
        max_height: u32,
    ) -> Result<ProcessedImage, anyhow::Error> {
        let img = ImageReader::new(Cursor::new(image_data))
            .with_guessed_format()?
            .decode()?;

        let resized = img.resize(max_width, max_height, image::imageops::FilterType::Lanczos3);

        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);

        resized.write_to(&mut cursor, ImageFormat::WebP)?;

        Ok(ProcessedImage {
            data: buffer,
            content_type: "image/webp".to_string(),
        })
    }

    pub fn is_gif(data: &[u8]) -> bool {
        ImageReader::new(Cursor::new(data))
            .with_guessed_format()
            .ok()
            .map(|r| r.format() == Some(ImageFormat::Gif))
            .unwrap_or(false)
    }

    pub fn process_gif(image_data: &[u8], max_size: u32) -> Result<ProcessedImage, anyhow::Error> {
        let img = ImageReader::new(Cursor::new(image_data))
            .with_guessed_format()?
            .decode()?;

        let resized = img.resize(max_size, max_size, image::imageops::FilterType::Lanczos3);

        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);

        resized.write_to(&mut cursor, ImageFormat::Gif)?;

        Ok(ProcessedImage {
            data: buffer,
            content_type: "image/gif".to_string(),
        })
    }
}
