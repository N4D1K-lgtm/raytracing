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

    /// Create a camera with look_at orientation
    pub fn new_look_at(
        position: Vec3,
        target: Vec3,
        up: Vec3,
        vertical_fov: f64, // In degrees
        aspect_ratio: f64,
    ) -> Self {
        let theta = vertical_fov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h;
        let viewport_width = aspect_ratio * viewport_height;

        let forward = (target - position).normalized();
        let right = forward.cross(up).normalized();
        let camera_up = right.cross(forward);

        let origin = position;
        let horizontal_vec = right * viewport_width;
        let vertical_vec = camera_up * viewport_height;
        let lower_left_corner = origin + forward - horizontal_vec / 2.0 - vertical_vec / 2.0;

        Camera {
            origin,
            lower_left_corner,
            horizontal: horizontal_vec,
            vertical: vertical_vec,
        }
    }

    /// Update camera orientation from position and direction vectors
    pub fn update_orientation(&mut self, position: Vec3, forward: Vec3, right: Vec3, up: Vec3) {
        // Use the existing viewport dimensions
        let viewport_width = self.horizontal.length();
        let viewport_height = self.vertical.length();

        self.origin = position;
        self.horizontal = viewport_width * right;
        self.vertical = viewport_height * up;
        self.lower_left_corner = position + forward - self.horizontal / 2.0 - self.vertical / 2.0;
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
