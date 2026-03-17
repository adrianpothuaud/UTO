/// Image preprocessing for vision-based UI detection.
///
/// Handles loading, resizing, and normalizing screenshots for ONNX model inference.
use image::{DynamicImage, GenericImageView};

use crate::error::{UtoError, UtoResult};

/// Loads a PNG or JPEG screenshot into an image buffer.
pub fn load_screenshot(image_bytes: &[u8]) -> UtoResult<DynamicImage> {
    image::load_from_memory(image_bytes)
        .map_err(|e| UtoError::Internal(format!("Failed to load screenshot: {e}")))
}

/// Resizes an image to a target size while maintaining aspect ratio.
///
/// Padding is applied if necessary to maintain exact dimensions.
pub fn resize_for_inference(img: &DynamicImage, target_size: (u32, u32)) -> DynamicImage {
    let (target_width, target_height) = target_size;
    let (img_width, img_height) = img.dimensions();

    // Calculate scale to fit within target while maintaining aspect ratio
    let scale =
        (target_width as f32 / img_width as f32).min(target_height as f32 / img_height as f32);

    let new_width = (img_width as f32 * scale).round() as u32;
    let new_height = (img_height as f32 * scale).round() as u32;

    // Resize image
    let resized = img.resize_exact(new_width, new_height, image::imageops::FilterType::Lanczos3);

    // Convert to RGBA for padding operations
    let rgba = resized.to_rgba8();
    let mut padded = image::RgbaImage::new(target_width, target_height);

    // Fill with white background
    for pixel in padded.pixels_mut() {
        *pixel = image::Rgba([255, 255, 255, 255]);
    }

    // Center the resized image on the canvas
    let x_offset = (target_width.saturating_sub(new_width)) / 2;
    let y_offset = (target_height.saturating_sub(new_height)) / 2;

    image::imageops::overlay(&mut padded, &rgba, x_offset as i64, y_offset as i64);

    DynamicImage::ImageRgba8(padded)
}

/// Normalizes image pixels to the range [0, 1] for model input.
///
/// Returns a flat vector of normalized values suitable for ONNX tensor input.
pub fn normalize_pixels(image: &DynamicImage) -> Vec<f32> {
    let rgba = image.to_rgba8();
    let mut normalized = Vec::with_capacity(rgba.len() / 4 * 3); // RGB channels only

    for pixel in rgba.pixels() {
        // Skip alpha channel, use RGB
        normalized.push(pixel[0] as f32 / 255.0); // R
        normalized.push(pixel[1] as f32 / 255.0); // G
        normalized.push(pixel[2] as f32 / 255.0); // B
    }

    normalized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resize_for_inference() {
        // Create a simple 100x100 test image using RgbaImage
        let mut test_img = image::RgbaImage::new(100, 100);
        for pixel in test_img.pixels_mut() {
            *pixel = image::Rgba([255, 0, 0, 255]);
        }
        let test_image = DynamicImage::ImageRgba8(test_img);

        let resized = resize_for_inference(&test_image, (224, 224));
        let (w, h) = resized.dimensions();

        assert_eq!(w, 224);
        assert_eq!(h, 224);
    }

    #[test]
    fn test_normalize_pixels() {
        // Create a simple 2x2 test image
        let mut test_img = image::RgbaImage::new(2, 2);
        for pixel in test_img.pixels_mut() {
            *pixel = image::Rgba([128, 64, 192, 255]);
        }
        let test_image = DynamicImage::ImageRgba8(test_img);

        let normalized = normalize_pixels(&test_image);

        // 2x2 image = 4 pixels, 3 channels (RGB) each = 12 values
        assert_eq!(normalized.len(), 12);

        // Check first pixel (128, 64, 192) normalized
        assert!((normalized[0] - (128.0 / 255.0)).abs() < 0.01);
        assert!((normalized[1] - (64.0 / 255.0)).abs() < 0.01);
        assert!((normalized[2] - (192.0 / 255.0)).abs() < 0.01);
    }
}
