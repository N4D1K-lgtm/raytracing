use crate::core::intersection::Intersection;
use crate::core::math::{Vec2, AABB};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Vec3;
use std::sync::Arc;

use super::{Primitive, Rect, RectAxis};

/// Axis-aligned box primitive composed of 6 rectangles
pub struct Box3 {
    /// Minimum corner of the box
    pub min: Vec3,
    /// Maximum corner of the box
    pub max: Vec3,
    /// Material for all faces
    pub material: Arc<dyn Material>,
    /// Six faces of the box
    faces: Vec<Rect>,
}

impl Box3 {
    /// Create a new axis-aligned box
    pub fn new(min: Vec3, max: Vec3, material: Arc<dyn Material>) -> Self {
        let mut faces = Vec::with_capacity(6);

        // Front face (XY plane at max.z)
        faces.push(Rect::new(
            RectAxis::XY,
            Vec2::new(min.x, min.y),
            Vec2::new(max.x, max.y),
            max.z,
            Arc::clone(&material),
        ));

        // Back face (XY plane at min.z)
        faces.push(Rect::new(
            RectAxis::XY,
            Vec2::new(min.x, min.y),
            Vec2::new(max.x, max.y),
            min.z,
            Arc::clone(&material),
        ));

        // Top face (XZ plane at max.y)
        faces.push(Rect::new(
            RectAxis::XZ,
            Vec2::new(min.x, min.z),
            Vec2::new(max.x, max.z),
            max.y,
            Arc::clone(&material),
        ));

        // Bottom face (XZ plane at min.y)
        faces.push(Rect::new(
            RectAxis::XZ,
            Vec2::new(min.x, min.z),
            Vec2::new(max.x, max.z),
            min.y,
            Arc::clone(&material),
        ));

        // Right face (YZ plane at max.x)
        faces.push(Rect::new(
            RectAxis::YZ,
            Vec2::new(min.y, min.z),
            Vec2::new(max.y, max.z),
            max.x,
            Arc::clone(&material),
        ));

        // Left face (YZ plane at min.x)
        faces.push(Rect::new(
            RectAxis::YZ,
            Vec2::new(min.y, min.z),
            Vec2::new(max.y, max.z),
            min.x,
            Arc::clone(&material),
        ));

        Box3 {
            min,
            max,
            material,
            faces,
        }
    }

    /// Create a unit cube centered at origin
    pub fn unit_cube(material: Arc<dyn Material>) -> Self {
        Box3::new(
            Vec3::new(-0.5, -0.5, -0.5),
            Vec3::new(0.5, 0.5, 0.5),
            material,
        )
    }
}

impl Primitive for Box3 {
    fn intersect(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<Intersection> {
        let mut closest_hit: Option<Intersection> = None;
        let mut closest_t = t_max;

        // Test intersection with all six faces
        for face in &self.faces {
            if let Some(hit) = face.intersect(ray, t_min, closest_t) {
                closest_t = hit.t;
                closest_hit = Some(hit);
            }
        }

        closest_hit
    }

    fn intersect_p(&self, ray: &Ray, t_min: f64, t_max: f64) -> bool {
        // Fast AABB intersection test
        self.world_bounds().hit(ray, t_min, t_max)
    }

    fn world_bounds(&self) -> AABB {
        AABB::new(self.min, self.max)
    }

    fn surface_area(&self) -> f64 {
        let d = self.max - self.min;
        2.0 * (d.x * d.y + d.y * d.z + d.z * d.x)
    }

    fn sample(&self, u: Vec2) -> (Vec3, Vec3, f64) {
        // Randomly select a face proportional to its area
        // For simplicity, we'll just select one face uniformly
        // TODO: Proper area-weighted face selection

        let face_idx = (u.x * 6.0).floor() as usize % 6;
        let face_u = Vec2::new((u.x * 6.0).fract(), u.y);

        self.faces[face_idx].sample(face_u)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::material::Lambertian;

    #[test]
    fn test_box_intersection() {
        let material = Arc::new(Lambertian::new(Vec3::new(0.5, 0.5, 0.5)));

        // Unit box from (-1,-1,-1) to (1,1,1)
        let box3 = Box3::new(
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, 1.0, 1.0),
            material,
        );

        // Ray pointing at box from outside
        let ray = Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0));
        let hit = box3.intersect(&ray, 0.0, f64::INFINITY);

        assert!(hit.is_some());
        let isect = hit.unwrap();
        // Should hit front face at z=1
        assert!((isect.point.z - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_box_from_inside() {
        let material = Arc::new(Lambertian::new(Vec3::new(0.5, 0.5, 0.5)));

        let box3 = Box3::new(
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, 1.0, 1.0),
            material,
        );

        // Ray from inside box
        let ray = Ray::new(Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0));
        let hit = box3.intersect(&ray, 0.0, f64::INFINITY);

        assert!(hit.is_some());
        let isect = hit.unwrap();
        // Should hit front face at z=1
        assert!((isect.point.z - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_box_miss() {
        let material = Arc::new(Lambertian::new(Vec3::new(0.5, 0.5, 0.5)));

        let box3 = Box3::new(
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, 1.0, 1.0),
            material,
        );

        // Ray missing the box
        let ray = Ray::new(Vec3::new(5.0, 5.0, 5.0), Vec3::new(1.0, 0.0, 0.0));
        let hit = box3.intersect(&ray, 0.0, f64::INFINITY);

        assert!(hit.is_none());
    }

    #[test]
    fn test_box_bounds() {
        let material = Arc::new(Lambertian::new(Vec3::new(0.5, 0.5, 0.5)));

        let box3 = Box3::new(
            Vec3::new(-2.0, -3.0, -4.0),
            Vec3::new(2.0, 3.0, 4.0),
            material,
        );

        let bounds = box3.world_bounds();
        assert_eq!(bounds.min(), Vec3::new(-2.0, -3.0, -4.0));
        assert_eq!(bounds.max(), Vec3::new(2.0, 3.0, 4.0));
    }

    #[test]
    fn test_box_surface_area() {
        let material = Arc::new(Lambertian::new(Vec3::new(0.5, 0.5, 0.5)));

        // Box with dimensions 2x3x4
        let box3 = Box3::new(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(2.0, 3.0, 4.0),
            material,
        );

        // Surface area = 2*(2*3 + 3*4 + 4*2) = 2*(6 + 12 + 8) = 52
        assert!((box3.surface_area() - 52.0).abs() < 1e-6);
    }
}
