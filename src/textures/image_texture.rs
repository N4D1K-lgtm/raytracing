use crate::core::math::Vec2;
use crate::vec3::Vec3;
use image::{DynamicImage, GenericImageView, Pixel};
use std::path::Path;
use std::sync::Arc;

use super::Texture;

/// Texture loaded from an image file
pub struct ImageTexture {
    image: Arc<DynamicImage>,
    width: u32,
    height: u32,
}

impl ImageTexture {
    /// Load an image texture from a file path
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, image::ImageError> {
        let image = image::open(path)?;
        let width = image.width();
        let height = image.height();

        Ok(ImageTexture {
            image: Arc::new(image),
            width,
            height,
        })
    }

    /// Create an image texture from an existing DynamicImage
    pub fn from_image(image: DynamicImage) -> Self {
        let width = image.width();
        let height = image.height();

        ImageTexture {
            image: Arc::new(image),
            width,
            height,
        }
    }

    /// Sample the image at normalized UV coordinates [0,1]
    fn sample_uv(&self, u: f64, v: f64) -> Vec3 {
        // Clamp UV coordinates to [0, 1]
        let u = u.clamp(0.0, 1.0);
        let v = 1.0 - v.clamp(0.0, 1.0); // Flip V (image coordinates vs texture coordinates)

        // Convert to pixel coordinates
        let i = ((u * self.width as f64) as u32).min(self.width - 1);
        let j = ((v * self.height as f64) as u32).min(self.height - 1);

        // Get pixel and convert to Vec3
        let pixel = self.image.get_pixel(i, j);
        let rgb = pixel.to_rgb();

        Vec3::new(
            rgb[0] as f64 / 255.0,
            rgb[1] as f64 / 255.0,
            rgb[2] as f64 / 255.0,
        )
    }
}

impl Texture for ImageTexture {
    fn evaluate(&self, uv: Vec2) -> Vec3 {
        self.sample_uv(uv.x, uv.y)
    }

    fn evaluate_3d(&self, point: Vec3) -> Vec3 {
        // For 3D sampling, use XY coordinates as UV
        self.sample_uv(point.x.fract(), point.y.fract())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgb};

    #[test]
    fn test_image_texture_from_buffer() {
        // Create a simple 2x2 test image
        // Image coordinates: (0,0) is top-left
        // UV coordinates: (0,0) is bottom-left (after V flip)
        let mut img = ImageBuffer::new(2, 2);

        // Set pixels in image coordinates:
        // Row 0: Red (0,0), Green (1,0)
        // Row 1: Blue (0,1), White (1,1)
        img.put_pixel(0, 0, Rgb([255, 0, 0]));     // Red - top-left in image
        img.put_pixel(1, 0, Rgb([0, 255, 0]));     // Green - top-right in image
        img.put_pixel(0, 1, Rgb([0, 0, 255]));     // Blue - bottom-left in image
        img.put_pixel(1, 1, Rgb([255, 255, 255])); // White - bottom-right in image

        let texture = ImageTexture::from_image(DynamicImage::ImageRgb8(img));

        // Sample corners in UV space (0,0) = bottom-left
        let bottom_left = texture.evaluate(Vec2::new(0.0, 0.0));   // Should be blue
        let bottom_right = texture.evaluate(Vec2::new(1.0, 0.0));  // Should be white
        let top_left = texture.evaluate(Vec2::new(0.0, 1.0));      // Should be red
        let top_right = texture.evaluate(Vec2::new(1.0, 1.0));     // Should be green

        // Check colors (with some tolerance for coordinate conversion)
        assert!(bottom_left.x < 0.1 && bottom_left.y < 0.1 && bottom_left.z > 0.9,
                "Expected blue at UV(0,0), got {:?}", bottom_left);
        assert!(bottom_right.x > 0.9 && bottom_right.y > 0.9 && bottom_right.z > 0.9,
                "Expected white at UV(1,0), got {:?}", bottom_right);
        assert!(top_left.x > 0.9 && top_left.y < 0.1 && top_left.z < 0.1,
                "Expected red at UV(0,1), got {:?}", top_left);
        assert!(top_right.x < 0.1 && top_right.y > 0.9 && top_right.z < 0.1,
                "Expected green at UV(1,1), got {:?}", top_right);
    }

    #[test]
    fn test_image_texture_uv_clamping() {
        // Create a simple 1x1 image
        let img = ImageBuffer::from_pixel(1, 1, Rgb([128, 128, 128]));
        let texture = ImageTexture::from_image(DynamicImage::ImageRgb8(img));

        // Test that out-of-bounds UVs are clamped
        let color1 = texture.evaluate(Vec2::new(-0.5, -0.5));
        let color2 = texture.evaluate(Vec2::new(1.5, 1.5));
        let color3 = texture.evaluate(Vec2::new(0.5, 0.5));

        // All should return the same gray color
        let expected = Vec3::new(128.0 / 255.0, 128.0 / 255.0, 128.0 / 255.0);

        assert!((color1.x - expected.x).abs() < 0.01);
        assert!((color2.x - expected.x).abs() < 0.01);
        assert!((color3.x - expected.x).abs() < 0.01);
    }

    #[test]
    fn test_image_texture_dimensions() {
        let img = ImageBuffer::new(100, 200);
        let texture = ImageTexture::from_image(DynamicImage::ImageRgb8(img));

        assert_eq!(texture.width, 100);
        assert_eq!(texture.height, 200);
    }
}
