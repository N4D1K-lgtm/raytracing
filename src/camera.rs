use crate::ray::Ray;
use crate::vec3::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    origin: Vec3,
    lower_left_corner: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
}

impl Camera {
    ///Create a new camera with custom aspect ratio
    pub fn new(aspect_ratio: f64) -> Self {
        let viewport_height = 2.0;
        let viewport_width = aspect_ratio * viewport_height;
        let focal_length = 1.0;

        let origin = Vec3::ZERO;
        let horizontal = Vec3::new(viewport_width, 0.0, 0.0);
        let vertical = Vec3::new(0.0, viewport_height, 0.0);
        let lower_left_corner =
            origin - horizontal / 2.0 - vertical / 2.0 - Vec3::new(0.0, 0.0, focal_length);

        Camera {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
        }
    }

    /// Generate a ray from the camera through viewport coordinates (u, v)
    /// u and v should be in range [0.0, 1.0]
    /// u = 0.0 is left edge, u = 1.0 is right edge
    /// v = 0.0 is bottom edge, v = 1.0 is top edge
    pub fn get_ray(&self, u: f64, v: f64) -> Ray {
        let direction =
            self.lower_left_corner + u * self.horizontal + v * self.vertical - self.origin;

        Ray::new(self.origin, direction)
    }
}

impl Default for Camera {
    /// Create a camera with 16:9 aspect ratio
    fn default() -> Self {
        Self::new(16.0 / 9.0)
    }
}
